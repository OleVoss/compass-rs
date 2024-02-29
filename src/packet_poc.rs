#[derive(Debug)]
enum ServicePacket {
    Unknown,
    Network(NetworkPayload),
}

struct BasePacket {
    packet_type: u8,
    raw_payload: u8,
}

trait Payload {
    fn from_base_packet(packet: BasePacket) -> Self;
}

impl Payload for NetworkPayload {
    fn from_base_packet(packet: BasePacket) -> Self {
        todo!()
    }
}

impl Into<NetworkPayload> for BasePacket {
    fn into(self) -> NetworkPayload {
        todo!()
    }
}

impl BasePacket {
    pub fn into_service(mut self) -> Result<ServicePacket, std::io::Error> {
        match &self.packet_type {
            0 => Ok(ServicePacket::Network { 0: self.into() }),
            _ => Ok(ServicePacket::Unknown),
        }
    }
}

#[derive(Debug)]
struct NetworkPayload;

fn handle_packet(packet: BasePacket) {
    if let Ok(service_packet) = packet.into_service() {
        match service_packet {
            ServicePacket::Unknown => todo!(),
            ServicePacket::Network(_) => todo!(),
        }
    }
}
