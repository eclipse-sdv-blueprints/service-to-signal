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

#define CONFIG_H
/* This uses Wi-Fi configuration that you can set via project configuration menu

   If you'd rather not, change the below entries to strings with
   the config you want - i.e., #define ESP_WIFI_SSID "mywifissid"
*/
#define ESP_WIFI_SSID                       CONFIG_ESP_WIFI_SSID
#define ESP_WIFI_PASS                       CONFIG_ESP_WIFI_PASSWORD
#define ESP_MAXIMUM_RETRY                   CONFIG_ESP_MAXIMUM_RETRY
#define WIFI_CONNECTED_BIT                  BIT0

#define                                     CLIENT_OR_PEER 0 // 0: Client mode; 1: Peer mode
#if CLIENT_OR_PEER == 0
#define MODE                                "client"
/*
LOCATOR specifies the connection string to the Zenoh router.
Replace <ip> with the IP address of the machine hosting the blueprint Docker setup.
Format: "tcp/<ip>:7447#iface=docker0".
*/
#define LOCATOR                             ""
#elif CLIENT_OR_PEER == 1
#define MODE                                "peer"
/*
 * Hint: Set this to "<protocol>/<ip>:<port>#<interface>" even when LOCATOR is empty to configure the scouting mechanism.
 * - Zenoh router (zenohd) typically listens on udp/224.0.0.224:7446
*/
#define LOCATOR                             ""
#else
#error "Unknown Zenoh operation mode. Check CLIENT_OR_PEER value."
#endif

#define KEYEXPR                             "Vehicle/Body/Horn/IsActive" // The key to subscribe/publish to
#define LED_GPIO                            GPIO_NUM_25 // Number of the GPIO pin with the LED connected
