use std::fmt;

#[derive(Debug)]
pub enum SstvError {
    ImageError(image::ImageError),
    IoError(std::io::Error),
    WavError(hound::Error),
    EncodingError(String),
    DecodingError(String),
    UnsupportedMode(String),
}

impl fmt::Display for SstvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SstvError::ImageError(e) => write!(f, "image error: {e}"),
            SstvError::IoError(e) => write!(f, "I/O error: {e}"),
            SstvError::WavError(e) => write!(f, "WAV error: {e}"),
            SstvError::EncodingError(e) => write!(f, "encoding error: {e}"),
            SstvError::DecodingError(e) => write!(f, "decoding error: {e}"),
            SstvError::UnsupportedMode(e) => write!(f, "unsupported mode: {e}"),
        }
    }
}

impl std::error::Error for SstvError {}

impl From<image::ImageError> for SstvError {
    fn from(e: image::ImageError) -> Self {
        SstvError::ImageError(e)
    }
}

impl From<std::io::Error> for SstvError {
    fn from(e: std::io::Error) -> Self {
        SstvError::IoError(e)
    }
}

impl From<hound::Error> for SstvError {
    fn from(e: hound::Error) -> Self {
        SstvError::WavError(e)
    }
}
