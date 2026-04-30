//! komitoto-ssdv — Pure Rust SSDV (Slow Scan Digital Video) encoder/decoder
//!
//! SSDV is a digital image transmission protocol used in amateur radio,
//! particularly for high-altitude balloon (HAB) imagery. It uses JPEG
//! DCT coefficients with Reed-Solomon FEC for forward error correction.
//!
//! This crate is a faithful Rust port of Philip Heron's C implementation.

pub mod callsign;
pub mod decode;
pub mod encode;
pub mod error;
pub mod jpeg;
pub mod packet;
pub mod process;
pub mod reed_solomon;

// Public API re-exports
pub use callsign::{decode_callsign, encode_callsign, is_valid_ham_callsign, itu_prefix_info, validate_callsign, MAX_CALLSIGN};
pub use decode::{DecoderInfo, SsdvDecoder};
pub use encode::SsdvEncoder;
pub use error::SsdvError;
pub use packet::{PacketInfo, PacketType, PKT_SIZE, PKT_SIZE_CRC, PKT_SIZE_HEADER, PKT_SIZE_RSCODES};
