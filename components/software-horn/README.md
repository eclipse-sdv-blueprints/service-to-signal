# Software Horn

The software horn emulates a horn hardware by logging the horn state and optionally making a horn sound. The software horn connects to an Kuksa databroker through a Zenoh router and an instance of the Zenoh-Kuksa provider, like the [actuation provider](../actuator-provider/README.md). This way, the software horn acts as replacement if the required hardware (ESP32) for the `actuation provider` is not available.

## Configuration

The service supports several configuration options that can be provided on the command line or via environment variables.
Please use the `--help` switch to get all relevant information:

```bash
cargo run -- --help
```
