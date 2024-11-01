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

use std::sync::Arc;
use clap::Parser;
use env_logger::Env;
use log::info;
use up_rust::communication::{InMemoryRpcServer, RpcServer};
use up_transport_zenoh::UPTransportZenoh;

mod request_processor;
mod request_handler;
mod connections;
mod config;

 #[tokio::main]
 async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ACTIVATE_HORN_METHOD_ID: u16 = 0x0001;
    const DEACTIVATE_HORN_METHOD_ID: u16 = 0x0002;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the horn service");
    let args = config::Args::parse();
    let (tx_kuksa, rx_kuksa) = tokio::sync::mpsc::channel(32);
    if args.kuksa_enabled {
        let parsed_uri = kuksa::Uri::try_from(args.kuksa_address.clone());
        if parsed_uri.is_ok() {
            tokio::spawn(connections::send_to_databroker(rx_kuksa, parsed_uri.unwrap()));
        } else {
            panic!("Shuting down due to invalid Kuksa address. Error is: {:?}", parsed_uri.err().unwrap().to_string());
        }
    } else {
        info!("Printing the horn signal to the terminal since the connection with Kuksa databroker is not enabled (use -k flag).");
        tokio::spawn(connections::send_to_terminal(rx_kuksa));
    }

    UPTransportZenoh::try_init_log_from_env();
    let transport = Arc::new(
         UPTransportZenoh::new(config::get_zenoh_config(args.clone()), "//horn-service-kuksa/1C/1/0")
             .await
             .unwrap(),
    );

    let uri_provider = transport.clone();
    let rpc_server = InMemoryRpcServer::new(transport.clone(), uri_provider.clone());

    let (tx_sequence, rx_sequence) = tokio::sync::mpsc::channel(4);
    tokio::spawn(request_processor::receive_requests(rx_sequence, tx_kuksa.clone()));

    let activate_horn_op = Arc::new(request_handler::ActivateHorn::new(tx_sequence.clone()));
    rpc_server
        .register_endpoint(None, ACTIVATE_HORN_METHOD_ID, activate_horn_op.clone())
        .await?;

    let deactivate_horn_op = Arc::new(request_handler::DeactivateHorn::new(tx_sequence.clone()));
    rpc_server
        .register_endpoint(None, DEACTIVATE_HORN_METHOD_ID, deactivate_horn_op.clone())
        .await?;

    std::thread::park();
    Ok(())
 }
