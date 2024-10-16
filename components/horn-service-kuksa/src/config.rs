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

use std::path::PathBuf;

use up_transport_zenoh::zenoh_config;

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Args {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<PathBuf>,
    #[arg(short, long, default_value = "tcp/0.0.0.0:15000", env = "SERVICE_LISTEN")]
    /// Endpoints to listen on.
    listen: Vec<String>,
    #[arg(long, default_value = "http://127.0.0.1:55556", env = "KUKSA_ADDRESS")]
    //#[arg(long)]
    /// The address for the Kuksa Databroker
    pub kuksa_address: String,
    #[arg(long, short = 'k', env = "KUKSA_ENABLED")]
    /// Enables the connection to the Kuksa Databroker
    /// Otherwise the value of the horn signal is printed to the terminal.
    pub kuksa_enabled: bool,
}

pub fn get_zenoh_config(args: Args) -> zenoh_config::Config {
    // Load the config from file path
    let mut zenoh_cfg = match &args.config {
        Some(path) => zenoh_config::Config::from_file(path).unwrap(),
        None => zenoh_config::Config::default(),
    };

    // Set listener address
    if !args.listen.is_empty() {
        zenoh_cfg
            .listen
            .endpoints
            .set(args.listen.iter().map(|v| v.parse().unwrap()).collect())
            .unwrap();
    }

    zenoh_cfg.scouting.multicast.set_enabled(Some(false)).unwrap();
    zenoh_cfg
}
