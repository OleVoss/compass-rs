#[allow(non_snake_case)]
pub trait FromBytes<In, Out> {
    fn from_bytes(bytes: In) -> Result<Out, std::io::Error>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

impl FromBytes<&[u8], CompassHeader> for CompassHeader {
    fn from_bytes(bytes: &[u8]) -> Result<CompassHeader, std::io::Error> {
        let mut header = CompassHeader::default();

        header.ROUTESET = (bytes[0] >> 6) & 1;
        header.SIZELEN0 = (bytes[0] >> 5) & 1;
        header.ADTN = (bytes[0] >> 4) & 1;
        header.DTN = (bytes[0] >> 3) & 1;
        header.API = (bytes[0] >> 2) & 1;
        header.REFSET = (bytes[0] >> 1) & 1;
        header.PIDSET = bytes[0] & 1;

        if ((bytes[0] >> 7) & 1) == 1 {
            header.API16 = (bytes[1] >> 6) & 1;
            header.SIZELEN1 = (bytes[1] >> 5) & 1;
            header.TIME = (bytes[1] >> 4) & 1;
            header.RSV = (bytes[1] >> 3) & 0b11;
            header.CRC = (bytes[1] >> 1) & 1;
            header.SGN = bytes[1] & 1;
        }

        if ((bytes[1] >> 7) & 1) == 1 {
            header.NULL = (bytes[2] >> 6) & 1;
            header.ERR = (bytes[2] >> 4) & 0b111;
            header.URG = (bytes[2] >> 3) & 1;
            header.ENC = (bytes[2] >> 2) & 1;
            header.ZIP = bytes[2] & 1;
        }

        Ok(header)
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
        let bytes = [
            // header                           // payload
            0b10000000, 0b10000000, 0b10010001, 0b11111111, 0b11111111, 0b11111111,
        ];
        let packet = CompassPacket::from_bytes(&bytes).unwrap();
        dbg!(packet);
    }
}
