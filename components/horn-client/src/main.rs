/*******************************************************************************
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
*******************************************************************************/

use env_logger::Env;
use log::error;
use log::info;
use std::sync::Arc;
use up_rust::{
    communication::{CallOptions, InMemoryRpcClient, RpcClient, UPayload},
    StaticUriProvider, UTransport,
};
use up_transport_mqtt5::{Mqtt5Transport, MqttClientOptions, TransportMode};

use horn_proto::horn_service::{ActivateHornRequest, ActivateHornResponse, DeactivateHornRequest};
use horn_proto::horn_topics::{HornCycle, HornMode, HornSequence};

const VEHICLE_AUTHORITY: &str = "WAUWAUGRRWAUWAU";
const HORN_SERVICE_ENTITY_ID: u32 = 0x0003;
const ACTIVATE_HORN_RESOURCE_ID: u16 = 1;
const DEACTIVATE_HORN_RESOURCE_ID: u16 = 2;

const APP_AUTHORITY: &str = "cloud";
const APP_UE_ID: u32 = 0x0001;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting the client for the COVESA Horn service over uProtocol");

    let mut mqtt_options = MqttClientOptions::default();
    mqtt_options.broker_uri =
        std::env::var("MQTT_HOSTNAME").unwrap() + ":" + &std::env::var("MQTT_PORT").unwrap();

    let mqtt5_transport = Mqtt5Transport::new(
        TransportMode::InVehicle,
        mqtt_options,
        APP_AUTHORITY.to_string(),
    )
    .await?;
    mqtt5_transport.connect().await?;

    let transport: Arc<dyn UTransport> = Arc::new(mqtt5_transport);
    let transport_uuri = Arc::new(StaticUriProvider::new(APP_AUTHORITY, APP_UE_ID, 1));

    let rpc_client = InMemoryRpcClient::new(transport, transport_uuri).await?;
    let activate_horn_uri = up_rust::UUri::try_from_parts(
        VEHICLE_AUTHORITY,
        HORN_SERVICE_ENTITY_ID,
        1,
        ACTIVATE_HORN_RESOURCE_ID,
    )?;
    let deactivate_horn_uri = up_rust::UUri::try_from_parts(
        VEHICLE_AUTHORITY,
        HORN_SERVICE_ENTITY_ID,
        1,
        DEACTIVATE_HORN_RESOURCE_ID,
    )?;
    let horn_request = ActivateHornRequest {
        mode: HornMode::HM_SEQUENCED.into(),
        command: vec![HornSequence {
            horn_cycles: vec![
                HornCycle {
                    on_time: 100,
                    off_time: 100,
                    ..Default::default()
                },
                HornCycle {
                    on_time: 200,
                    off_time: 300,
                    ..Default::default()
                },
                HornCycle {
                    on_time: 100,
                    off_time: 200,
                    ..Default::default()
                },
                HornCycle {
                    on_time: 10000,
                    off_time: 500,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    let deactivate_request = DeactivateHornRequest::default();

    loop {
        let payload = UPayload::try_from_protobuf(horn_request.clone())?;

        match rpc_client
            .invoke_method(
                activate_horn_uri.clone(),
                CallOptions::for_rpc_request(1_000, None, None, None),
                Some(payload),
            )
            .await
        {
            Ok(Some(payload)) => {
                let response = payload.extract_protobuf::<ActivateHornResponse>()?;
                info!("Activate Horn returned message: {}", response);
            }
            Ok(None) => info!("The activate horn request returned an empty response"),
            Err(e) => error!("The activate horn request returned the error: {:?}", e),
        }

        // Wait before deactivating the horn
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        let deactivate_payload = UPayload::try_from_protobuf(deactivate_request.clone())?;

        match rpc_client
            .invoke_method(
                deactivate_horn_uri.clone(),
                CallOptions::for_rpc_request(1_000, None, None, None),
                Some(deactivate_payload),
            )
            .await
        {
            Ok(Some(_)) => {
                info!("The deactivate horn request returned successfully");
            }
            Ok(None) => error!("The deactivate horn request returned an empty response"),
            Err(e) => error!("The deactivate horn request returned the error: {:?}", e),
        }
    }
}
