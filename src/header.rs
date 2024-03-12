#![allow(dead_code)]
pub mod network;

use std::{alloc::System, default};

use bytes::{buf, Buf, BytesMut};
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B2, B3},
};

use self::network::{RouteData, RouteEntry};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CompassHeader {
    pub header_flags: CompassHeaderFlags,
    pub pid: u16,
    pub route: RouteData,
    pub api: u16,
    pub time: u64,
    pub payload_size: u32,
}

impl CompassHeader {
    pub fn from_raw(bytes: &mut BytesMut) -> Result<Self, std::io::Error> {
        match CompassHeaderFlags::from_bytes_buf(bytes) {
            Ok(flags) => {
                let pid: u16 = bytes.get_u16_le();

                let mut header = Self::default();
                header.header_flags = flags;
                header.pid = pid;

                // routing
                if header.header_flags.routeset() == 1 {
                    // TODO implement route fmt bit
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        "Routeformat not yet implemented!",
                    ));
                } else {
                    let mut route_data = RouteData::default();
                    for _ in ["source", "target"] {
                        let sys_address = bytes.get_u8();
                        let subsys_addres = bytes.get_u8();
                        route_data.add((sys_address as u16, subsys_addres as u16));
                    }
                    header.route = route_data;
                }

                // api
                if header.header_flags.api() == 1 {
                    let api: u16;
                    if header.header_flags.api16() == 1 {
                        api = bytes.get_u16_le();
                    } else {
                        api = bytes.get_u8() as u16;
                    }
                    header.api = api;
                }

                // time
                if header.header_flags.time() == 1 {
                    let time_bytes: [u8; 8] = [
                        bytes.get_u8(),
                        bytes.get_u8(),
                        bytes.get_u8(),
                        bytes.get_u8(),
                        bytes.get_u8(),
                        bytes.get_u8(),
                        0,
                        0,
                    ];
                    let timestamp = u64::from_le_bytes(time_bytes);
                    header.time = timestamp;
                }

                // payload size
                let size_flag =
                    (header.header_flags.sizelen0() << 1) | header.header_flags.sizelen1();
                header.payload_size = match size_flag {
                    0 => 0,
                    1 => bytes.get_u32_le(),
                    2 => bytes.get_u8() as u32,
                    3 => bytes.get_u16_le() as u32,
                    _ => 0,
                };
                return Ok(header);
            }
            Err(_) => todo!(),
        }
    }
}

#[bitfield(bits = 24)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CompassHeaderFlags {
    // byte 1
    pub pidset: B1,
    pub refset: B1, // bit 7
    pub api: B1,
    pub dtn: B1,
    pub adtn: B1,
    pub sizelen0: B1,
    pub routeset: B1,
    pub halt0: B1, // bit 0
    // byte 2
    pub sgn: B1,
    pub crc: B1,
    pub rsv: B2,
    pub time: B1,
    pub sizelen1: B1,
    pub api16: B1,
    pub halt1: B1,
    // byte 3
    pub null: B3,
    pub err: B1,
    pub urg: B1,
    pub enc: B1,
    pub zip: B1,
    pub halt2: B1,
}

impl CompassHeaderFlags {
    pub fn from_raw(bytes: &[u8]) -> Result<Self, std::io::Error> {
        if ((bytes[0] >> 7) & 1) == 1 {
            return Ok(Self::from_bytes([bytes[0], 0, 0]));
        } else if ((bytes[1] >> 7) & 1) == 1 {
            return Ok(Self::from_bytes([bytes[0], bytes[1], 0]));
        } else if ((bytes[2] >> 7) & 1) == 1 {
            return Ok(Self::from_bytes([bytes[0], bytes[1], bytes[2]]));
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Wrong header bytes.",
        ))
    }

    pub fn from_bytes_buf(bytes: &mut BytesMut) -> Result<Self, std::io::Error> {
        let mut header: Option<Self> = None;
        if ((bytes[0] >> 7) & 1) == 1 {
            header = Some(Self::from_bytes([bytes[0], 0, 0]));
            bytes.advance(1);
        } else if ((bytes[1] >> 7) & 1) == 1 {
            header = Some(Self::from_bytes([bytes[0], bytes[1], 0]));
            bytes.advance(2);
        } else if ((bytes[2] >> 7) & 1) == 1 {
            header = Some(Self::from_bytes([bytes[0], bytes[1], bytes[2]]));
            bytes.advance(3);
        }

        match header {
            Some(header) => Ok(header),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Wrong header bytes.",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, BufMut};

    use super::*;

    #[test]
    fn from_bits() {
        let bytes = [
            // header
            0b01010001, 0b10000000, 0b00000000,
        ];
        let mut buf = BytesMut::with_capacity(5);
        buf.put_slice(&bytes);

        let flags = CompassHeaderFlags::from_bytes_buf(&mut buf).unwrap();
        // TODO: add more test cases
        assert!(flags.halt1() == 1);
    }

    #[test]
    fn build_header() {
        let bytes = [
            0b00100101, 0b10010000, 0b00000001, 0b00000000, 0b00000001, 0b00001111, 0b11111111,
            0b00000101, 0b00000000, 0b00000001, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000011,
        ];

        let mut buf = BytesMut::with_capacity(5);
        buf.put_slice(&bytes);

        let header = CompassHeader::from_raw(&mut buf);
    }

    #[test]
    fn test_bytes() {
        let bytes = [
            0b01000001, 0b10000000, 0b00000001, 0b00000000, 0b00000001, 0b00000000, 0b00001111,
            0b00000000, 0b00100101, 0b00000001, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000,
        ];

        let mut buf = BytesMut::with_capacity(5);
        buf.put_slice(&bytes);
        buf.get_u16_le();
    }
}
