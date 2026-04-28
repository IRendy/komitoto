use std::fmt;

/// Errors that can occur during SSTV encoding or decoding.
#[derive(Debug)]
pub enum SstvError {
    /// An error occurred while reading or writing an image.
    ImageError(image::ImageError),
    /// An I/O error occurred.
    IoError(std::io::Error),
    /// An error occurred while reading or writing a WAV file.
    WavError(hound::Error),
    /// An error occurred during SSTV encoding.
    EncodingError(String),
    /// An error occurred during SSTV decoding.
    DecodingError(String),
    /// The requested SSTV mode is not supported.
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
