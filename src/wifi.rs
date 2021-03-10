use eui48::MacAddress;

macro_rules! mac {
    ($bytes:expr => $start:expr) => {
        MacAddress::new([
            $bytes[$start],
            $bytes[$start + 1],
            $bytes[$start + 2],
            $bytes[$start + 3],
            $bytes[$start + 4],
            $bytes[$start + 5]
        ])
    };
}
macro_rules! u64 {
    (le[$bytes:expr => $start:expr]) => {
        u64::from_le_bytes([
            $bytes[$start],
            $bytes[$start + 1],
            $bytes[$start + 2],
            $bytes[$start + 3],
            $bytes[$start + 4],
            $bytes[$start + 5],
            $bytes[$start + 6],
            $bytes[$start + 7]
        ])
    };
}

#[derive(Debug)]
pub enum FrameType {
    Control(ControlFrame),
    Management(ManagementFrame),
    Data(DataFrame),
    Extension(ExtensionFrame)
}
impl FrameType {
    fn new(ty: u8, subty: u8, flags: u8, address1: MacAddress, frame: &[u8]) -> Result<Self> {
        match (ty, subty) {
            (0, 8) => ManagementFrame::beacon(frame, address1),
            (1, 13) => ControlFrame::ack(),
            (2, 0) => DataFrame::data(frame, flags, address1),
            _ => Err(Error::UnrecognisedFrameType)
        }
    }
}

pub struct Frame<'a> {
    pub frame_type: FrameType,
    pub duration: u16,
    pub body: &'a [u8],
    pub fcs: u32
}
impl<'a> Frame<'a> {
    pub fn new(mut frame: &'a [u8]) -> Result<Self> {
        if frame.len() < 10 {
            return Err(Error::UnexpectedEof)
        }
        let frame_control = frame[0];
        let flags = frame[1];
        let duration = u16::from_le_bytes([frame[2], frame[3]]);
        let address1 = mac!(frame => 4);

        let version = frame_control & 0b11;
        if version != 0 {
            return Err(Error::InvalidVersion(version));
        }
        let frame_type = FrameType::new(
            (frame_control >> 2) & 0b11, 
            (frame_control >> 4) & 0b1111,
            flags,
            address1,
            &mut frame
        )?;
        let fcs = u32::from_le_bytes({
            let b = &frame[frame.len() - 4..];
            [b[0], b[1], b[2], b[3]]
        });
        let body = &frame[..frame.len() - 4];
        Ok(Self {
            frame_type,
            duration,
            body,
            fcs
        })
    }
}

#[derive(Debug)]
pub enum ControlFrame {
    Ack
}
impl ControlFrame {
    fn ack() -> Result<FrameType> {
        Ok(FrameType::Control(Self::Ack))
    }
}

#[derive(Debug)]
pub enum ManagementTag {
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
impl ManagementTag {
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
pub enum ManagementFields {
    Beacon {
        timestamp: u64,
        ssid: String,
        supported_rates: Vec<u8>,
        tags: Vec<ManagementTag>
    }
}
#[derive(Debug)]
pub struct ManagementFrame {
    pub receiver: MacAddress,
    pub transmitter: MacAddress,
    pub bssid: MacAddress,
    pub sequence_control: u16,
    pub fields: ManagementFields
}
impl ManagementFrame {
    fn new(frame: &[u8], receiver: MacAddress, fields: ManagementFields) -> Self {
        Self {
            receiver,
            transmitter: mac!(frame => 10),
            bssid: mac!(frame => 16),
            sequence_control: u16::from_le_bytes([frame[22], frame[23]]),
            fields
        }
    }
    fn beacon(frame: &[u8], receiver: MacAddress) -> Result<FrameType> {
        if frame.len() < 40 {
            Err(Error::UnexpectedEof)
        } else {
            let data = &frame[24..frame.len() - 4];
            let tags = ManagementTag::parse_all(&data[12..])?;
            let fields = ManagementFields::Beacon {
                timestamp: u64!(le[data => 0]),
                ssid: tags.iter().find_map(|t| if let ManagementTag::Ssid(ssid) = t { Some(ssid.clone()) } else { None }).ok_or(Error::MissingTag("SSID"))?,
                supported_rates: tags.iter().find_map(|t| if let ManagementTag::SupportedRates(rates) = t { Some(rates.clone()) } else { None }).ok_or(Error::MissingTag("Supported Rates"))?,
                tags
            };
            Ok(FrameType::Management(Self::new(frame, receiver, fields)))
        }
    }
}

#[derive(Debug)]
pub struct DataFrame {
    pub receiver: MacAddress,
    pub transmitter: MacAddress,
    pub destination: MacAddress,
    pub source: MacAddress,
    pub bssid: Option<MacAddress>,
    pub sequence_control: u16
}
impl DataFrame {
    fn data(frame: &[u8], flags: u8, receiver: MacAddress) -> Result<FrameType> {
        let to_ds = flags & 0b1 != 0;
        let from_ds = flags & 0b10 != 0;

        if to_ds && from_ds && frame.len() < 40 {
            Err(Error::UnexpectedEof)
        } else if frame.len() < 34 {
            Err(Error::UnexpectedEof)
        } else {
            let transmitter = mac!(frame => 10);
            let address3 = mac!(frame => 16);
            let (destination, source, bssid) = match (to_ds, from_ds) {
                (false, false) => (receiver, transmitter, Some(address3)),
                (false, true) => (receiver, address3, Some(transmitter)),
                (true, false) => (address3, transmitter, Some(receiver)),
                (true, true) => (address3, mac!(frame => 24), None)
            };
            let sequence_control = u16::from_le_bytes([frame[22], frame[23]]);
            Ok(FrameType::Data(Self {
                receiver,
                transmitter,
                destination,
                source,
                bssid,
                sequence_control
            }))
        }
    }
}

#[derive(Debug)]
pub enum ExtensionFrame {
}
impl ExtensionFrame {
}

type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InvalidVersion(u8),
    UnrecognisedFrameType,
    MissingTag(&'static str),
}
impl From<eui48::ParseError> for Error {
    fn from(_: eui48::ParseError) -> Self {
        Self::UnexpectedEof
    }
}