//! SSDV packet constants, header parsing, and validation

use crate::callsign::decode_callsign;
use crate::error::SsdvError;
use crate::reed_solomon;

pub const PKT_SIZE: usize = 0x100;
pub const PKT_SIZE_HEADER: usize = 0x0F;
pub const PKT_SIZE_CRC: usize = 0x04;
pub const PKT_SIZE_RSCODES: usize = 0x20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    Normal = 0x00,
    Nofec = 0x01,
}

#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub packet_type: PacketType,
    pub callsign: u32,
    pub callsign_s: String,
    pub image_id: u8,
    pub packet_id: u16,
    pub width: u16,
    pub height: u16,
    pub eoi: u8,
    pub quality: u8,
    pub mcu_mode: u8,
    pub mcu_offset: u8,
    pub mcu_id: u16,
    pub mcu_count: u16,
}

/// Read packet header info from a raw packet buffer.
pub fn read_header(packet: &[u8]) -> PacketInfo {
    let ptype = if packet[1] == 0x66 + PacketType::Normal as u8 {
        PacketType::Normal
    } else {
        PacketType::Nofec
    };
    let callsign = ((packet[2] as u32) << 24)
        | ((packet[3] as u32) << 16)
        | ((packet[4] as u32) << 8)
        | (packet[5] as u32);
    let callsign_s = decode_callsign(callsign);
    let image_id = packet[6];
    let packet_id = ((packet[7] as u16) << 8) | (packet[8] as u16);
    let width = (packet[9] as u16) << 4;
    let height = (packet[10] as u16) << 4;
    let eoi = (packet[11] >> 2) & 1;
    let quality = ((packet[11] >> 3) & 7) ^ 4;
    let mcu_mode = packet[11] & 0x03;
    let mcu_offset = packet[12];
    let mcu_id = ((packet[13] as u16) << 8) | (packet[14] as u16);
    let mut mcu_count = (packet[9] as u16) * (packet[10] as u16);
    if mcu_mode == 1 || mcu_mode == 2 {
        mcu_count *= 2;
    } else if mcu_mode == 3 {
        mcu_count *= 4;
    }

    PacketInfo {
        packet_type: ptype,
        callsign,
        callsign_s,
        image_id,
        packet_id,
        width,
        height,
        eoi,
        quality,
        mcu_mode,
        mcu_offset,
        mcu_id,
        mcu_count,
    }
}

/// Validate an SSDV packet. Returns Ok(errors_corrected) or Err if invalid.
/// The packet buffer is modified in-place if RS correction is applied.
pub fn validate_packet(packet: &mut [u8], pkt_size: usize) -> Result<i32, SsdvError> {
    if pkt_size > PKT_SIZE || pkt_size < PKT_SIZE_HEADER + PKT_SIZE_CRC + 2 {
        return Err(SsdvError::InvalidPacket("Invalid packet size".into()));
    }

    // Force sync byte
    packet[0] = 0x55;

    let mut ptype = None;
    let mut errors = 0i32;

    // Test NOFEC packet
    if packet[1] == 0x66 + PacketType::Nofec as u8 {
        let pkt_size_payload = pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC;
        let pkt_size_crcdata = PKT_SIZE_HEADER + pkt_size_payload - 1;
        let x = crc32fast::hash(&packet[1..=pkt_size_crcdata]);
        let i = 1 + pkt_size_crcdata;
        let stored_crc = ((packet[i] as u32) << 24)
            | ((packet[i + 1] as u32) << 16)
            | ((packet[i + 2] as u32) << 8)
            | (packet[i + 3] as u32);
        if x == stored_crc {
            ptype = Some(PacketType::Nofec);
        }
    }

    // Test NORMAL packet (without RS)
    if ptype.is_none() && packet[1] == 0x66 + PacketType::Normal as u8 {
        let pkt_size_payload = pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC - PKT_SIZE_RSCODES;
        let pkt_size_crcdata = PKT_SIZE_HEADER + pkt_size_payload - 1;
        let x = crc32fast::hash(&packet[1..=pkt_size_crcdata]);
        let i = 1 + pkt_size_crcdata;
        let stored_crc = ((packet[i] as u32) << 24)
            | ((packet[i + 1] as u32) << 16)
            | ((packet[i + 2] as u32) << 8)
            | (packet[i + 3] as u32);
        if x == stored_crc {
            ptype = Some(PacketType::Normal);
        }
    }

    // Try RS error correction for NORMAL packet
    if ptype.is_none() {
        let pkt_size_payload = pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC - PKT_SIZE_RSCODES;
        let pkt_size_crcdata = PKT_SIZE_HEADER + pkt_size_payload - 1;
        let pad = PKT_SIZE - pkt_size;

        packet[1] = 0x66 + PacketType::Normal as u8;
        let decoded = reed_solomon::decode_rs_8(&mut packet[1..], pad as i32);
        if decoded < 0 {
            return Err(SsdvError::InvalidPacket("RS decode failed".into()));
        }
        errors = decoded;

        let x = crc32fast::hash(&packet[1..=pkt_size_crcdata]);
        let i = 1 + pkt_size_crcdata;
        let stored_crc = ((packet[i] as u32) << 24)
            | ((packet[i + 1] as u32) << 16)
            | ((packet[i + 2] as u32) << 8)
            | (packet[i + 3] as u32);
        if x == stored_crc {
            ptype = Some(PacketType::Normal);
        }
    }

    let ptype = match ptype {
        Some(pt) => pt,
        None => return Err(SsdvError::InvalidPacket("All validation attempts failed".into())),
    };

    // Sanity checks
    let info = read_header(packet);
    if info.packet_type != ptype {
        return Err(SsdvError::InvalidPacket("Type mismatch after validation".into()));
    }
    if info.width == 0 || info.height == 0 {
        return Err(SsdvError::InvalidPacket("Zero dimensions".into()));
    }
    let pkt_size_payload = match ptype {
        PacketType::Normal => pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC - PKT_SIZE_RSCODES,
        PacketType::Nofec => pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC,
    };
    if info.mcu_id != 0xFFFF {
        if info.mcu_id >= info.mcu_count {
            return Err(SsdvError::InvalidPacket("MCU ID >= MCU count".into()));
        }
        if info.mcu_offset as usize >= pkt_size_payload {
            return Err(SsdvError::InvalidPacket("MCU offset >= payload size".into()));
        }
    }

    Ok(errors)
}
