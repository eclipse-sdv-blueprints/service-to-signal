/********************************************************************************
 * Copyright (c) 2024 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

mod config;
mod gateway;

use clap::Parser;
use gateway::GatewayForwarder;
use log::info;
use std::sync::Arc;
use std::thread;
use up_rust::{UListener, UStatus, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;

const GATEWAY_AUTHORITY: &str = "WAUWAUGRRWAUWAU";
const GATEWAY_UE_ID: u32 = 0x0002;
const GATEWAY_UE_VERSION_MAJOR: u8 = 1;
const GATEWAY_RESOURCE_ID: u16 = 0;

#[tokio::main]
async fn main() -> Result<(), UStatus> {
    env_logger::init();
    let args = config::Args::parse();
    info!("Started zenoh_service");

    let service_uri = UUri::try_from_parts(
        GATEWAY_AUTHORITY,
        GATEWAY_UE_ID,
        GATEWAY_UE_VERSION_MAJOR,
        GATEWAY_RESOURCE_ID,
    )
    .unwrap();

    let gateway: Arc<dyn UTransport> = Arc::new(
        UPTransportZenoh::new(
            args.get_zenoh_config()
                .expect("Could not get Zenoh config."),
            service_uri,
        )
        .await?,
    );

    let mut source_filter = UUri::any();
    source_filter.authority_name = "cloud".to_string();
    let mut sink_filter = UUri::any();
    sink_filter.authority_name = "WAUWAUGRRWAUWAU".to_string();

    let gateway_forwarder: Arc<dyn UListener> = Arc::new(GatewayForwarder::new(gateway.clone()));
    gateway
        .register_listener(
            &source_filter,
            Some(&sink_filter),
            gateway_forwarder.clone(),
        )
        .await?;

    let mut source_filter = UUri::any();
    source_filter.authority_name = "hcp5".to_string();
    let mut sink_filter = UUri::any();
    sink_filter.authority_name = "cloud".to_string();

    let gateway_forwarder: Arc<dyn UListener> = Arc::new(GatewayForwarder::new(gateway.clone()));
    gateway
        .register_listener(
            &source_filter,
            Some(&sink_filter),
            gateway_forwarder.clone(),
        )
        .await?;

    thread::park();
    Ok(())
}
