pub trait FromBytes<In, Out> {
    fn from_bytes(bytes: In) -> Result<Out, std::io::Error>;
}

#[derive(Debug)]
pub struct CompassHeader {
    pub HALT0: u8,
    pub ROUTESET: u8,
    pub SIZELEN0: u8,
    pub ADTN: u8,
    pub DTN: u8,
    pub API: u8,
    pub REFSET: u8,
    pub PIDSET: u8,
    pub HALT1: u8,
    pub API16: u8,
    pub SIZELEN1: u8,
    pub TIME: u8,
    pub RSV: u8,
    pub CRC: u8,
    pub SGN: u8,
    pub HALT2: u8,
    pub NULL: u8,
    pub ERR: u8,
    pub URG: u8,
    pub ENC: u8,
    pub ZIP: u8,
}

pub const HEADER_FIELDS: [(&'static str, u8); 21] = [
    // 1st Byte (mandatory)
    // it is meant to be disabled. But then table 4.2 is wrong
    ("HALT0", 1),
    ("ROUTESET ", 1), // route fmt set
    ("SIZELEN0 ", 1), // in dombrovski doc this is numbered from 1 which is inconsistent with cs numbering schemes-> we start counting from 0
    ("ADTN ", 1),     // Answer packet must be DTN
    ("DTN ", 1),      // use DTN for delivery
    ("API ", 1),      // service ID is set
    ("REFSET", 1),    // Reference to other packet. No further doc found about this
    ("PIDSET", 1), // packet id value is set . makes little sense since it is marked as mandatory in doc....apparently for inter system communication
    // 2nd byte (optional)
    ("HALT1 ", 1), // if this is set, this is the last byte from header flag
    ("API16 ", 1), // if set API field is 2 bytes long instead of 1
    ("SIZELEN1 ", 1),
    ("TIME ", 1), // if set Time field is included
    ("RSV", 2),   // two bits reserved
    ("CRC ", 1),  // CRC active
    ("SGN ", 1), // signed (not to be interpreted as cryptographically signed, definitely too short for that)
    // 3rd byte (even more optional)
    ("HALT2 ", 1), // no sense to be defined if limit is 3 bytes but here we go
    ("NULL", 3),   // not defined but not reserved as well
    ("ERR ", 1),   // payload is service error message
    ("URG ", 1),   // packet is urgent
    ("ENC ", 1),   // packet is encrypted
    ("ZIP ", 1),   // packet is zipped
];

impl FromBytes<&[u8], CompassHeader> for CompassHeader {
    fn from_bytes(bytes: &[u8]) -> Result<CompassHeader, std::io::Error> {
        dbg!((bytes[0] >> 7) & CompassHeaderFields::HALT0);
        Ok(CompassHeader {
            first_byte: bytes[0],
            second_byte: bytes[1],
            third_byte: bytes[2],
        })
    }
}

#[derive(Debug)]
pub struct CompassPacket {
    header: CompassHeader,
}

impl FromBytes<&[u8], CompassPacket> for CompassPacket {
    fn from_bytes(bytes: &[u8]) -> Result<CompassPacket, std::io::Error> {
        let header = CompassHeader::from_bytes(&bytes)?;
        Ok(CompassPacket { header })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_packet() {
        let bytes = [0b10000001, 0b00000000, 0b00000000];
        let packet = CompassPacket::from_bytes(&bytes).unwrap();
        dbg!(packet);
    }
}
