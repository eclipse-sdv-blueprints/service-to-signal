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

use horn_proto::{horn_service::ActivateHornRequest, horn_topics::{HornMode, HornSequence}};
use log::{debug, error};
use tokio::select;

// Listens to the request channel and applies the requests. When the channel returns 'None',
// 'receive_requests' stops the execution of the previous request and the horn is deactived.
pub(crate) async fn receive_requests(
    mut rx_request_channel: tokio::sync::mpsc::Receiver<Option<ActivateHornRequest>>, 
    tx_kuksa: tokio::sync::mpsc::Sender<bool>) {
    let mut request;
    while let Some(request_inner) = rx_request_channel.recv().await {
        request = Some(request_inner);
        while request.is_some() {
            request = select! {
                req = rx_request_channel.recv() => req,
                req = request_apply(request.unwrap(), tx_kuksa.clone()) => req,
            }
        };
    }
}

async fn request_apply(request: Option<ActivateHornRequest>, tx_kuksa: tokio::sync::mpsc::Sender<bool>) -> Option<Option<ActivateHornRequest>> {
    match request {
        Some(request_inner) => {
            horn_request_apply(request_inner, tx_kuksa).await;
            None
        },
        None => {
            // treat None as a signal to deactivate the horn
            let _ = tx_kuksa.send(false).await;
            None
        },
    }
}

pub async fn horn_request_apply(req: ActivateHornRequest, tx_kuksa: tokio::sync::mpsc::Sender<bool>) {
    match req.mode.enum_value() {
        Ok(mode) => {
            match mode {
                HornMode::HM_SEQUENCED => {
                    let sequences = req.command;
                    horn_sequence_apply(sequences, tx_kuksa).await;
                },
                HornMode::HM_CONTINUOUS => horn_continous_apply(tx_kuksa).await,
                HornMode::HM_UNKNOWN => println!("Horn Mode: Unknown"),
                HornMode::HM_UNSPECIFIED => println!("Horn Mode: Unspecified"),
            };
        },
        Err(e) => error!("Error in Horn Mode value {:?}", e),
        };
}


pub async fn horn_continous_apply(tx_kuksa: tokio::sync::mpsc::Sender<bool>) {
    debug!("Starting Continous Horn");
    let _ = tx_kuksa.send(true).await;
}

pub async fn horn_sequence_apply(sequences: Vec<HornSequence>, tx_kuksa: tokio::sync::mpsc::Sender<bool>) {
    for sequence in sequences {
        for cycle in sequence.horn_cycles {
            debug!("\nOn Time: {}, Off Time: {}", cycle.on_time, cycle.off_time);
            let _ = tx_kuksa.send(true).await;
            let _ = tokio::time::sleep(std::time::Duration::from_millis(cycle.on_time as u64)).await;
            let _ = tx_kuksa.send(false).await;
            let _ = tokio::time::sleep(std::time::Duration::from_millis(cycle.off_time as u64)).await;
        }
    }
}
