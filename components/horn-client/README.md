# Horn Client

The `Horn Client` package implements a Client for the Horn service over Eclipse uProtocol from the COVESA uServices. This implementation relies on Eclipse Zenoh for the transport layer of the Eclipse uProtocol communication.

To start the client, execute:

```bash
cargo run
```

in this directory.

We recommend to configure the port at which the client searches to for the Horn service server, to not rely on the discovery and multicast features in Zenoh:

```bash
cargo run -- --connect tcp/127.0.0.1:15000
```

You can replace the default port `15000` with any other port, if the server runs on an alternative port. Instead of using the command line, it is also possible to configure the address by setting the environment variable `HORN_ADDRESS`, e.g., as in:

```bash
HORN_ADDRESS=tcp/127.0.0.1:15000 cargo run
```
