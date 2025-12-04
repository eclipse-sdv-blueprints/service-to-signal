/********************************************************************************
 * Copyright (c) 2024 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License 2.0 which is available at
 * http://www.eclipse.org/legal/epl-2.0
 *
 * SPDX-License-Identifier: EPL-2.0
 ********************************************************************************/

#include "driver/gpio.h"
#include <esp_event.h>
#include <esp_log.h>
#include <esp_system.h>
#include <esp_wifi.h>
#include <freertos/FreeRTOS.h>
#include <freertos/event_groups.h>
#include <freertos/task.h>
#include <nvs_flash.h>
#include <regex.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <zenoh-pico.h>

#include "config.h"

#if Z_FEATURE_SUBSCRIPTION == 1
#define WIFI_CONNECTED_BIT BIT0

static bool s_is_wifi_connected = false;
static EventGroupHandle_t s_event_group_handler;
static int s_retry_count = 0;

static z_owned_publisher_t pub;

static const char *TAG = "ACTUATOR PROVIDER";

typedef enum {
  SIGNAL_TYPE_CURRENT_VALUE,
  SIGNAL_TYPE_TARGET_VALUE,
  SIGNAL_TYPE_UNKNOWN
} signal_type_t;

void gpio_init() {
  gpio_reset_pin(LED_GPIO);
  gpio_set_direction(LED_GPIO, GPIO_MODE_OUTPUT);
}

int is_valid_locator(char *url) {
  const char pattern[] =
      "^tcp/([0-9]{1,3}\\.){3}[0-9]{1,3}:[0-9]+#iface=[a-zA-Z0-9_-]+$";
  regex_t reg;

  int compile_status = regcomp(&reg, pattern, REG_EXTENDED | REG_NOSUB);
  if (compile_status != 0) {
    char errbuf[100];
    regerror(compile_status, &reg, errbuf, sizeof(errbuf));
    ESP_LOGE(TAG, "Regex compile error: %s\n", errbuf);
    return -1;
  }

  int result = regexec(&reg, url, 0, NULL, 0);
  if (result != 0) {
    char errbuf[100];
    regerror(result, &reg, errbuf, sizeof(errbuf));
    ESP_LOGE(TAG, "Regex match error: %s\n", errbuf);
  }

  regfree(&reg);
  return result;
}

void pub_status(const char *value) {
  const char attachment_payload[] = "currentValue";
  char buffer[32] = {0};

  snprintf(buffer, sizeof(buffer), "%s", value);

  z_owned_bytes_t attachment;
  z_bytes_empty(&attachment);

  z_publisher_put_options_t options;
  z_publisher_put_options_default(&options);

  z_owned_bytes_t payload;
  z_bytes_copy_from_str(&payload, buffer);

  z_owned_encoding_t encoding;
  z_encoding_from_str(&encoding, "zenoh/string;utf8");

  z_bytes_copy_from_str(&attachment, attachment_payload);

  options.attachment = z_move(attachment);
  options.encoding = z_move(encoding);

  ESP_LOGI(TAG, "Publishing message...");
  z_publisher_put(z_loan(pub), z_move(payload), &options);
  ESP_LOGI(TAG, "Message published successfully");
}

signal_type_t attachment_handler(z_loaned_sample_t *sample) {
  const z_loaned_bytes_t *attachment = z_sample_attachment(sample);
  if (attachment != NULL) {
    ze_deserializer_t deserializer = ze_deserializer_from_bytes(attachment);
    size_t attachment_len;
    ze_deserializer_deserialize_sequence_length(&deserializer, &attachment_len);

    z_owned_string_t attachment_str;
    z_bytes_to_string(z_sample_attachment(sample), &attachment_str);
    ESP_LOGI(TAG, "    with attachment: %.*s\n",
             (int)z_string_len(z_loan(attachment_str)),
             z_string_data(z_loan(attachment_str)));

    signal_type_t result = SIGNAL_TYPE_UNKNOWN;

    const char *data = z_string_data(z_loan(attachment_str));
    size_t len = z_string_len(z_loan(attachment_str));

    if (strncmp(data, "currentValue", len) == 0) {
      result = SIGNAL_TYPE_CURRENT_VALUE;
    } else if (strncmp(data, "targetValue", len) == 0) {
      result = SIGNAL_TYPE_TARGET_VALUE;
    }

    z_drop(z_move(attachment_str));

    return result;
  }
  return SIGNAL_TYPE_UNKNOWN;
}

static void event_handler(void *arg, esp_event_base_t event_base,
                          int32_t event_id, void *event_data) {
  if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {
    esp_wifi_connect();
  } else if (event_base == WIFI_EVENT &&
             event_id == WIFI_EVENT_STA_DISCONNECTED) {
    if (s_retry_count < ESP_MAXIMUM_RETRY) {
      esp_wifi_connect();
      s_retry_count++;
    }
  } else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
    xEventGroupSetBits(s_event_group_handler, WIFI_CONNECTED_BIT);
    s_retry_count = 0;
  }
}

void wifi_init_sta(void) {
  s_event_group_handler = xEventGroupCreate();

  ESP_ERROR_CHECK(esp_netif_init());

  ESP_ERROR_CHECK(esp_event_loop_create_default());
  esp_netif_create_default_wifi_sta();

  wifi_init_config_t config = WIFI_INIT_CONFIG_DEFAULT();
  ESP_ERROR_CHECK(esp_wifi_init(&config));

  esp_event_handler_instance_t handler_any_id;
  esp_event_handler_instance_t handler_got_ip;
  ESP_ERROR_CHECK(esp_event_handler_instance_register(
      WIFI_EVENT, ESP_EVENT_ANY_ID, &event_handler, NULL, &handler_any_id));
  ESP_ERROR_CHECK(esp_event_handler_instance_register(
      IP_EVENT, IP_EVENT_STA_GOT_IP, &event_handler, NULL, &handler_got_ip));

  wifi_config_t wifi_config = {.sta = {
                                   .ssid = ESP_WIFI_SSID,
                                   .password = ESP_WIFI_PASS,
                               }};

  ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
  ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
  ESP_ERROR_CHECK(esp_wifi_start());

  EventBits_t bits =
      xEventGroupWaitBits(s_event_group_handler, WIFI_CONNECTED_BIT, pdFALSE,
                          pdFALSE, portMAX_DELAY);

  if (bits & WIFI_CONNECTED_BIT) {
    s_is_wifi_connected = true;
  }

  ESP_ERROR_CHECK(esp_event_handler_instance_unregister(
      IP_EVENT, IP_EVENT_STA_GOT_IP, handler_got_ip));
  ESP_ERROR_CHECK(esp_event_handler_instance_unregister(
      WIFI_EVENT, ESP_EVENT_ANY_ID, handler_any_id));
  vEventGroupDelete(s_event_group_handler);
}

void data_handler(z_loaned_sample_t *sample, void *ctx) {
  (void)(ctx);
  z_view_string_t keystr;
  z_keyexpr_as_view_string(z_sample_keyexpr(sample), &keystr);
  z_owned_string_t value;
  z_bytes_to_string(z_sample_payload(sample), &value);
  z_owned_string_t encoding;
  z_encoding_to_string(z_sample_encoding(sample), &encoding);

  ESP_LOGI(TAG, ">> [Subscriber] Received ('%.*s': '%.*s')\n",
           (int)z_string_len(z_loan(keystr)), z_string_data(z_loan(keystr)),
           (int)z_string_len(z_loan(value)), z_string_data(z_loan(value)));
  ESP_LOGI(TAG, "    with encoding: %.*s\n",
           (int)z_string_len(z_loan(encoding)),
           z_string_data(z_loan(encoding)));

  // Check timestamp
  const z_timestamp_t *ts = z_sample_timestamp(sample);
  if (ts != NULL) {
    ESP_LOGI(TAG, "    with timestamp: %" PRIu64 "\n",
             z_timestamp_ntp64_time(ts));
  }

  // Check attachment
  signal_type_t signal_type = attachment_handler(sample);

  if (signal_type == SIGNAL_TYPE_CURRENT_VALUE) {
    ESP_LOGI(TAG, "Received currentValue. Discarding signal.\n");
  }
  if (signal_type == SIGNAL_TYPE_TARGET_VALUE) {
    ESP_LOGI(TAG, "Received targetValue. \n");

    const char *value_data = z_string_data(z_loan(value));
    size_t value_len = z_string_len(z_loan(value));

    if (value_len == 4 && strncmp(value_data, "true", 4) == 0) {
      ESP_LOGI(TAG, "Turning on LED.");
      gpio_set_level(LED_GPIO, 1);
      pub_status("true");
    } else if (value_len == 5 && strncmp(value_data, "false", 5) == 0) {
      ESP_LOGI(TAG, "Turning off LED.");
      gpio_set_level(LED_GPIO, 0);
      pub_status("false");
    } else {
      ESP_LOGI(TAG, "Received unknown value: '%.*s'\n", (int)value_len,
               value_data);
    }
  }
  z_drop(z_move(value));
  z_drop(z_move(encoding));
}

void app_main() {

  esp_log_level_set(TAG, ESP_LOG_DEBUG);

  esp_err_t ret = nvs_flash_init();
  if (ret == ESP_ERR_NVS_NO_FREE_PAGES ||
      ret == ESP_ERR_NVS_NEW_VERSION_FOUND) {
    ESP_ERROR_CHECK(nvs_flash_erase());
    ret = nvs_flash_init();
  }
  ESP_ERROR_CHECK(ret);

  // Set WiFi in STA mode and trigger attachment
  ESP_LOGI(TAG, "Connecting to WiFi...");
  wifi_init_sta();
  while (!s_is_wifi_connected) {
    ESP_LOGI(TAG, ".");
    sleep(1);
  }
  ESP_LOGI(TAG, "Establishing the Wifi connection was successful!\n");

  // Initialize GPIO pin with led
  gpio_init();

  // Initialize Zenoh Session and other parameters
  z_owned_config_t config;
  z_config_default(&config);
  zp_config_insert(z_loan_mut(config), Z_CONFIG_MODE_KEY, MODE);
  if (is_valid_locator(LOCATOR) != 0) {
    ESP_LOGE(TAG, "Invalid locator format. Please use "
                  "'tcp/<ip>:<port>#<interface>' format.");
    exit(-1);
  }
  if (strcmp(LOCATOR, "") != 0) {
    if (strcmp(MODE, "client") == 0) {
      zp_config_insert(z_loan_mut(config), Z_CONFIG_CONNECT_KEY, LOCATOR);
    } else {
      zp_config_insert(z_loan_mut(config), Z_CONFIG_LISTEN_KEY, LOCATOR);
    }
  }

  // Open Zenoh session
  ESP_LOGI(TAG, "Opening Zenoh Session...");
  z_owned_session_t s;
  if (z_open(&s, z_move(config), NULL) < 0) {
    ESP_LOGE(TAG, "Unable to open session!");
    exit(-1);
  }
  ESP_LOGI(TAG, "Opening Zenoh session was succesful!\n");

  // Start the receive and the session lease loop for zenoh-pico
  zp_start_read_task(z_loan_mut(s), NULL);
  zp_start_lease_task(z_loan_mut(s), NULL);

  ESP_LOGI(TAG, "Declaring Subscriber on '%s'...", KEYEXPR);
  z_owned_closure_sample_t callback;
  z_closure(&callback, data_handler, NULL, NULL);
  z_owned_subscriber_t sub;
  z_view_keyexpr_t ke;
  z_view_keyexpr_from_str_unchecked(&ke, KEYEXPR);
  if (z_declare_subscriber(z_loan(s), &sub, z_loan(ke), z_move(callback),
                           NULL) < 0) {
    ESP_LOGE(TAG, "Unable to declare subscriber.");
    exit(-1);
  }
  ESP_LOGI(TAG, "OK!");

  ESP_LOGI(TAG, "Declaring Publisher on '%s'...", KEYEXPR);
  z_view_keyexpr_from_str_unchecked(&ke, KEYEXPR);
  if (z_declare_publisher(z_loan(s), &pub, z_loan(ke), NULL) < 0) {
    ESP_LOGE(TAG, "Unable to declare publisher for key expression!");
    exit(-1);
  }
  ESP_LOGI(TAG, "OK");

  while (1) {
    sleep(1);
  }

  ESP_LOGI(TAG, "Closing Zenoh Session...");
  z_drop(z_move(sub));

  z_drop(z_move(s));
  ESP_LOGI(TAG, "OK!");
}
#else
void app_main() {
  ESP_LOGE(TAG, "ERROR: Zenoh pico was compiled without Z_FEATURE_SUBSCRIPTION "
                "but this example requires it.");
}
#endif