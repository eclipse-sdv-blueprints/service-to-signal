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

use kuksa::Uri;
use up_transport_zenoh::zenoh_config;

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Args {
    #[arg(short, long)]
    /// A Zenoh configuration file.
    config: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value = "tcp/0.0.0.0:15000",
        env = "SERVICE_LISTEN"
    )]
    /// Endpoints to listen on.
    listen: Vec<String>,

    #[arg(long, default_value = "http://127.0.0.1:55556", env = "KUKSA_ADDRESS", value_parser = valid_uri)]
    /// The address for the Kuksa Databroker
    pub kuksa_address: Uri,

    #[arg(long, short = 'k', default_value = "false", env = "KUKSA_ENABLED")]
    /// Enables the connection to the Kuksa Databroker
    /// Otherwise the value of the horn signal is printed to the terminal.
    pub kuksa_enabled: bool,
}

fn valid_uri(uri: &str) -> Result<Uri, String> {
    kuksa::Uri::try_from(uri).map_err(|e| format!("invalid Kuksa Databroker URI: {e}"))
}

impl Args {
    pub fn get_zenoh_config(&self) -> Result<zenoh_config::Config, Box<dyn std::error::Error>> {
        // Load the config from file path
        let mut zenoh_cfg = self
            .config
            .as_ref()
            .map_or_else(
                || Ok(zenoh_config::Config::default()),
                zenoh_config::Config::from_file,
            )
            .map_err(|e| e as Box<dyn std::error::Error>)?;

        // Set listener address
        if !self.listen.is_empty() {
            zenoh_cfg
                .listen
                .endpoints
                .set(self.listen.iter().map(|v| v.parse().unwrap()).collect())
                .map_err(|_e| "Failed to set listener endpoints")?;
        }

        zenoh_cfg
            .scouting
            .multicast
            .set_enabled(Some(false))
            .map_err(|_e| "Failed to disable multicast scouting")?;
        Ok(zenoh_cfg)
    }
}
