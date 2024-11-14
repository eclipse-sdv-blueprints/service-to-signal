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

use clap::Parser;
use env_logger::Env;
use log::error;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use up_rust::communication::{CallOptions, InMemoryRpcClient, RpcClient, UPayload};
use up_transport_zenoh::zenoh_config;
use up_transport_zenoh::UPTransportZenoh;

use horn_proto::horn_service::{ActivateHornRequest, ActivateHornResponse, DeactivateHornRequest};
use horn_proto::horn_topics::{HornCycle, HornMode, HornSequence};

const HORN_SERVICE_AUTHORITY_NAME: &str = "horn-service-kuksa";
// see https://github.com/COVESA/uservices/blob/main/src/main/proto/vehicle/body/horn/v1/horn_service.proto
const HORN_SERVICE_ENTITY_ID: u32 = 28;
const ACTIVATE_HORN_RESOURCE_ID: u16 = 0x0001;
const DEACTIVATE_HORN_RESOURCE_ID: u16 = 0x0002;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    info!("Starting the client for the COVESA Horn service over uProtocol");

    let transport = UPTransportZenoh::new(args.get_zenoh_config()?, "//horn_client/1/1/0")
        .await
        .map(Arc::new)?;

    // The Zenoh transport happens to implement the
    // traits for UTransport and LocalUriProvider,
    // which is why it is used twice here.
    let rpc_client = InMemoryRpcClient::new(transport.clone(), transport).await?;
    let activate_horn_uri = up_rust::UUri::try_from_parts(
        HORN_SERVICE_AUTHORITY_NAME,
        HORN_SERVICE_ENTITY_ID,
        1,
        ACTIVATE_HORN_RESOURCE_ID,
    )?;
    let deactivate_horn_uri = up_rust::UUri::try_from_parts(
        HORN_SERVICE_AUTHORITY_NAME,
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

    let payload = UPayload::try_from_protobuf(horn_request)?;
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

    let deactivate_request = DeactivateHornRequest::default();

    let deactivate_payload = UPayload::try_from_protobuf(deactivate_request)?;

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
    let horn_request = ActivateHornRequest {
        mode: HornMode::HM_CONTINUOUS.into(),
        command: vec![HornSequence {
            horn_cycles: vec![],
            ..Default::default()
        }],
        ..Default::default()
    };

    let payload = UPayload::try_from_protobuf(horn_request)?;
    match rpc_client
        .invoke_method(
            activate_horn_uri.clone(),
            CallOptions::for_rpc_request(1_000, None, None, None),
            Some(payload),
        )
        .await
    {
        Ok(Some(payload)) => {
            let value = payload.extract_protobuf::<ActivateHornResponse>()?;
            info!(
                "Activate Horn returned message: {}",
                value.status.unwrap().code
            );
        }
        Ok(None) => error!("The activate horn request returned an empty response"),
        Err(e) => error!("The activate horn request returned the error: {:?}", e),
    }

    // Wait before deactivating the horn
    tokio::time::sleep(std::time::Duration::from_millis(4000)).await;

    let deactivate_request = DeactivateHornRequest::default();

    let deactivate_payload = UPayload::try_from_protobuf(deactivate_request)?;

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
    Ok(())
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Args {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<PathBuf>,
    #[arg(
        short = 'e',
        long,
        default_value = "tcp/127.0.0.1:15000",
        env = "HORN_ADDRESS"
    )]
    /// Endpoints to connect to.
    connect: Vec<String>,
}

impl Args {
    pub fn get_zenoh_config(&self) -> Result<zenoh_config::Config, Box<dyn std::error::Error>> {
        // Load the config from file path
        let mut zenoh_cfg = match &self.config {
            Some(path) => zenoh_config::Config::from_file(path)
                .map_err(|e| e as Box<dyn std::error::Error>)?,
            None => zenoh_config::Config::default(),
        };

        // Set connection address
        if !self.connect.is_empty() {
            zenoh_cfg
                .connect
                .endpoints
                .set(self.connect.iter().map(|v| v.parse().unwrap()).collect())
                .unwrap();
            info!("Setting Zenoh connect to {:?}", self.connect);
        }

        zenoh_cfg
            .scouting
            .multicast
            .set_enabled(Some(false))
            .unwrap();

        Ok(zenoh_cfg)
    }
}
