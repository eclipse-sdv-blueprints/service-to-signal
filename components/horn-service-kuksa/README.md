# Horn Service Kuksa

This component implements the COVEAS uService for the Horn. It uses the Zenoh transport for Eclipse uProtocol. To start the service simply run:

```bash
cargo run
```

You can use the following parameters to configure the `horn-service` implementation:

| Long Name | Short Name | Environment Variable | Default Value | Description |
|-----------|------------|----------------------|---------------|-------------|
| config |  |  |  | Path to set a configuration for the Eclipse Zenoh transport. If no path is set, the default values from Eclipse Zenoh are used. |
| listen |  | SERVICE_LISTEN | tcp/0.0.0.0:15000 | Endpoint on which the service listens and is available. |
| kuksa_address |  | KUKSA_ADDRESS | http://127.0.0.1:55556 | Endpoint at which the services tries to connect with an instance of the Kuksa Databroker. |
| kuksa_enabled | k | KUKSA_ENABLED | false | A flag to indicate whether an instance of the Kuksa Databroker is available. If `kuksa_enabled` is `false`, the service will only log changes of the horn status. |
