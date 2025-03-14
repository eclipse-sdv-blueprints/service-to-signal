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
use log::info;
use std::sync::Arc;
use up_rust::{
    communication::{InMemoryRpcServer, RpcServer},
    StaticUriProvider, UTransport, UUri,
};
use up_transport_zenoh::UPTransportZenoh;

mod config;
mod connections;
mod request_handler;
mod request_processor;

const SERVICE_AUTHORITY: &str = "hcp5";
const SERVICE_UE_ID: u32 = 0x0003;
const ACTIVATE_HORN_METHOD_ID: u16 = 0x0001;
const DEACTIVATE_HORN_METHOD_ID: u16 = 0x0002;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the Horn service");
    let args = config::Args::parse();
    let (tx_kuksa, rx_kuksa) = tokio::sync::mpsc::channel(32);
    if args.kuksa_enabled {
        tokio::spawn(connections::send_to_databroker(
            rx_kuksa,
            args.kuksa_address.clone(),
        ));
    } else {
        info!("Printing the horn signal to the terminal since the connection with Kuksa databroker is not enabled (use -k flag).");
        tokio::spawn(connections::send_to_terminal(rx_kuksa));
    }

    UPTransportZenoh::try_init_log_from_env();

    let server_uuri = UUri::try_from_parts(SERVICE_AUTHORITY, SERVICE_UE_ID, 1, 0)?;
    let transport: Arc<dyn UTransport> =
        UPTransportZenoh::new(args.get_zenoh_config()?, server_uuri)
            .await
            .map(Arc::new)?;
    let transport_uuri = Arc::new(StaticUriProvider::new(SERVICE_AUTHORITY, SERVICE_UE_ID, 1));
    let rpc_server = InMemoryRpcServer::new(transport.clone(), transport_uuri);

    let (tx_sequence, rx_sequence) = tokio::sync::mpsc::channel(4);
    tokio::spawn(request_processor::receive_requests(
        rx_sequence,
        tx_kuksa.clone(),
    ));

    let activate_horn_op = Arc::new(request_handler::ActivateHorn::new(tx_sequence.clone()));
    rpc_server
        .register_endpoint(None, ACTIVATE_HORN_METHOD_ID, activate_horn_op)
        .await?;

    let deactivate_horn_op = Arc::new(request_handler::DeactivateHorn::new(tx_sequence.clone()));
    rpc_server
        .register_endpoint(None, DEACTIVATE_HORN_METHOD_ID, deactivate_horn_op)
        .await?;

    std::thread::park();
    Ok(())
}
