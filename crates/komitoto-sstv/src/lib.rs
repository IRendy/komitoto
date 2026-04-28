//! SSTV (Slow-Scan Television) encoding and decoding for HAM radio.
//!
//! Converts images to SSTV audio waveforms and vice versa, supporting 14 modes
//! across Martin, Scottie, Robot, PD, and AVT families.
//!
//! # Supported Modes
//!
//! - **RGB modes**: Martin M1/M2, Scottie S1/S2, AVT 90
//! - **YUV modes**: Robot 36/72, PD 50/90/120/160/180/240/290
//!
//! # Example: Encode an image to SSTV audio
//!
//! ```no_run
//! use komitoto_sstv::{SstvEncoder, SstvMode, image_proc::ResizeStrategy};
//!
//! let encoder = SstvEncoder::new(SstvMode::MartinM1);
//! encoder.encode_to_wav("image.png", "sstv_output.wav", ResizeStrategy::Fit)?;
//! # Ok::<_, komitoto_sstv::SstvError>(())
//! ```
//!
//! # Example: Decode SSTV audio to an image
//!
//! ```no_run
//! use komitoto_sstv::{SstvDecoder, SstvMode};
//!
//! let decoder = SstvDecoder::new(SstvMode::MartinM1);
//! decoder.decode_to_file("sstv_input.wav", "decoded.png")?;
//! # Ok::<_, komitoto_sstv::SstvError>(())
//! ```

pub mod error;
pub mod mode;
pub mod spec;
pub mod dsp;
pub mod image_proc;
pub mod audio;
pub mod encoder;
pub mod decoder;
pub mod colorspace;

pub use error::SstvError;
pub use mode::SstvMode;
pub use encoder::SstvEncoder;
pub use decoder::SstvDecoder;
