use std::io::Error;

use bytes::{Buf, Bytes, BytesMut};

use crate::{
    header::{self, CompassHeader, CompassHeaderFlags},
    payload::{network::NetworkPayload, ServicePayload},
};

#[derive(Debug, PartialEq, Eq)]
struct ServicePacket {
    header: CompassHeader,
    payload: ServicePayload,
    sgn: u16,
    crc: u16,
}

impl ServicePacket {
    fn from_bytes(bytes: &mut BytesMut) -> Result<Self, std::io::Error> {
        let header = CompassHeader::from_raw(bytes)?;
        let payload: ServicePayload = match header.api {
            0 => ServicePayload::parse_network(bytes, header.payload_size),
            2 => todo!(),
            14 => todo!(),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Service not yet implemented!",
                ))
            }
        };
        let sgn = match header.header_flags.sgn() {
            1 => bytes.get_u16_le(),
            _ => 0,
        };
        let crc = match header.header_flags.crc() {
            1 => bytes.get_u16_le(),
            _ => 0,
        };

        Ok(Self {
            header,
            payload,
            sgn,
            crc,
        })
    }
}

pub trait Decoder {
    type Item;
    type Error: From<Error>;

    // Required method
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;
}

struct RawBytesDecoder {
    current_frame_len: usize,
}
impl Decoder for RawBytesDecoder {
    type Item = ServicePacket;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let escape = src.last();
        if escape == Some(&0xc0) {
            return Ok(Some(ServicePacket::from_bytes(src)?));
        } else {
            self.current_frame_len += src.len();
            src.reserve(self.current_frame_len + 4);
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::{
        header::network::RouteData,
        payload::network::{NetworkEntry, NetworkInfo},
    };

    use super::*;
    use bytes::{BufMut, BytesMut};
    #[test]
    fn test_decode_beacon() {
        // 21 9A A0 00 01 0F 00 00 1E 22 42 F0 8D 01 0E 00 70 00 1F 49 00 2B 02 01 4A 01 BC 01 01 88 11 c0
        let bytes: [u8; 32] = [
            0b00100001, // header byte 1
            0b10011010, // header byte 2
            0b10100000, // pid
            0b00000000, // pid  little endian
            0b00000001, // from system
            0b00001111, // from subsystem
            0b00000000, // to system
            0b00000000, // to subsystem
            0b00011110, // time begin (allways 6 bytes; not documented)
            0b00100010, 0b01000010, 0b11110000, 0b10001101, 0b00000001, // time end
            0b00001110, // size in bytes -> 14
            0b00000000, // payload begin network_type: 0 (beacon)
            // entry 1
            0b01110000, // flags
            0b00000000, // age
            0b00011111, // 1:15
            // entry 2
            0b01001001, // flags
            0b00000000, // age
            0b00101011, // system addr B1
            0b00000010, // system addr B2
            0b00000001, // subsystem addr
            // entry 3
            0b01001010, 0b00000001, 0b10111100, 0b00000001, 0b00000001,
            // payload end
            0b10001000, 0b00010001, 0b11000000,
        ];

        let mut buffer = BytesMut::with_capacity(4);
        let mut decoder = RawBytesDecoder {
            current_frame_len: 0,
        };
        for chunk in bytes.chunks(4) {
            buffer.put_slice(chunk);
            match decoder.decode(&mut buffer) {
                Ok(Some(packet)) => {}
                Ok(None) => println!("not enough bytes for packet"),
                Err(e) => println!("{:?}", e),
            };
        }
    }

    #[test]
    fn test_decode_extended_beacon() {
        // 21 9A A7 07 01 0F 00 00 EF 8C D7 32 8E 01 05 05 70 00 1F BF E4 F3

        let bytes: [u8; 23] = [
            0b00100001, // header byte 1
            0b10011010, // header byte 2
            0b10100111, 0b00000111, // pid
            0b00000001, 0b00001111, // from
            0b00000000, 0b00000000, // to
            0b11101111, 0b10001100, 0b11010111, 0b00110010, 0b10001110, 0b00000001, // time
            0b00000101, // payload size -> 5 bytes
            0b00000101, 0b01110000, 0b00000000, 0b00011111, 0b10111111, 0b11100100, 0b11110011,
            0b11000000,
        ];

        let mut buffer = BytesMut::with_capacity(4);
        let mut decoder = RawBytesDecoder {
            current_frame_len: 0,
        };
        for chunk in bytes.chunks(4) {
            buffer.put_slice(chunk);
            match decoder.decode(&mut buffer) {
                Ok(Some(packet)) => {
                    let mut header_flags = CompassHeaderFlags::default();
                    header_flags.set_pidset(1);
                    header_flags.set_sizelen0(1);
                    header_flags.set_crc(1);
                    header_flags.set_rsv(2);
                    header_flags.set_time(1);
                    header_flags.set_halt1(1);

                    let mut network_info = NetworkInfo::default();
                    network_info.set_packet_size(7);
                    network_info.set_speed(7);
                    network_info.set_dl_only(0);
                    network_info.set_halt(1);

                    assert_eq!(
                        packet,
                        ServicePacket {
                            header: CompassHeader {
                                header_flags: header_flags,
                                pid: 1959,
                                route: RouteData {
                                    entries: vec![(1, 15), (0, 0),]
                                },
                                api: 0,
                                time: 1710249970927,
                                payload_size: 5,
                            },
                            payload: ServicePayload::Network(NetworkPayload {
                                network_type: crate::payload::network::NetworkType::ExtendedBeacon,
                                entries: vec![NetworkEntry {
                                    dtn: 0,
                                    active: 1,
                                    size_mode: 1,
                                    addr_size: 2,
                                    hops: 0,
                                    age: 0,
                                    addr: (1, 15),
                                    info: Some(network_info),
                                }]
                            }),
                            sgn: 0,
                            crc: 62436,
                        }
                    )
                }
                Ok(None) => println!("not enough bytes for packet"),
                Err(e) => println!("{:?}", e),
            };
        }
    }
}
