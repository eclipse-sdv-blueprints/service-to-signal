/********************************************************************************
 * Copyright (c) 2024 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License 2.0 which is available at
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

#include <esp_event.h>
#include <esp_log.h>
#include <esp_system.h>
#include <esp_wifi.h>
#include <freertos/FreeRTOS.h>
#include <freertos/event_groups.h>
#include <freertos/task.h>
#include <nvs_flash.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <zenoh-pico.h>
#include "config.h"
#include "driver/gpio.h"
#include <regex.h>

#if Z_FEATURE_PUBLICATION == 1
static bool s_is_wifi_connected = false;
static EventGroupHandle_t s_event_group_handler;
static int s_retry_count = 0;

static const char *TAG = "MAIN";

static z_owned_publisher_t pub;

typedef enum
{
    SIGNAL_TYPE_CURRENT_VALUE,
    SIGNAL_TYPE_TARGET_VALUE,
    SIGNAL_TYPE_UNKNOWN
} signal_type_t;

#if Z_FEATURE_ATTACHMENT == 1

signal_type_t attachment_handler(z_bytes_t key, z_bytes_t value, void *ctx)
{
    (void)ctx;

    char type[50] = "";
    size_t type_len = value.len;
    strncpy(type, (const char *)value.start, type_len);
    type[type_len] = '\0';

    if (strcmp(type, "currentValue") == 0)
    {
        return SIGNAL_TYPE_CURRENT_VALUE;
    }
    else if (strcmp(type, "targetValue") == 0)
    {
        return SIGNAL_TYPE_TARGET_VALUE;
    }
    else
    {
        return SIGNAL_TYPE_UNKNOWN;
    }
}
#endif

int is_valid_tcp_url(char *url)
{
    char pattern[] = "^tcp/.*:[0-9]+$";
    int result;
    regex_t reg;

    if (regcomp(&reg, pattern, REG_EXTENDED | REG_NOSUB) != 0)
        return -1;

    result = regexec(&reg, url, 0, 0, 0);
    regfree(&reg);

    return result;
}

void gpio_init()
{
    gpio_reset_pin(LED_GPIO);
    gpio_set_direction(LED_GPIO, GPIO_MODE_OUTPUT);
}

void turn_led(bool on)
{
    if (on)
    {
        gpio_set_level(LED_GPIO, 1);
    }
    else if (!on)
    {
        gpio_set_level(LED_GPIO, 0);
    }
}

void pub_status(char *value_str)
{
    char buf[32];
    sprintf(buf, "%s", value_str);

    z_publisher_put_options_t options = z_publisher_put_options_default();
    z_owned_bytes_map_t map = z_bytes_map_new();
    z_bytes_map_insert_by_alias(&map, _z_bytes_wrap((uint8_t *)"type", 4), _z_bytes_wrap((uint8_t *)"currentValue", 12));
    options.attachment = z_bytes_map_as_attachment(&map);

    z_publisher_put(z_loan(pub), (const uint8_t *)buf, strlen(buf), &options);
}

char *payload_to_string(const z_bytes_t *payload)
{
    char *string = (char *)malloc(payload->len + 1);
    if (string == NULL)
    {
        fprintf(stderr, "Memory allocation failed\n");
        exit(EXIT_FAILURE);
    }

    for (size_t i = 0; i < payload->len; i++)
    {
        string[i] = payload->start[i];
    }

    string[payload->len] = '\0';

    return string;
}

void sample_handler(const z_sample_t *sample, void *arg)
{
    z_owned_str_t keystr = z_keyexpr_to_string(sample->keyexpr);
    ESP_LOGI(TAG, ">> [Subscriber handler] Received ('%s': '%.*s')\n",
             z_str_loan(&keystr), (int)sample->payload.len,
             sample->payload.start);

#if Z_FEATURE_ATTACHMENT == 1
    if (z_attachment_check(&sample->attachment))
    {
        int8_t type_result =
            z_attachment_iterate(sample->attachment, attachment_handler, NULL);

        if (type_result == SIGNAL_TYPE_CURRENT_VALUE)
        {
            ESP_LOGI(TAG, "[Subscriber handler] Received currentValue. Discarding signal.\n");
        }
        else if (type_result == SIGNAL_TYPE_TARGET_VALUE)
        {
            ESP_LOGI(TAG, "[Subscriber handler] Recieved targetValue\n");

            char *value_str = payload_to_string(&sample->payload);

            if (strcmp(value_str, "true") == 0)
            {
                ESP_LOGI(TAG, "[Subscriber handler] Activating the horn.\n");
                turn_led(true);

                pub_status(value_str);
                free(value_str);
            }
            else if (strcmp(value_str, "false") == 0)
            {
                ESP_LOGI(TAG, "[Subscriber handler] Turning off the horn.\n");
                turn_led(false);
                pub_status(value_str);
                free(value_str);
            }
            else
            {
                ESP_LOGI(TAG, "[Subscriber handler] Received a faulty payload value.");
            }
        }
        else if (type_result == SIGNAL_TYPE_UNKNOWN)
        {
            ESP_LOGI(TAG, "[Subscriber handler] Received an unknown signal type. Discarding the signal.\n");
        };
    };
#else
    ESP_LOGI(TAG, "The attachment feature is not enabled but is required for the full functionality.")
#endif

    z_str_drop(z_str_move(&keystr));
}

static void event_handler(void *arg, esp_event_base_t event_base,
                          int32_t event_id, void *event_data)
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START)
    {
        esp_wifi_connect();
    }
    else if (event_base == WIFI_EVENT &&
             event_id == WIFI_EVENT_STA_DISCONNECTED)
    {
        if (s_retry_count < ESP_MAXIMUM_RETRY)
        {
            esp_wifi_connect();
            s_retry_count++;
        }
    }
    else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP)
    {
        xEventGroupSetBits(s_event_group_handler, WIFI_CONNECTED_BIT);
        s_retry_count = 0;
    }
}

void wifi_init_sta(void)
{
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

    if (bits & WIFI_CONNECTED_BIT)
    {
        s_is_wifi_connected = true;
    }

    ESP_ERROR_CHECK(esp_event_handler_instance_unregister(
        IP_EVENT, IP_EVENT_STA_GOT_IP, handler_got_ip));
    ESP_ERROR_CHECK(esp_event_handler_instance_unregister(
        WIFI_EVENT, ESP_EVENT_ANY_ID, handler_any_id));
    vEventGroupDelete(s_event_group_handler);
}

void app_main()
{
    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES ||
        ret == ESP_ERR_NVS_NEW_VERSION_FOUND)
    {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
    ESP_ERROR_CHECK(ret);

    // Set WiFi in STA mode and trigger attachment
    ESP_LOGI(TAG, "Connecting to WiFi...");
    wifi_init_sta();
    while (!s_is_wifi_connected)
    {
        printf(".");
        sleep(1);
    }
    ESP_LOGI(TAG, "Establishing the Wifi connection was successful!\n");

    // Initialize GPIO pin with led
    gpio_init();

    // Initialize Zenoh Session and other parameters
    z_owned_config_t config = z_config_default();
    zp_config_insert(z_loan(config), Z_CONFIG_MODE_KEY, z_string_make(MODE));

    if (strcmp(CONNECT, "") == 0)
    {
        ESP_LOGI(TAG, "CONNECT string is empty. Using scouting to find peers in the network.\n");
    }
    else
    {
        if (is_valid_tcp_url(CONNECT))
        {
            zp_config_insert(z_loan(config), Z_CONFIG_CONNECT_KEY,
                             z_string_make(CONNECT));
        }
    }

    // Open Zenoh session
    ESP_LOGI(TAG, "Opening Zenoh session at %s\n", CONNECT);
    z_owned_session_t s = z_open(z_move(config));
    if (!z_check(s))
    {
        ESP_LOGE(TAG, "Unable to open session!\n");
        exit(-1);
    }
    ESP_LOGI(TAG, "Opening Zenoh session was succesful!\n");

    // Start the receive and the session lease loop for zenoh-pico
    zp_start_read_task(z_loan(s), NULL);
    zp_start_lease_task(z_loan(s), NULL);

    ESP_LOGI(TAG, "Declaring publisher for '%s'...", KEYEXPR);
    pub = z_declare_publisher(z_loan(s), z_keyexpr(KEYEXPR), NULL);
    if (!z_check(pub))
    {
        ESP_LOGE(TAG, "Unable to declare publisher for key expression!\n");
        exit(-1);
    }
    ESP_LOGI(TAG, "Successfully declared publisher for '%s'\n", KEYEXPR);

    ESP_LOGI(TAG, "Declaring subscriber on '%s'...", KEYEXPR);
    z_owned_closure_sample_t callback = z_closure(sample_handler);
    z_owned_subscriber_t sub = z_declare_subscriber(
        z_loan(s), z_keyexpr(KEYEXPR), z_move(callback), NULL);
    if (!z_check(sub))
    {
        ESP_LOGE(TAG, "Unable to declare subscriber.\n");
        exit(-1);
    }
    ESP_LOGI(TAG, "Succesfully declared subscriber on '%s'\n", KEYEXPR);

    while (1)
    {
        sleep(1);
    }

    ESP_LOGI(TAG, "Closing Zenoh session...\n");
    z_undeclare_subscriber(z_move(sub));

    // Stop the receive and the session lease loop for zenoh-pico
    zp_stop_read_task(z_loan(s));
    zp_stop_lease_task(z_loan(s));

    z_close(z_move(s));
    ESP_LOGI(TAG, "Successfully closed the Zenoh session.\n");
}
#else
void app_main()
{
    ESP_LOGI(TAG,
             "ERROR: Zenoh pico was compiled without Z_FEATURE_PUBLICATION but "
             "this example requires it.\n");
}
#endif
