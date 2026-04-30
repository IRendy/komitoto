//! SSDV decoder — port of C's `ssdv_dec_*` functions
//!
//! Decodes SSDV packets back into a JPEG image, with support for
//! packet gap filling and missing MCU zero-padding.

use crate::callsign::decode_callsign;
use crate::error::SsdvError;
use crate::jpeg::{STD_DHT00, STD_DHT01, STD_DHT10, STD_DHT11, STD_DQT0, STD_DQT1};
use crate::packet::{PacketType, PKT_SIZE, PKT_SIZE_HEADER};
use crate::process::{ProcessResult, ProcessorMode, ProcessorState, SsdvProcessor};

pub struct SsdvDecoder {
    s: SsdvProcessor,
}

impl SsdvDecoder {
    /// Initialise the decoder.
    /// `pkt_size` is the expected packet size (typically 256).
    pub fn new(pkt_size: usize) -> Result<Self, SsdvError> {
        if pkt_size > PKT_SIZE {
            return Err(SsdvError::DecodingError("Invalid packet size".into()));
        }

        let mut s = SsdvProcessor::new(ProcessorMode::Decoding);
        s.pkt_size = pkt_size;
        s.state = ProcessorState::Huff;
        s.packet_mcu_id = 0xFFFF;
        s.packet_mcu_offset = 0xFF;

        // Source DHT tables (for decoding incoming Huffman data)
        s.sdht[0][0] = s.stblcpy(&STD_DHT00);
        s.sdht[0][1] = s.stblcpy(&STD_DHT01);
        s.sdht[1][0] = s.stblcpy(&STD_DHT10);
        s.sdht[1][1] = s.stblcpy(&STD_DHT11);

        // Destination DHT tables (for encoding output JPEG)
        s.ddht[0][0] = s.dtblcpy(&STD_DHT00);
        s.ddht[0][1] = s.dtblcpy(&STD_DHT01);
        s.ddht[1][0] = s.dtblcpy(&STD_DHT10);
        s.ddht[1][1] = s.dtblcpy(&STD_DHT11);

        // Pre-allocate a generous output buffer for the JPEG
        s.out = vec![0u8; 512 * 1024]; // 512 KB
        s.outp = 0;
        s.out_len = s.out.len();

        Ok(Self { s })
    }

    /// Feed a single SSDV packet into the decoder.
    /// Returns `Ok(true)` when the image is complete, `Ok(false)` if more
    /// packets are needed, or an error on failure.
    ///
    /// Port of C's `ssdv_dec_feed()`.
    pub fn feed(&mut self, packet: &[u8]) -> Result<bool, SsdvError> {
        let pkt_size = self.s.pkt_size;
        if packet.len() < pkt_size {
            return Err(SsdvError::DecodingError("Packet too short".into()));
        }

        let packet_id = ((packet[7] as u16) << 8) | packet[8] as u16;
        self.s.packet_mcu_offset = packet[12];
        self.s.packet_mcu_id = ((packet[13] as u16) << 8) | packet[14] as u16;

        if self.s.packet_mcu_id != 0xFFFF {
            self.s.next_reset_mcu = self.s.packet_mcu_id as u32;
        }

        // First packet: write JPEG headers
        if self.s.packet_id == 0 {
            self.s.type_ = if packet[1] == 0x66 + PacketType::Nofec as u8 {
                PacketType::Nofec
            } else {
                PacketType::Normal
            };
            self.s.callsign = ((packet[2] as u32) << 24)
                | ((packet[3] as u32) << 16)
                | ((packet[4] as u32) << 8)
                | packet[5] as u32;
            self.s.image_id = packet[6];
            self.s.width = (packet[9] as u16) << 4;
            self.s.height = (packet[10] as u16) << 4;
            self.s.mcu_count = (packet[9] as u16) * (packet[10] as u16);
            self.s.quality = ((packet[11] >> 3) & 7) ^ 4;
            self.s.mcu_mode = packet[11] & 0x03;

            self.s.ssdv_set_packet_conf();

            // Generate DQT tables
            self.s.sdqt[0] = self.s.sload_standard_dqt(&STD_DQT0, self.s.quality);
            self.s.sdqt[1] = self.s.sload_standard_dqt(&STD_DQT1, self.s.quality);
            self.s.ddqt[0] = self.s.dload_standard_dqt(&STD_DQT0, self.s.quality);
            self.s.ddqt[1] = self.s.dload_standard_dqt(&STD_DQT1, self.s.quality);

            match self.s.mcu_mode & 3 {
                0 => self.s.ycparts = 4,
                1 => {
                    self.s.ycparts = 2;
                    self.s.mcu_count *= 2;
                }
                2 => {
                    self.s.ycparts = 2;
                    self.s.mcu_count *= 2;
                }
                3 => {
                    self.s.ycparts = 1;
                    self.s.mcu_count *= 4;
                }
                _ => {}
            }

            // Output JPEG headers and enable byte stuffing
            self.s.ssdv_out_headers();
            self.s.out_stuff = true;
        }

        // Packet gap detection
        if packet_id != self.s.packet_id {
            if packet_id < self.s.packet_id {
                return Err(SsdvError::DecodingError(format!(
                    "Packets out of order: expected {} got {}",
                    self.s.packet_id, packet_id
                )));
            }

            // Gap: one or more packets lost
            if self.s.packet_mcu_id == 0xFFFF {
                // This packet has no new MCU, skip it
                return Ok(false);
            }

            // Fill the gap left by the missing packet(s)
            self.s.ssdv_fill_gap(self.s.packet_mcu_id);

            let skip_bytes = self.s.packet_mcu_offset as usize;

            // Reset the JPEG decoder state
            self.s.state = ProcessorState::Huff;
            self.s.component = 0;
            self.s.mcupart = 0;
            self.s.acpart = 0;
            self.s.accrle = 0;

            self.s.packet_id = packet_id;

            // Feed the packet payload, skipping the lost MCU bytes
            self.feed_payload(packet, skip_bytes)
        } else {
            // Normal: feed the entire payload
            self.feed_payload(packet, 0)
        }
    }

    /// Feed packet payload bytes into the processor.
    /// `skip` is the number of initial payload bytes to skip (for gap recovery).
    fn feed_payload(&mut self, packet: &[u8], skip: usize) -> Result<bool, SsdvError> {
        for i in skip..self.s.pkt_size_payload {
            if i == self.s.packet_mcu_offset as usize {
                // The first MCU in a packet is byte aligned — drop old bits
                self.s.workbits = 0;
                self.s.worklen = 0;

                // Verify MCU ID matches
                if self.s.mcu_id != self.s.packet_mcu_id {
                    return Ok(false);
                }
            }

            let b = packet[PKT_SIZE_HEADER + i];
            self.s.workbits = (self.s.workbits << 8) | b as u32;
            self.s.worklen += 8;

            // Process until more bits needed or error
            let mut r = ProcessResult::Ok;
            while r == ProcessResult::Ok {
                r = self.s.ssdv_process();
            }

            match r {
                ProcessResult::BufferFull => {
                    // Grow output buffer
                    let current_pos = self.s.outp;
                    let extra = 256 * 1024; // 256 KB
                    self.s.out.resize(self.s.out.len() + extra, 0);
                    self.s.outp = current_pos;
                    self.s.out_len = extra;
                    // Flush remaining bits
                    self.s.ssdv_outbits(0, 0);
                }
                ProcessResult::Eoi => {
                    self.s.packet_id += 1;
                    return Ok(true);
                }
                ProcessResult::FeedMe => {}
                ProcessResult::Error => {
                    return Err(SsdvError::DecodingError("ssdv_process error".into()));
                }
                _ => {}
            }
        }

        self.s.packet_id += 1;
        Ok(false)
    }

    /// Finalize and return the decoded JPEG data.
    /// Call this after `feed()` returns `Ok(true)`, or after all packets
    /// have been fed (the image will be padded with zero MCUs for any gaps).
    ///
    /// Port of C's `ssdv_dec_get_jpeg()`.
    pub fn get_jpeg(&mut self) -> Vec<u8> {
        // Fill any remaining MCU gaps
        if self.s.mcu_id < self.s.mcu_count {
            self.s.ssdv_fill_gap(self.s.mcu_count);
        }

        // Sync remaining bits and write EOI
        self.s.ssdv_outbits_sync();
        self.s.out_stuff = false;
        self.s.ssdv_write_marker(0xFFD9, &[]); // J_EOI

        // Return only the bytes we've written
        let len = self.s.outp.min(self.s.out.len());
        self.s.out[..len].to_vec()
    }

    /// Get image information decoded from the first packet.
    pub fn info(&self) -> DecoderInfo {
        DecoderInfo {
            callsign: decode_callsign(self.s.callsign),
            callsign_code: self.s.callsign,
            image_id: self.s.image_id,
            width: self.s.width,
            height: self.s.height,
            mcu_count: self.s.mcu_count,
            mcu_mode: self.s.mcu_mode,
            quality: self.s.quality,
            packet_type: self.s.type_,
        }
    }
}

/// Image information extracted from the first SSDV packet.
#[derive(Debug, Clone)]
pub struct DecoderInfo {
    pub callsign: String,
    pub callsign_code: u32,
    pub image_id: u8,
    pub width: u16,
    pub height: u16,
    pub mcu_count: u16,
    pub mcu_mode: u8,
    pub quality: u8,
    pub packet_type: PacketType,
}
