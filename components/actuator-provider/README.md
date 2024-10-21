# Actuator Provider

## Prerequisites

### Hardware Required

* A development board based on the ESP32 SoC.
   > Note, that this project was tested with the ESP-32-WROOM-32UE. According to the
   [Zenoh-pico project](https://github.com/eclipse-zenoh/zenoh-pico?tab=readme-ov-file#223-esp-idf),
   the implementation was also tested with the az-delivery-devkit-v4 ESP32 board. Make sure that the chip has at
   least 2MB flash size.
* USB cable for power supply and programming.

We use PlatformIO to install the necessary toolchain for building the actuator provider for the microcontroller.

1. Install PlatformIO Core (CLI) for your system:
   [PlatformIO installation instructions](https://docs.platformio.org/en/latest/core/installation/methods/index.html)

2. Configure the provider application with the header file [config.h](src/config.h).
Here, we need to configure the IP address of the Zenoh router to connect to.
To adjust the Wi-Fi configuration execute the following command:

   ```bash
   platformio run -t menuconfig
   ```

   This will build the menuconfig where you can set the SSID and password for
   the Wi-Fi connection under `Application Configuration > WiFi SSID`

3. Build, flash, and monitor the application using:

   ```bash
   platformio run
   platformio run -t upload
   platformio run -t monitor
   ```
