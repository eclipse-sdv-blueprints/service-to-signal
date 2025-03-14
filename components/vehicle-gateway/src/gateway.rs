use async_trait::async_trait;
use log::info;
use std::sync::Arc;
use up_rust::{UListener, UMessage, UMessageBuilder, UMessageType, UTransport, UUri};

#[allow(dead_code)]
pub(crate) struct GatewayForwarder {
    client: Arc<dyn UTransport>,
}
impl GatewayForwarder {
    #[allow(dead_code)]
    pub(crate) fn new(client: Arc<dyn UTransport>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UListener for GatewayForwarder {
    async fn on_receive(&self, msg: UMessage) {
        info!("ServiceResponseListener: Received a message: {msg:?}");

        let forward_authority = if &msg.attributes.source.authority_name == "cloud" {
            info!("forwarding message from cloud to vehicle");

            // if the message comes from the cloud side, the source authority should be matched to the device on which the uEntity lives
            "hcp5"
        } else {
            info!("forwarding message from vehicle to cloud");

            // if the message comes from the vehicle side, the source authority should become the VIN
            "WAUWAUGRRWAUWAU"
        };

        let mut forwarded_msg = match msg.attributes.type_.enum_value().unwrap() {
            UMessageType::UMESSAGE_TYPE_REQUEST => {
                let forward_address = UUri::try_from_parts(
                    forward_authority,
                    msg.attributes.sink.ue_id,
                    msg.attributes.sink.ue_version_major as u8,
                    msg.attributes.sink.resource_id as u16,
                )
                .unwrap();

                UMessageBuilder::request(
                    forward_address,
                    msg.attributes.source.get_or_default().to_owned(),
                    msg.attributes.ttl.unwrap(),
                )
            }
            UMessageType::UMESSAGE_TYPE_RESPONSE => {
                let forward_address = UUri::try_from_parts(
                    forward_authority,
                    msg.attributes.source.ue_id,
                    msg.attributes.source.ue_version_major as u8,
                    msg.attributes.source.resource_id as u16,
                )
                .unwrap();
                UMessageBuilder::response(
                    msg.attributes.sink.get_or_default().to_owned(),
                    msg.attributes.reqid.get_or_default().to_owned(),
                    forward_address,
                )
            }
            _ => {
                panic!()
            }
        };

        self.client
            .send(
                forwarded_msg
                    .build_with_payload(
                        msg.payload.unwrap(),
                        msg.attributes.payload_format.enum_value().unwrap(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}
