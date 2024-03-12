use bytes::{Buf, BytesMut};

use self::network::NetworkPayload;

pub mod command;
pub mod model;
pub mod network;

#[derive(Debug, PartialEq, Eq)]
pub enum ServicePayload {
    Unknown,
    Network(network::NetworkPayload),
    Model(model::ModelPayload),
    Command(command::CommandPayload),
}

impl ServicePayload {
    pub fn parse_network(bytes: &mut BytesMut, size: u32) -> Self {
        let payload = NetworkPayload::from_bytes(bytes, size as u8);
        Self::Network(payload.unwrap())
    }
}
