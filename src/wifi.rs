use pcap::Packet;

#[derive(Debug)]
pub struct Mac([u8; 6]);
impl Mac {
    pub fn new(d: &[u8]) -> Self {
        assert_eq!(d.len(), 6);
        Self([d[0], d[1], d[2], d[3], d[4], d[5]])
    }
}

#[derive(Debug)]
pub enum FrameType {
    AssociationRequest,
    AssociationResponse,
    ReassociationRequest,
    ReassociationResponse,
    Beacon
}
impl FrameType {
    fn new(ty: u16, subty: u16) -> Self {
        match (ty, subty) {
            (0, 0) => Self::AssociationRequest,
            (0, 1) => Self::AssociationResponse,
            (0, 2) => Self::ReassociationRequest,
            (0, 3) => Self::ReassociationResponse,
            (0, 8) => Self::Beacon,
            _ => unimplemented!()
        }
    }
}

#[derive(Debug)]
pub enum Frame {
    Beacon {
        destination: Mac,
        source: Mac,
        ssid: String
    }
}
impl Frame {
    pub fn parse(packet: Packet) -> Result<Self> {
        if packet.data.len() < 32 {
            panic!("Frame too small. 802.11 frames must be >= 32 bytes in length")
        }
        let frame_control = u16::from_be_bytes([packet.data[0], packet.data[1]]);
        let version = frame_control & 0b11;
        if version != 0 {
            return Err(Error::InvalidVersion(version));
        }
        let frame_type = FrameType::new((frame_control >> 2) & 0b11, (frame_control >> 4) & 0b1111);

        match frame_type {
            FrameType::Beacon => Self::beacon(packet.data),
            _ => Err(Error::UnrecognisedFrameType)
        }
    }

    pub fn beacon(bytes: &[u8]) -> Result<Self> {
        Ok(Self::Beacon {
            destination: Mac::new(&bytes[4..10]),
            source: Mac::new(&bytes[10..16]),
            ssid: String::from("apples")
        })
    }
}

type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    InvalidVersion(u16),
    UnrecognisedFrameType
}