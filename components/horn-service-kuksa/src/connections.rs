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

use kuksa::{proto, Uri};
use log::{debug, error, info};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::select;

pub(crate) async fn send_to_databroker(mut rx: tokio::sync::mpsc::Receiver<bool>, uri: Uri) {
    info!("Connecting to Kuksa Databroker [{uri}]");
    let mut client = kuksa::Client::new(uri);
    while let Some(is_active) = rx.recv().await {
        debug!("Sending: {:?}", is_active);
        let ts = prost_types::Timestamp::from(SystemTime::now());
        let datapoints = HashMap::from([(
            "Vehicle.Body.Horn.IsActive".to_string(),
            proto::v1::Datapoint {
                timestamp: Some(ts),
                value: Some(proto::v1::datapoint::Value::Bool(is_active)),
            },
        )]);
        if let Err(e) = client.set_target_values(datapoints).await {
            error!("Failed to send the Horn signal to Kuksa Databroker: {e}");
        }
    }
}

pub(crate) async fn send_to_terminal(mut rx: tokio::sync::mpsc::Receiver<bool>) {
    let mut is_active = Some(false);
    while is_active.is_some() {
        is_active = select! {
            next_is_active = rx.recv() => next_is_active,
            _ = print_is_active(is_active.unwrap()) => is_active,
        }
    }
}

async fn print_is_active(is_active: bool) {
    let is_active_str = if is_active { '!' } else { '-' };
    loop {
        info!("{}", is_active_str);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}
