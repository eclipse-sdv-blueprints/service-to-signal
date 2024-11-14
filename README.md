# service-to-signal-blueprint

In this Service-To-Signal Blueprint we show how one can implement a service over Eclipse uProtocol where the interface definition is part of the COVESA uServices. We use the Rust implementation of the Eclipse Zenoh transport of Eclipse uProtocol. The service implementation further relies on the interaction with VSS Signals, brokered in an Eclipse Kuksa Databroker.

![Overview of Service to Signal Blueprint](./img/overview.drawio.png)

In order to apply the requested changes we need a so-called Eclipse Kuksa _Provider_ to communicate the requested changes from the Kuksa Databroker to the underlying hardware.
In many current vehicles this might be done over a CAN-bus.
Since this blueprint focuses more on future vehicle generations and is intended as a technology showcase, we use Eclipse Zenoh instead.
We use the Eclipse Kuksa Zenoh Provider which forwards messages between the gRPC-API of the Eclipse Kuksa Databroker and topics in the Zenoh network concerning the relevant and configurable VSS signals.

Translating between the HTTP/2 based gRPC communication and Eclipse Zenoh makes it possible to connect embedded devices like an Arduino or ESP32 which can use the [PicoZenoh implementation](https://github.com/eclipse-zenoh/zenoh-pico).
If you do not have such hardware available, there is also the option to use a Zenoh client in software as the _software horn_.

## Components

In the following, we give a more detailed overview of the different involved components.

### Horn Service Kuksa

The [_Horn Service Kuksa_](./components/horn-service-kuksa/README.md) provides the interfaces defined in the [COVESA Horn uService](https://github.com/COVESA/uservices/blob/main/src/main/proto/vehicle/body/horn/v1/horn_service.proto).
The implementation utilizes the [`Vehicle.Body.Horn.IsActive` COVESA VSS Signal](https://github.com/COVESA/vehicle_signal_specification/blob/6024c4b29065b37c074649a1a65396b9d4de9b55/spec/Body/Body.vspec#L65) managed by the Eclipse Kuksa Databroker.

The service can be invoked by means of uProtocol using Eclipse Zenoh as the transport layer.

### Horn Client

The [_Horn Client_](./components/horn-client/README.md) is an application that interacts with _Horn Service Kuksa_ to trigger the execution of specific Horn sequences.

### Eclipse Kuksa Databroker

The [Kuksa Databroker](https://github.com/eclipse-kuksa/kuksa-databroker) acts as a vehicle abstraction layer and manages the interaction between applications and vehicle signals defined in the [COVESA Vehicle Signal Specification]((https://github.com/COVESA/vehicle_signal_specification/).
Consumers of the `kuksa.val.v1` API, implemented by the Kuksa Databroker, can get, subscribe and write to the target or the current value of such a signal within the Kuksa Databroker.

### ESP32 Activator Provider

We also need a component which actually performs the signaling of the Horn. In the easiest setup this is a small program which writes to the console whenever the VSS signal `Vehicle.Body.Horn.IsActive` is `True`. To make the setup a bit more realisitic we decided to integrate hardware like an ESP32 for which there is the [actuator-provider](./components/actuator-provider/)

These smaller hardware platforms struggle with running a full HTTP/2 based gRPC stack which is one of the reasons to utilize Eclipse Zenoh with the [Zenoh-Pico](https://github.com/eclipse-zenoh/zenoh-pico) implementation as transport here.

### Software Horn

To allow a quick setup of the overall system and in case you do not have an ESP32 hardware available to run the [embedded horn activator](#embedded-horn-activator), there is an alternative software horn. This components connects to the Zenoh router as well and logs the state of the Horn to the console. Optionally, the software horn can play a sound when the horn is active.

### Zenoh Kuksa Provider

For the integration of the hardware controlling the horn we use Eclipse Zenoh&trade; as transport.
The [horn actuator provider](#embedded-horn-activator) publishes and subscribes on a Zenoh topic which is derived from the respective COVESA VSS signal name in the Kuksa Databroker.
In the case of the Horn the topic is `Vehicle/Body/Horn/IsActive`.
It is then the responsibility of the Zenoh-Kuksa-Provider to listen to these topics and forward the messages between the Zenoh network and the Kuksa Databroker using gRPC.

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

* [Horn Service](#horn-service-implementation)
* [Kuksa Databroker](#kuksa-databroker)
* [Kuksa-Zenoh Provider](#kuksa-zenoh-provider)
* [Zenoh Router](#zenoh-router)
* [Software Horn](#software-horn)

You can then run the [horn client](#horn-client-app) to invoke the Horn Service.

1. In `components/horn-client/` run:

```bash
cargo run
```

For more details read the documentation in the [horn client Readme](./components/horn-client/README.md).

This requires that you installed the [Rust toolchain](https://rustup.rs) on your computer. As an alternative you can umcomment the section for the `horn-client` in the `service-to-signal-compose.yaml` and re-deploy the modified Docker Compose setup.

4. Check logs for Horn

To see the status of the Horn and check whether the setup worked you can read the logs of the `software-horn`. To do this run:

```bash
docker logs -f software-horn
```

### Optional: Configuring and starting the actuator provider (microcontroller implementation)

If you have the necessary hardware, you can replace the software-based horn with a
microcontroller-based actuator provider. To configure and build the application for
the microcontroller, follow the instructions provided for the
[actuator-provider](./components/actuator-provider/README.md).
