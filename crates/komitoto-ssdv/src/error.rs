use std::fmt;

#[derive(Debug)]
pub enum SsdvError {
    Io(std::io::Error),
    InvalidPacket(String),
    InvalidJpeg(String),
    InvalidCallsign(String),
    EncodingError(String),
    DecodingError(String),
}

impl fmt::Display for SsdvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsdvError::Io(e) => write!(f, "IO error: {}", e),
            SsdvError::InvalidPacket(s) => write!(f, "Invalid packet: {}", s),
            SsdvError::InvalidJpeg(s) => write!(f, "Invalid JPEG: {}", s),
            SsdvError::InvalidCallsign(s) => write!(f, "Invalid callsign: {}", s),
            SsdvError::EncodingError(s) => write!(f, "Encoding error: {}", s),
            SsdvError::DecodingError(s) => write!(f, "Decoding error: {}", s),
        }
    }
}

impl std::error::Error for SsdvError {}

impl From<std::io::Error> for SsdvError {
    fn from(e: std::io::Error) -> Self {
        SsdvError::Io(e)
    }
}
