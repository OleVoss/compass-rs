use bytes::{Buf, BytesMut};
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B3},
};

fn addr_from_normal_mode(bytes: &mut BytesMut, addr_size: u8) -> (u32, u8) {
    match addr_size {
        0 => (bytes.get_u8() as u32, bytes.get_u8()),
        1 => (bytes.get_u16_le() as u32, bytes.get_u8()),
        2 => {
            let system_addr = [bytes.get_u8(), bytes.get_u8(), bytes.get_u8(), 0];
            return (u32::from_le_bytes(system_addr), bytes.get_u8());
        }
        3 => (bytes.get_u32_le(), bytes.get_u8()),
        _ => (0, 0),
    }
}

fn addr_from_micro_mode(bytes: &mut BytesMut, addr_size: u8) -> (u32, u8) {
    let addr_byte = bytes.get_u8();
    match addr_size {
        0 => (addr_byte as u32, 0),
        1 => (((addr_byte >> 2) & 0b00111111) as u32, (addr_byte & 0b11)),
        2 => (((addr_byte >> 4) & 0b00001111) as u32, (addr_byte & 0b1111)),
        3 => (
            ((addr_byte >> 6) & 0b00000011) as u32,
            (addr_byte & 0b111111),
        ),
        _ => (0, 0),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NetworkPayload {
    pub network_type: NetworkType,
    pub entries: Vec<NetworkEntry>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NetworkType {
    ACK,
    Beacon,
    RequestBeacon,
    ExtendedBeacon,
}

// TODO: type-state pattern?
impl NetworkType {
    pub fn value(&self) -> Option<u8> {
        match self {
            NetworkType::ACK => None,
            NetworkType::Beacon => Some(0),
            NetworkType::RequestBeacon => Some(1),
            NetworkType::ExtendedBeacon => Some(5),
        }
    }
}

impl NetworkPayload {
    fn parse_entry(bytes: &mut BytesMut, info: bool) -> Result<NetworkEntry, std::io::Error> {
        let flag_byte = bytes.get_u8();
        let dtn = (flag_byte >> 7) & 1;
        let active = (flag_byte >> 6) & 1;
        let size_mode = (flag_byte >> 5) & 1;
        let addr_size = (flag_byte >> 3) & 0b11;
        let hops = flag_byte & 0b111;

        let age = bytes.get_u8();
        let addr = match size_mode {
            0 => addr_from_normal_mode(bytes, addr_size),
            1 => addr_from_micro_mode(bytes, addr_size),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unknown Size-Mode in network entry.",
                ))
            }
        };

        if info {
            let info = NetworkInfo::from_bytes([bytes.get_u8()]);
            Ok(NetworkEntry {
                dtn,
                active,
                size_mode,
                addr_size,
                hops,
                age,
                addr,
                info: Some(info),
            })
        } else {
            Ok(NetworkEntry {
                dtn,
                active,
                size_mode,
                addr_size,
                hops,
                age,
                addr,
                info: None,
            })
        }
    }

    pub fn from_bytes(bytes: &mut BytesMut, size: u8) -> Result<Self, std::io::Error> {
        if size == 0 {
            return Ok(Self {
                network_type: NetworkType::ACK,
                entries: vec![],
            });
        }
        let initial_size = bytes.remaining();
        let packet_type = bytes.get_u8();
        match packet_type {
            0 => {
                let mut entries: Vec<NetworkEntry> = Vec::new();
                while initial_size - bytes.remaining() < size as usize {
                    let entry = Self::parse_entry(bytes, false)?;
                    entries.push(entry);
                }
                Ok(Self {
                    network_type: NetworkType::Beacon,
                    entries,
                })
            }
            1 => Ok(Self {
                network_type: NetworkType::RequestBeacon,
                entries: vec![],
            }),
            5 => {
                let mut entries: Vec<NetworkEntry> = Vec::new();
                while initial_size - bytes.remaining() < size as usize {
                    let entry = Self::parse_entry(bytes, true)?;
                    entries.push(entry);
                }
                Ok(Self {
                    network_type: NetworkType::ExtendedBeacon,
                    entries,
                })
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NetworkEntry {
    pub dtn: u8,
    pub active: u8,
    pub size_mode: u8,
    pub addr_size: u8,
    pub hops: u8,
    pub age: u8,
    pub addr: (u32, u8),
    pub info: Option<NetworkInfo>,
}

#[bitfield(bits = 8)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NetworkInfo {
    pub packet_size: B3,
    pub speed: B3,
    pub dl_only: B1,
    pub halt: B1,
}
