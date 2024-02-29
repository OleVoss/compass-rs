use std::io::Error;

use crate::packet::CompassHeader;
use bytes::BytesMut;

#[derive(Debug)]
struct ServicePacket {
    // header: CompassHeader,
    // payload: ServicePayload,
}

enum ServicePayload {
    Unknown(Vec<u8>),
    Network(NetworkPayload),
    Model(ModelPayload),
    Command(CommandPayload),
}

struct NetworkPayload;
struct ModelPayload;
struct CommandPayload;

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
            return Ok(Some(ServicePacket {}));
        } else {
            self.current_frame_len += src.len();
            src.reserve(self.current_frame_len + 4);
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytes::{BufMut, BytesMut};
    #[test]
    fn test_decode_from_bytes() {
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
            0b00000000, // payload begin
            0b01110000, 0b00000000, 0b00011111, 0b01001001, 0b00000000, 0b00101011, 0b00000010,
            0b00000001, 0b01001010, 0b00000001, 0b10111100, 0b00000001,
            0b00000001, // payload end
            0b10001000, 0b00010001, 0b11000000,
        ];

        let mut buffer = BytesMut::with_capacity(4);
        let mut decoder = RawBytesDecoder {
            current_frame_len: 0,
        };
        for chunk in bytes.chunks(4) {
            buffer.put_slice(chunk);
            match decoder.decode(&mut buffer) {
                Ok(Some(packet)) => {
                    dbg!(packet);
                }
                Ok(None) => println!("not enough bytes for packet"),
                Err(e) => println!("{:?}", e),
            };
        }
    }
}
