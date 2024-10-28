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
use log::{debug, info};
use std::path::PathBuf;
use std::str::FromStr;
use zenoh::buffers::ZBuf;
use zenoh::prelude::r#async::*;
use zenoh::publication::Publisher;
use zenoh::sample::{Attachment, Sample};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    info!("Starting the software horn connected over Eclipse Zenoh");

    let horn_keyexpr = String::from("Vehicle/Body/Horn/IsActive");

    let session = zenoh::open(get_zenoh_config(&args)).res().await.unwrap();

    let subscriber = session
        .declare_subscriber(&horn_keyexpr)
        .res()
        .await
        .unwrap();

    let publisher = session
        .declare_publisher(&horn_keyexpr)
        .res()
        .await
        .unwrap();

    debug!(
        "Waiting for messages on topic: {} and connecting to router at {}",
        &horn_keyexpr, args.connect
    );

    while let Ok(sample) = subscriber.recv_async().await {
        let attachement = extract_attachment_as_string(&sample);
        let value = zbuf_to_string(&sample.value.payload).unwrap();
        if attachement == "targetValue" {
            if value == "true" {
                info!("activate Horn");
                pub_current_status(&publisher, true).await;
            } else {
                info!("deactivate Horn");
                pub_current_status(&publisher, false).await;
            }
        }
    }

    Ok(())
}

pub async fn pub_current_status(publisher: &Publisher<'_>, status: bool) {
    let mut attachment = Attachment::new();
    attachment.insert("type", "currentValue");

    publisher
        .put(status.to_string())
        .with_attachment(attachment)
        .res()
        .await
        .unwrap();
}

pub fn get_zenoh_config(args: &Args) -> zenoh::config::Config {
    // Load the config from file path
    let mut zenoh_cfg = match &args.config {
        Some(path) => zenoh::config::Config::from_file(path).unwrap(),
        None => {
            debug!("No configuration file provided, using default configuration");
            zenoh::config::Config::default()
        }
    };

    // Set connection address
    if !args.connect.is_empty() {
        let endpoint = EndPoint::from_str(args.connect.as_str()).unwrap();
        zenoh_cfg.connect.endpoints.insert(0, endpoint);
        debug!("Inserted endpoint from connect argument");
    }

    zenoh_cfg
        .scouting
        .multicast
        .set_enabled(Some(false))
        .unwrap();

    zenoh_cfg
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Args {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<PathBuf>,
    #[arg(long, default_value = "tcp/127.0.0.1:7447", env = "ROUTER_ADDRESS")]
    /// Endpoints to connect to.
    connect: String,
    #[arg(short, long, default_value = "true", env = "IS_SOUND_ENABLED")]
    sound: bool,
}

pub fn extract_attachment_as_string(sample: &Sample) -> String {
    if let Some(attachment) = sample.attachment() {
        let bytes = attachment.iter().next().unwrap();
        String::from_utf8_lossy(bytes.1.as_slice()).to_string()
    } else {
        String::new()
    }
}

pub fn zbuf_to_string(zbuf: &ZBuf) -> Result<String, std::str::Utf8Error> {
    let mut bytes = Vec::new();
    for zslice in zbuf.zslices() {
        bytes.extend_from_slice(zslice.as_slice());
    }
    String::from_utf8(bytes).map_err(|e| std::str::Utf8Error::from(e.utf8_error()))
}
