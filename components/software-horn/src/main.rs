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
use log::{debug, error, info, warn};
use std::path::PathBuf;
use zenoh::bytes::ZBytes;
use zenoh::pubsub::Publisher;
use zenoh::sample::Sample;
use zenoh::Config;

#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long, env = "ZENOH_CONFIG")]
    /// A Zenoh configuration file.
    config: PathBuf,
    #[arg(short, long, default_value = "true", env = "IS_SOUND_ENABLED")]
    sound: bool,
}

impl Args {
    pub fn get_zenoh_config(&self) -> Result<Config, Box<dyn std::error::Error>> {
        // Load the config from file path
        zenoh::config::Config::from_file(&self.config).map_err(|e| e as Box<dyn std::error::Error>)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let zenoh_config = args.get_zenoh_config()?;
    info!("Starting the software horn connected over Eclipse Zenoh");

    let horn_keyexpr = String::from("Vehicle/Body/Horn/IsActive");

    let session = zenoh::open(zenoh_config)
        .await
        .map_err(|e| e as Box<dyn std::error::Error>)?;

    let subscriber = session
        .declare_subscriber(&horn_keyexpr)
        .await
        .map_err(|e| e as Box<dyn std::error::Error>)?;

    let publisher = session
        .declare_publisher(&horn_keyexpr)
        .await
        .map_err(|e| e as Box<dyn std::error::Error>)?;
    debug!("Waiting for messages on topic: {}", &horn_keyexpr);

    while let Ok(sample) = subscriber.recv_async().await {
        if let Some(value_type) = extract_attachment_as_string(&sample) {
            if value_type == "targetValue" {
                match zbytes_to_string(sample.payload()) {
                    Ok(value) => {
                        if value == "true" {
                            info!("activate Horn");
                            pub_current_status(&publisher, true).await;
                        } else {
                            info!("deactivate Horn");
                            pub_current_status(&publisher, false).await;
                        }
                    }
                    Err(e) => error!("Payload from Zenoh message is not a String: {e}"),
                }
            }
        }
    }

    Ok(())
}

pub async fn pub_current_status(publisher: &Publisher<'_>, status: bool) {
    if let Err(e) = publisher
        .put(status.to_string())
        .attachment("currentValue")
        .await
    {
        warn!("failed to publish current status: {e}");
    }
}

pub fn extract_attachment_as_string(sample: &Sample) -> Option<String> {
    sample
        .attachment()
        .and_then(|a| a.try_to_string().map(|v| v.to_string()).ok())
}

pub fn zbytes_to_string(zbuf: &ZBytes) -> Result<String, std::str::Utf8Error> {
    zbuf.try_to_string().map(|v| v.to_string())
}
