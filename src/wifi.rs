use std::borrow::Cow;

use oui::{OuiDatabase, OuiEntry};
use eui48::MacAddress;
use pcap::Packet;

#[derive(Debug)]
pub enum Tag {
    Ssid(String),
    SupportedRates(Vec<u8>),
    Country {
        code: [u8; 2],
    },
    VendorSpecific {
        vendor: [u8; 3],
    },
    Unknown
}
impl Tag {
    /// Parse a single management tag, removing itself from the start of the given buffer
    pub fn parse(data: &mut &[u8]) -> Result<Self> {
        let &length = data.get(1).ok_or(Error::UnexpectedEof)?;
        let tag = data[0];
        let (d, other) = data.split_at(2 + length as usize);
        *data = other;
        let data = &d[2..];
        Ok(match tag {
            0x00 => Self::Ssid(String::from_utf8_lossy(data).to_string()),
            0x01 => Self::SupportedRates(data.to_vec()),
            0x07 => Self::Country {
                code: [data[0], data[1]]
            },
            0xdd => Self::VendorSpecific {
                vendor: [data[0], data[1], data[2]]
            },
            _ => Self::Unknown
        })
    }
    /// Parse all of the management tags inside of a given buffer
    pub fn parse_all(mut data: &[u8]) -> Result<Vec<Self>> {
        let mut tags = vec![];
        while !data.is_empty() {
            tags.push(Self::parse(&mut data)?)
        }
        Ok(tags)
    }
}

#[derive(Debug)]
pub enum FrameType {
    AssociationRequest,
    AssociationResponse,
    ReassociationRequest,
    ReassociationResponse,
    Beacon,
    Ack,
    Reserved,
    Unknown
}
impl FrameType {
    fn new(ty: u8, subty: u8) -> Self {
        match (ty, subty) {
            (0, 0) => Self::AssociationRequest,
            (0, 1) => Self::AssociationResponse,
            (0, 2) => Self::ReassociationRequest,
            (0, 3) => Self::ReassociationResponse,
            (0, 8) => Self::Beacon,
            (1, 13) => Self::Ack,
            (2, 13) => Self::Reserved,
            _ => Self::Unknown
        }
    }
}

#[derive(Debug)]
pub enum Frame {
    Beacon {
        destination: MacAddress,
        source: MacAddress,
        bssid: MacAddress,
        ssid: String,
        tags: Vec<Tag>
    },
    Ack {
        reciever: MacAddress,
    },
    Unknown
}
impl Frame {
    pub fn parse(packet: &[u8]) -> Result<Self> {
        if packet.len() < 10 {
            return Err(Error::UnexpectedEof)
        }
        let frame_control = packet[0];
        let flags = packet[1];
        let duration = u16::from_le_bytes([packet[2], packet[3]]);
        let address1 = MacAddress::from_bytes(&packet[4..10])?;

        let version = frame_control & 0b11;
        if version != 0 {
            return Err(Error::InvalidVersion(version));
        }
        let frame_type = FrameType::new((frame_control >> 2) & 0b11, (frame_control >> 4) & 0b1111);

        match frame_type {
            FrameType::Beacon => Self::beacon(address1, MacAddress::from_bytes(&packet[10..16])?, MacAddress::from_bytes(&packet[16..22])?, u16::from_le_bytes([packet[22], packet[23]]), &packet[24..]),
            FrameType::Ack => Ok(Self::Ack { reciever: address1 }),
            _ => Ok(Self::Unknown)
        }
    }

    pub fn beacon(destination: MacAddress, source: MacAddress, bssid: MacAddress, sequence_control: u16, data: &[u8]) -> Result<Self> {
        let timestamp = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        let beacon_interval = u16::from_le_bytes([data[8], data[9]]);
        let capabilities = u16::from_le_bytes([data[10], data[11]]);
        let tags = Tag::parse_all(&data[12..data.len() - 4])?;
        let ssid = tags.iter().find_map(|tag| if let Tag::Ssid(ssid) = tag { Some(ssid.clone()) } else { None }).ok_or(Error::MissingTag("SSID"))?;
        Ok(Self::Beacon {
            destination,
            source,
            bssid,
            ssid,
            tags
        })
    }
}

type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InvalidVersion(u8),
    UnrecognisedFrameType,
    MissingTag(&'static str),
    InvalidMac(eui48::ParseError)
}
impl From<eui48::ParseError> for Error {
    fn from(error: eui48::ParseError) -> Self {
        Self::InvalidMac(error)
    }
}