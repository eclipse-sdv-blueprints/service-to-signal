# Software Horn

The software horn emulates a horn hardware by logging the horn state and optionally making a horn sound. The software horn connects to an Kuksa databroker through a Zenoh router and an instance of the Zenoh-Kuksa provider, like the [actuation provider](../actuator-provider/README.md). This way, the software horn acts as replacement if the required hardware (ESP32) for the `actuation provider` is not available.

## Configuration

It is possible to configure the software horn with the following paramters:

| Long Name | Short Name | Environment Variable | Default Value | Description |
|-----------|------------|----------------------|---------------|-------------|
| config | c |  |  | Path to set a configuration for the Eclipse Zenoh transport. If no path is set, the default values from Eclipse Zenoh are used. |
| connect |  | ROUTER_ADDRESS | tcp/127.0.0.1:7447 | Endpoint on which the application tries to connect to an Eclipse Zenoh router. This value is only used if no Zenoh config is set (see `config`) |
| sound |  | IS_SOUND_ENABLED | true | A feature flag to enable the playback of sound while the Horn is activated. |
