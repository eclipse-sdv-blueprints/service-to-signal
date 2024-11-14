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

use horn_proto::horn_service::{
    ActivateHornRequest, ActivateHornResponse, DeactivateHornRequest, DeactivateHornResponse,
};
use horn_proto::status::Status;
use log::info;
use protobuf::MessageField;
use up_rust::communication::{RequestHandler, ServiceInvocationError, UPayload};

pub(crate) struct ActivateHorn {
    tx_sequence_channel: tokio::sync::mpsc::Sender<Option<ActivateHornRequest>>,
}

impl ActivateHorn {
    pub fn new(
        tx_sequence_channel: tokio::sync::mpsc::Sender<Option<ActivateHornRequest>>,
    ) -> Self {
        Self {
            tx_sequence_channel,
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for ActivateHorn {
    async fn handle_request(
        &self,
        _resource_id: u16,
        request_payload: Option<UPayload>,
    ) -> Result<Option<UPayload>, ServiceInvocationError> {
        info!("Handle new request to apply horn sequence");

        let req = request_payload
            .unwrap()
            .extract_protobuf::<ActivateHornRequest>()
            .unwrap();
        let _ = self.tx_sequence_channel.send(Some(req.clone())).await;

        let response = ActivateHornResponse {
            status: MessageField::some(Status::new()),
            ..Default::default()
        };
        let payload = UPayload::try_from_protobuf(response).unwrap();
        Ok(Some(payload))
    }
}

pub(crate) struct DeactivateHorn {
    tx_sequence_channel: tokio::sync::mpsc::Sender<Option<ActivateHornRequest>>,
}

impl DeactivateHorn {
    pub fn new(
        tx_sequence_channel: tokio::sync::mpsc::Sender<Option<ActivateHornRequest>>,
    ) -> Self {
        Self {
            tx_sequence_channel,
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for DeactivateHorn {
    async fn handle_request(
        &self,
        _resource_id: u16,
        request_payload: Option<UPayload>,
    ) -> Result<Option<UPayload>, ServiceInvocationError> {
        info!("Handle new deactivation request for the horn.");

        //Expect the deactivate horn request
        //to be empty.
        let _req = request_payload
            .unwrap()
            .extract_protobuf::<DeactivateHornRequest>()
            .unwrap();
        let _ = self.tx_sequence_channel.send(None).await;
        let response = DeactivateHornResponse {
            status: MessageField::some(Status::new()),
            ..Default::default()
        };
        let payload = UPayload::try_from_protobuf(response).unwrap();
        Ok(Some(payload))
    }
}
