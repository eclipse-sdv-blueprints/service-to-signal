# service-to-signal-blueprint

In this Service-To-Signal Blueprint we show how one can implement a service over Eclipse uProtocol where the interface definition is part of the COVESA uServices. We use the Rust implementation of the Eclipse Zenoh transport of Eclipse uProtocol. The service implementation further relies on the interaction with VSS Signals, brokered in an Eclipse Kuksa Databroker.

<img src=./img/overview.drawio.png>

In order to apply, the requested changes we need a so-called provider to communicate the requested changes from the Kuksa databroker to the underlying hardware. In many current vehicles this might be done over a CAN-bus. Since this blueprint focuses more on future vehicle generations and is intended as a technology showcase, we use Eclipse Zenoh instead.
We use the Eclipse Kuksa Zenoh Provider which forwards messages between the gRPC-API of the Eclipse KUKSA Databroker and topics in the Zenoh network concerning the relevant and configurable VSS signals.

Translating between the HTTP/2 based gRPC communication and Eclipse Zenoh makes it possible to connect embedded devices like an Arduino or ESP32 which can use the [PicoZenoh implementation](https://github.com/eclipse-zenoh/zenoh-pico).
If you do not have such hardware available, there is also the option to use a Zenoh client in software as "software horn".

## Components

In the following, we give a more detailed overview of the different involved components.

### Horn Service Implementation

The [horn service implementation](./components/horn-service-kuksa/) provides the interfaces defined in the [COVESA uService for Horn](https://github.com/COVESA/uservices/blob/main/src/main/proto/vehicle/body/horn/v1/horn_service.proto).
The implementation relies in the [COVESA VSS Signal `Vehicle.Body.Horn.IsActive`](https://github.com/COVESA/vehicle_signal_specification/blob/6024c4b29065b37c074649a1a65396b9d4de9b55/spec/Body/Body.vspec#L65) managed by the Eclipse KUKSA Databroker.
We use Eclipse Zenoh as transport for the provided Horn service over Eclipse uProtocol.

### Horn Client (App)

The App is a consumer of the Horn service and triggers the execution of specific Horn sequences through this interface. There is the [horn-client](./components/horn-client/) which interacts with  the horn service in a pre-defined way and serves as a basic example of how an app can use the horn service.

### up-Simulator

The [up-simulator](https://github.com/eclipse-uprotocol/up-simulator?tab=readme-ov-file) is a general purpose application to interact with services over uProtocol especially for testing purposes.
A developer can run the up-simulator and execute publish-subscribed actions or remote procedure calls over the transports for Zenoh, Android and SOME/IP in uProtocol.
The up-simulator is not a direct part of this blueprint but you can use it to test out more complex usage sceneraios of the horn-service.

### KUKSA Databroker

The [KUKSA Databroker](https://github.com/eclipse-kuksa/kuksa-databroker) acts as a vehicle abstraction layer and manages the interaction between applications and vehicle signals defined in the Vehicle Signal Specification.
Consumers of the kuksa.val.v1 API, implemented by the KUKSA Databroker, can get, subscribe and write to the target or the current value of such a signal within the KUKSA Databroker.

### Embedded Horn Activator

We need one component which actually performs the signaling of the Horn. In the easiest setup this is a small program which writes to the console whenever the VSS signal `Vehicle.Body.Horn.IsActive` is `True`. To make the setup a bit more realisitic we decided to integrate hardware like an ESP32 for which there is the [actuator-provider](./components/actuator-provider/)

These smaller hardware platforms struggle with running a full HTTP/2 based gRPC stack which is one of the reasons to utilize Zenoh with the [Zenoh-Pico](https://github.com/eclipse-zenoh/zenoh-pico) implementation as transport here.

### Software Horn

To allow a quick setup of the overall system and in case you do not have an ESP32 hardware available to run the [embedded horn activator](#embedded-horn-activator), there is an alternative software horn. This components connects to the Zenoh router as well and logs the state of the Horn to the console. Optionally, the software horn can play a sound when the horn is active.

### Kuksa Zenoh Provider

For the integration of the hardware controlling the horn, we use Zenoh as transport. So the [horn actuator provider](#embedded-horn-activator) publishes and subscribes on a Zenoh topic which is derived from the respective COVESA VSS signal in the KUKSA Databroker.
In the case of the Horn, the topic is `Vehicle/Body/Horn/IsActive`.
It is then the responsibility of the Zenoh-Kuksa-Provider to listen to these topics and forward the messages between the Zenoh network and the Kuksa Databroker using gRPC.

### Zenoh Router

The Zenoh router routes message between the [Kuksa-Zenoh-Provider](#kuksa-zenoh-provider) and the Horn ([Embedded Horn Activator](#embedded-horn-activator) or [Software Horn](#software-horn)). We use the upstream available [Docker image](https://zenoh.io/docs/getting-started/quick-test/#run-zenoh-in-docker) of the Zenoh router and reference it in the [Docker Compose file](./service-to-signal-compose.yaml).

## Quick Start

### Configuring the zenoh-kuksa-provider

1. Initialize the kuksa-incubation submodule to add the zenoh-kuksa-provider to your components directory:

```bash
git submodule update --init
```

As an alternative you can pull the service-to-signal repository directly by executing: 

```bash
git clone --recurse-submodules https://github.com/eclipse-sdv-blueprints/service-to-signal.git
```


2. After that, the easiest way to set up and start the services is by means of using the Docker Compose file in the top
level directory:

```bash
docker compose -f service-to-signal-compose.yaml up --build --detach
```

This will pull or build (if necessary) the container images, create, and start the required components, namely:

* [horn-service-kuksa](#horn-service-implementation)
* [Kuksa Databroker](#kuksa-databroker)
* [Kuksa-Zenoh Provider](#kuksa-zenoh-provider)
* [Zenoh Router](#zenoh-router)
* [Software Horn](#software-horn)

As a result the `horn-service-kuksa` becomes available on port 15000 on the host machine. You can then run the [horn client](#horn-client-app) to invoke the `horn-service-kuksa`.

3. In `components/horn-client/` run:

```bash
cargo run 
```

For more details read the documentation in the [horn client Readme](./components/horn-client/README.md).

> This requires that you installed the [Rust toolchain](https://rustup.rs) on your computer. As an alternative you can umcomment the section for the `horn-client` in the [service-to-signal-compose.yaml](./service-to-signal-compose.yaml) and re-deploy the modified Docker Compose setup.

4. Check logs for Horn 

To see the status of the Horn and check whether the setup worked you can read the logs of the `software-horn`. To do this run: 

```bash
docker logs software-horn
```