//! Core SSDV processor - faithful port of ssdv.c state machine
//!
//! This module contains the SsdvProcessor struct and all internal methods
//! for processing JPEG scan data through the SSDV encoding/decoding pipeline.
//! Ported from Philip Heron's ssdv.c (GPLv3+).

use crate::jpeg::{
    TBL_LEN, HBUFF_LEN, J_SOF0, J_SOF2, J_DHT, J_SOI, J_EOI, J_SOS, J_DQT, J_DRI, J_APP0,
    APP0, SOS, STD_DHT00, STD_DHT01, STD_DHT10, STD_DHT11,
    load_standard_dqt, irdiv, jpeg_int, jpeg_encode_int,
};
use crate::packet::{PKT_SIZE_CRC, PKT_SIZE_HEADER, PKT_SIZE_RSCODES, PacketType};

/// Processor state for the JPEG marker/Huffman state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorState {
    Marker,
    MarkerLen,
    MarkerData,
    Huff,
    Int,
    Eoi,
}

/// Processor mode: encoding (JPEG→SSDV) or decoding (SSDV→JPEG)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorMode {
    Encoding,
    Decoding,
}

/// Result of processing operations — mirrors C's SSDV_OK/FEED_ME/etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessResult {
    Ok,
    FeedMe,
    HavePacket,
    BufferFull,
    Eoi,
    Error,
}

/// SSDV processor — direct port of C's `ssdv_t` struct.
///
/// In C the struct uses raw pointers into fixed-size arrays for DHT/DQT
/// tables. Here we use `Vec<u8>` with `usize` offsets (wrapped in `Option`
/// to represent the C null-pointer sentinel).
pub struct SsdvProcessor {
    // Packet type configuration
    pub type_: PacketType,
    pub pkt_size_payload: usize,
    pub pkt_size_crcdata: usize,
    pub pkt_size: usize,

    // Image information
    pub width: u16,
    pub height: u16,
    pub callsign: u32,
    pub image_id: u8,
    pub packet_id: u16,
    pub mcu_mode: u8,
    pub mcu_id: u16,
    pub mcu_count: u16,
    pub quality: u8,
    pub packet_mcu_id: u16,
    pub packet_mcu_offset: u8,

    // Input buffer
    pub inp: Vec<u8>,
    pub inp_pos: usize,
    pub in_len: usize,
    pub in_skip: usize,

    // Source bits (bit buffer for incoming Huffman data)
    pub workbits: u32,
    pub worklen: u8,

    // Output buffer
    pub out: Vec<u8>,
    pub outp: usize,
    pub out_len: usize,
    pub out_stuff: bool,

    // Output bits
    pub outbits: u32,
    pub outlen: u8,

    // State machine
    pub state: ProcessorState,
    pub marker: u16,
    pub marker_len: u16,
    pub marker_data_start: usize,
    pub marker_data_len: usize,

    // JPEG decoder/encoder state
    pub greyscale: bool,
    pub component: u8,
    pub ycparts: u8,
    pub mcupart: u8,
    pub acpart: u8,
    pub dc: [i32; 3],
    pub adc: [i32; 3],
    pub acrle: u8,
    pub accrle: u8,
    pub dri: u16,
    pub mode: ProcessorMode,
    pub reset_mcu: u32,
    pub next_reset_mcu: u32,
    pub needbits: u8,

    // Source (input) Huffman and quantisation tables
    pub stbls: Vec<u8>,
    pub sdht: [[Option<usize>; 2]; 2],
    pub sdqt: [Option<usize>; 2],
    pub stbl_len: usize,

    // Destination (output) Huffman and quantisation tables
    pub dtbls: Vec<u8>,
    pub ddht: [[Option<usize>; 2]; 2],
    pub ddqt: [Option<usize>; 2],
    pub dtbl_len: usize,
}

impl SsdvProcessor {
    /// Create a new processor in the given mode, with zeroed state.
    pub fn new(mode: ProcessorMode) -> Self {
        Self {
            type_: PacketType::Normal,
            pkt_size_payload: 0,
            pkt_size_crcdata: 0,
            pkt_size: 0,

            width: 0,
            height: 0,
            callsign: 0,
            image_id: 0,
            packet_id: 0,
            mcu_mode: 0,
            mcu_id: 0,
            mcu_count: 0,
            quality: 4,
            packet_mcu_id: 0,
            packet_mcu_offset: 0,

            inp: Vec::new(),
            inp_pos: 0,
            in_len: 0,
            in_skip: 0,

            workbits: 0,
            worklen: 0,

            out: Vec::new(),
            outp: 0,
            out_len: 0,
            out_stuff: false,

            outbits: 0,
            outlen: 0,

            state: ProcessorState::Marker,
            marker: 0,
            marker_len: 0,
            marker_data_start: 0,
            marker_data_len: 0,

            greyscale: false,
            component: 0,
            ycparts: 0,
            mcupart: 0,
            acpart: 0,
            dc: [0; 3],
            adc: [0; 3],
            acrle: 0,
            accrle: 0,
            dri: 0,
            mode,
            reset_mcu: 0,
            next_reset_mcu: 0,
            needbits: 0,

            stbls: Vec::with_capacity(TBL_LEN + HBUFF_LEN),
            sdht: [[None; 2]; 2],
            sdqt: [None; 2],
            stbl_len: 0,

            dtbls: Vec::with_capacity(TBL_LEN),
            ddht: [[None; 2]; 2],
            ddqt: [None; 2],
            dtbl_len: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Packet configuration
    // -----------------------------------------------------------------------

    /// Configure payload size and CRC position based on packet type.
    /// Port of C's `ssdv_set_packet_conf()`.
    pub fn ssdv_set_packet_conf(&mut self) {
        match self.type_ {
            PacketType::Normal => {
                self.pkt_size_payload =
                    self.pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC - PKT_SIZE_RSCODES;
                self.pkt_size_crcdata = PKT_SIZE_HEADER + self.pkt_size_payload - 1;
            }
            PacketType::Nofec => {
                self.pkt_size_payload = self.pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC;
                self.pkt_size_crcdata = PKT_SIZE_HEADER + self.pkt_size_payload - 1;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Table management — C used raw pointers; we use Vec + usize offsets
    // -----------------------------------------------------------------------

    /// Load a standard DQT table into the *source* tables area.
    /// Returns the offset into `stbls`, or `None` if out of space.
    /// Port of C's `sload_standard_dqt()`.
    pub fn sload_standard_dqt(&mut self, table: &[u8], quality: u8) -> Option<usize> {
        if self.stbl_len + 65 > TBL_LEN + HBUFF_LEN {
            return None;
        }
        let offset = self.stbl_len;
        // Ensure space
        while self.stbls.len() < offset + 65 {
            self.stbls.push(0);
        }
        load_standard_dqt(&mut self.stbls[offset..offset + 65], table, quality);
        self.stbl_len += 65;
        Some(offset)
    }

    /// Load a standard DQT table into the *destination* tables area.
    /// Returns the offset into `dtbls`, or `None` if out of space.
    /// Port of C's `dload_standard_dqt()`.
    pub fn dload_standard_dqt(&mut self, table: &[u8], quality: u8) -> Option<usize> {
        if self.dtbl_len + 65 > TBL_LEN {
            return None;
        }
        let offset = self.dtbl_len;
        while self.dtbls.len() < offset + 65 {
            self.dtbls.push(0);
        }
        load_standard_dqt(&mut self.dtbls[offset..offset + 65], table, quality);
        self.dtbl_len += 65;
        Some(offset)
    }

    /// Copy raw bytes into the *source* tables area.
    /// Port of C's `stblcpy()`.
    pub fn stblcpy(&mut self, src: &[u8]) -> Option<usize> {
        let n = src.len();
        if self.stbl_len + n > TBL_LEN + HBUFF_LEN {
            return None;
        }
        let offset = self.stbl_len;
        // Ensure space
        while self.stbls.len() < offset + n {
            self.stbls.push(0);
        }
        self.stbls[offset..offset + n].copy_from_slice(src);
        self.stbl_len += n;
        Some(offset)
    }

    /// Copy raw bytes into the *destination* tables area.
    /// Port of C's `dtblcpy()`.
    pub fn dtblcpy(&mut self, src: &[u8]) -> Option<usize> {
        let n = src.len();
        if self.dtbl_len + n > TBL_LEN {
            return None;
        }
        let offset = self.dtbl_len;
        while self.dtbls.len() < offset + n {
            self.dtbls.push(0);
        }
        self.dtbls[offset..offset + n].copy_from_slice(src);
        self.dtbl_len += n;
        Some(offset)
    }

    // -----------------------------------------------------------------------
    // DHT / DQT accessors — C used macros SDHT, DDHT, SDQT, DDQT
    // -----------------------------------------------------------------------

    /// Return the offset of the current *source* DHT table.
    /// `acpart > 0` selects the AC table, `component > 0` selects chroma.
    #[inline]
    fn sdht_offset(&self) -> Option<usize> {
        let ac = if self.acpart > 0 { 1 } else { 0 };
        let comp = if self.component > 0 { 1 } else { 0 };
        self.sdht[ac][comp]
    }

    /// Return the offset of the current *destination* DHT table.
    #[inline]
    fn ddht_offset(&self) -> Option<usize> {
        let ac = if self.acpart > 0 { 1 } else { 0 };
        let comp = if self.component > 0 { 1 } else { 0 };
        self.ddht[ac][comp]
    }

    /// Return the current *source* DQT coefficient value.
    /// Equivalent to C macro `SDQT`: `s->sdqt[comp][1 + acpart]`.
    #[inline]
    fn sdqt_val(&self) -> u8 {
        let comp = if self.component > 0 { 1 } else { 0 };
        let offset = self.sdqt[comp].unwrap();
        self.stbls[offset + 1 + self.acpart as usize]
    }

    /// Return the current *destination* DQT coefficient value.
    /// Equivalent to C macro `DDQT`: `s->ddqt[comp][1 + acpart]`.
    #[inline]
    fn ddqt_val(&self) -> u8 {
        let comp = if self.component > 0 { 1 } else { 0 };
        let offset = self.ddqt[comp].unwrap();
        self.dtbls[offset + 1 + self.acpart as usize]
    }

    // -----------------------------------------------------------------------
    // Quantisation adjustment macros — AADJ, UADJ, BADJ
    // -----------------------------------------------------------------------

    /// AADJ: adjust integer from source to destination quantisation.
    #[inline]
    fn aadj(&self, i: i32) -> i32 {
        let sq = self.sdqt_val() as i32;
        let dq = self.ddqt_val() as i32;
        if sq == dq {
            i
        } else {
            irdiv(i, dq)
        }
    }

    /// UADJ: "un-adjust" — multiply by source quantisation factor.
    #[inline]
    fn uadj(&self, i: i32) -> i32 {
        let sq = self.sdqt_val() as i32;
        let dq = self.ddqt_val() as i32;
        if sq == dq {
            i
        } else {
            i * sq
        }
    }

    /// BADJ: bi-directional adjustment — convert through both quantisations.
    #[inline]
    fn badj(&self, i: i32) -> i32 {
        let sq = self.sdqt_val() as i32;
        let dq = self.ddqt_val() as i32;
        if sq == dq {
            i
        } else {
            irdiv(i * sq, dq)
        }
    }

    // -----------------------------------------------------------------------
    // Huffman lookup — faithful port of C's jpeg_dht_lookup / _symbol
    // -----------------------------------------------------------------------

    /// Look up the current work bits in the *source* DHT table.
    /// Returns `Ok((symbol, width))` on match, `Err(FeedMe)` if not enough
    /// bits, or `Err(Error)` if no match found.
    ///
    /// Port of C's `jpeg_dht_lookup()`.
    pub fn jpeg_dht_lookup(&self) -> Result<(u8, u8), ProcessResult> {
        let dht_offset = match self.sdht_offset() {
            Some(o) => o,
            None => return Err(ProcessResult::Error),
        };
        let dht = &self.stbls[dht_offset..];
        let ss_offset = 17; // symbols start after the 16 count bytes + 1 class/id byte

        let mut code: u16 = 0;
        let mut ss_pos = ss_offset;

        for cw in 1..=16u8 {
            // Not enough bits?
            if cw > self.worklen {
                return Err(ProcessResult::FeedMe);
            }
            let n = dht[cw as usize]; // number of codes of this length
            for _ in 0..n {
                if ss_pos >= dht.len() {
                    return Err(ProcessResult::Error);
                }
                if (self.workbits >> (self.worklen - cw)) == code as u32 {
                    return Ok((dht[ss_pos], cw));
                }
                ss_pos += 1;
                code += 1;
            }
            code <<= 1;
        }

        // No match found
        Err(ProcessResult::Error)
    }

    /// Look up a symbol in the *destination* DHT table to find its Huffman code.
    /// Returns `Ok((bits, width))` on match, `Err(())` if not found.
    ///
    /// Port of C's `jpeg_dht_lookup_symbol()`.
    pub fn jpeg_dht_lookup_symbol(&self, symbol: u8) -> Result<(u16, u8), ()> {
        let dht_offset = match self.ddht_offset() {
            Some(o) => o,
            None => return Err(()),
        };
        let dht = &self.dtbls[dht_offset..];
        let ss_offset = 17;

        let mut code: u16 = 0;
        let mut ss_pos = ss_offset;

        for cw in 1..=16u8 {
            let n = dht[cw as usize];
            for _ in 0..n {
                if ss_pos >= dht.len() {
                    return Err(());
                }
                if dht[ss_pos] == symbol {
                    return Ok((code, cw));
                }
                ss_pos += 1;
                code += 1;
            }
            code <<= 1;
        }

        Err(())
    }

    // -----------------------------------------------------------------------
    // Bit output — faithful port of C's ssdv_outbits / ssdv_outbits_sync
    // -----------------------------------------------------------------------

    /// Output bits to the output buffer.
    /// Returns `BufferFull` when `out_len` reaches 0, `Ok` otherwise.
    ///
    /// Port of C's `ssdv_outbits()`.
    pub fn ssdv_outbits(&mut self, bits: u16, length: u8) -> ProcessResult {
        if length > 0 {
            self.outbits <<= length;
            self.outbits |= (bits as u32) & ((1u32 << length) - 1);
            self.outlen += length;
        }

        while self.outlen >= 8 && self.out_len > 0 {
            let b = (self.outbits >> (self.outlen - 8)) as u8;

            // Write byte to output buffer
            if self.outp < self.out.len() {
                self.out[self.outp] = b;
            } else {
                self.out.push(b);
            }
            self.outp += 1;
            self.outlen -= 8;
            self.out_len -= 1;

            // Byte stuffing: after 0xFF, insert an implicit 0x00 by adding
            // 8 zero bits back to the buffer (they'll be output next iteration)
            if self.out_stuff && b == 0xFF {
                self.outbits &= (1u32 << self.outlen) - 1;
                self.outlen += 8;
            }
        }

        if self.out_len > 0 {
            ProcessResult::Ok
        } else {
            ProcessResult::BufferFull
        }
    }

    /// Sync output bits to byte boundary (pad with 1-bits).
    /// Port of C's `ssdv_outbits_sync()`.
    pub fn ssdv_outbits_sync(&mut self) -> ProcessResult {
        let b = self.outlen % 8;
        if b != 0 {
            self.ssdv_outbits(0xFF, 8 - b)
        } else {
            ProcessResult::Ok
        }
    }

    /// Output a JPEG integer value with Huffman coding.
    /// `rle` is the run-length prefix (0 for DC, 0-15 for AC).
    /// `value` is the coefficient value.
    ///
    /// Port of C's `ssdv_out_jpeg_int()`.
    pub fn ssdv_out_jpeg_int(&mut self, rle: u8, value: i32) {
        let (intbits, intlen) = jpeg_encode_int(value);
        let symbol = (rle << 4) | (intlen & 0x0F);

        if let Ok((huffbits, hufflen)) = self.jpeg_dht_lookup_symbol(symbol) {
            self.ssdv_outbits(huffbits, hufflen);
        }
        // Even if Huffman lookup fails, C code continues (just prints stderr)

        if intlen > 0 {
            self.ssdv_outbits(intbits as u16, intlen);
        }
    }

    // -----------------------------------------------------------------------
    // Core state machine — faithful port of C's ssdv_process()
    // -----------------------------------------------------------------------

    /// Process one step of the JPEG scan data state machine.
    /// Called in a loop by the encoder/decoder until a non-OK result is returned.
    ///
    /// Port of C's `ssdv_process()`.
    pub fn ssdv_process(&mut self) -> ProcessResult {
        if self.state == ProcessorState::Huff {
            // At the start of each MCU, update the reset MCU
            if self.mcupart == 0 && self.acpart == 0 && self.next_reset_mcu > self.reset_mcu {
                self.reset_mcu = self.next_reset_mcu;
            }

            // Look up the Huffman code from the input bit buffer
            let (symbol, width) = match self.jpeg_dht_lookup() {
                Ok(r) => r,
                Err(r) => return r,
            };

            if self.acpart == 0 {
                // DC coefficient
                if symbol == 0x00 {
                    // No change in DC
                    if self.reset_mcu == self.mcu_id as u32
                        && (self.mcupart == 0 || self.mcupart >= self.ycparts)
                    {
                        if self.mode == ProcessorMode::Encoding {
                            self.ssdv_out_jpeg_int(0, self.adc[self.component as usize]);
                        } else {
                            self.ssdv_out_jpeg_int(0, 0 - self.dc[self.component as usize]);
                            self.dc[self.component as usize] = 0;
                        }
                    } else {
                        self.ssdv_out_jpeg_int(0, 0);
                    }
                    // Skip to the next AC part immediately
                    self.acpart += 1;
                } else {
                    // DC value follows, `symbol` bits wide
                    self.state = ProcessorState::Int;
                    self.needbits = symbol;
                }
            } else {
                // AC coefficient
                self.acrle = 0;
                if symbol == 0x00 {
                    // EOB — all remaining AC parts are zero
                    self.ssdv_out_jpeg_int(0, 0);
                    self.acpart = 64;
                } else if symbol == 0xF0 {
                    // Next 16 AC parts are zero
                    self.ssdv_out_jpeg_int(15, 0);
                    self.acpart += 16;
                } else {
                    // Next bits are an integer value
                    self.state = ProcessorState::Int;
                    self.acrle = symbol >> 4;
                    self.acpart += self.acrle;
                    self.needbits = symbol & 0x0F;
                }
            }

            // Clear processed bits
            self.worklen -= width;
            if self.worklen > 0 {
                self.workbits &= (1u32 << self.worklen) - 1;
            } else {
                self.workbits = 0;
            }
        } else if self.state == ProcessorState::Int {
            // Not enough bits yet?
            if self.worklen < self.needbits {
                return ProcessResult::FeedMe;
            }

            // Decode the integer
            let i = jpeg_int(
                (self.workbits >> (self.worklen - self.needbits)) as i32,
                self.needbits as i32,
            );

            if self.acpart == 0 {
                // DC coefficient
                if self.reset_mcu == self.mcu_id as u32
                    && (self.mcupart == 0 || self.mcupart >= self.ycparts)
                {
                    if self.mode == ProcessorMode::Encoding {
                        // Output absolute DC value
                        self.dc[self.component as usize] += self.uadj(i);
                        self.adc[self.component as usize] = self.aadj(self.dc[self.component as usize]);
                        self.ssdv_out_jpeg_int(0, self.adc[self.component as usize]);
                    } else {
                        // Output relative DC value
                        self.ssdv_out_jpeg_int(0, i - self.dc[self.component as usize]);
                        self.dc[self.component as usize] = i;
                    }
                } else if self.mode == ProcessorMode::Decoding {
                    self.dc[self.component as usize] += self.uadj(i);
                    self.ssdv_out_jpeg_int(0, i);
                } else {
                    // Encoding: output relative DC value
                    self.dc[self.component as usize] += self.uadj(i);
                    let adj = self.aadj(self.dc[self.component as usize]);
                    self.ssdv_out_jpeg_int(0, adj - self.adc[self.component as usize]);
                    self.adc[self.component as usize] = adj;
                }
            } else {
                // AC coefficient
                let adj = self.badj(i);
                if adj != 0 {
                    self.accrle += self.acrle;
                    while self.accrle >= 16 {
                        self.ssdv_out_jpeg_int(15, 0);
                        self.accrle -= 16;
                    }
                    self.ssdv_out_jpeg_int(self.accrle, adj);
                    self.accrle = 0;
                } else {
                    // AC value reduced to 0 in DQT conversion
                    if self.acpart >= 63 {
                        self.ssdv_out_jpeg_int(0, 0);
                        self.accrle = 0;
                    } else {
                        self.accrle += self.acrle + 1;
                    }
                }
            }

            // Next AC part to expect
            self.acpart += 1;
            // Next bits are a Huffman code
            self.state = ProcessorState::Huff;

            // Clear processed bits
            self.worklen -= self.needbits;
            if self.worklen > 0 {
                self.workbits &= (1u32 << self.worklen) - 1;
            } else {
                self.workbits = 0;
            }
        }

        // Check MCU completion
        if self.acpart >= 64 {
            self.mcupart += 1;

            // Greyscale padding
            if self.greyscale && self.mcupart == self.ycparts {
                for _ in self.mcupart..(self.ycparts + 2) {
                    self.component = self.mcupart - self.ycparts + 1;
                    self.acpart = 0;
                    self.ssdv_out_jpeg_int(0, 0); // DC
                    self.acpart = 1;
                    self.ssdv_out_jpeg_int(0, 0); // AC
                    self.mcupart += 1;
                }
            }

            // End of MCU block
            if self.mcupart == self.ycparts + 2 {
                self.mcupart = 0;
                self.mcu_id += 1;

                // End of image?
                if self.mcu_id >= self.mcu_count {
                    self.ssdv_outbits_sync();
                    return ProcessResult::Eoi;
                }

                // Set the packet MCU marker — encoder only
                if self.mode == ProcessorMode::Encoding && self.packet_mcu_id == 0xFFFF {
                    self.ssdv_outbits_sync();
                    self.next_reset_mcu = self.mcu_id as u32;
                    self.packet_mcu_id = self.mcu_id;
                    self.packet_mcu_offset = (self.pkt_size_payload - self.out_len
                        + ((self.outlen as usize + 7) / 8)) as u8;
                }

                // Test for a reset marker
                if self.dri > 0 && self.mcu_id > 0 && self.mcu_id % self.dri == 0 {
                    self.state = ProcessorState::Marker;
                    return ProcessResult::FeedMe;
                }
            }

            // Set component for next MCU part
            if self.mcupart < self.ycparts {
                self.component = 0;
            } else {
                self.component = self.mcupart - self.ycparts + 1;
            }

            self.acpart = 0;
            self.accrle = 0;
        }

        // Buffer full?
        if self.out_len == 0 {
            return ProcessResult::BufferFull;
        }

        ProcessResult::Ok
    }

    // -----------------------------------------------------------------------
    // Marker handling — faithful port of ssdv_have_marker / _data
    // -----------------------------------------------------------------------

    /// Process a recognised JPEG marker (without data payload).
    /// Port of C's `ssdv_have_marker()`.
    pub fn ssdv_have_marker(&mut self) -> ProcessResult {
        match self.marker {
            J_SOF0 | J_SOS | J_DRI | J_DHT | J_DQT => {
                // These markers have data — copy it into stbls
                if self.marker_len as usize > TBL_LEN + HBUFF_LEN - self.stbl_len {
                    return ProcessResult::Error;
                }
                self.marker_data_start = self.stbl_len;
                self.marker_data_len = 0;
                // Ensure stbls has enough space
                while self.stbls.len() < self.stbl_len + self.marker_len as usize {
                    self.stbls.push(0);
                }
                self.state = ProcessorState::MarkerData;
            }

            J_SOF2 => {
                // Progressive images not supported
                return ProcessResult::Error;
            }

            J_EOI => {
                self.state = ProcessorState::Eoi;
            }

            0xFFD0..=0xFFD7 => {
                // RST0–RST7: reset DC values and state
                self.dc = [0; 3];
                self.mcupart = 0;
                self.acpart = 0;
                self.component = 0;
                self.acrle = 0;
                self.accrle = 0;
                self.workbits = 0;
                self.worklen = 0;
                self.state = ProcessorState::Huff;
            }

            _ => {
                // Ignore other markers, skip their data
                self.in_skip = self.marker_len as usize;
                self.state = ProcessorState::Marker;
            }
        }

        ProcessResult::Ok
    }

    /// Process the data payload of a JPEG marker.
    /// Port of C's `ssdv_have_marker_data()`.
    pub fn ssdv_have_marker_data(&mut self) -> ProcessResult {
        let base = self.marker_data_start;
        let l = self.marker_len as usize;

        match self.marker {
            J_SOF0 => {
                let d = &self.stbls[base..];

                self.width = ((d[3] as u16) << 8) | d[4] as u16;
                self.height = ((d[1] as u16) << 8) | d[2] as u16;

                // Precision must be 8
                if d[0] != 8 {
                    return ProcessResult::Error;
                }
                // Must have 1 or 3 components
                if d[5] != 1 && d[5] != 3 {
                    return ProcessResult::Error;
                }
                // Max 4080x4080
                if self.width > 4080 || self.height > 4080 {
                    return ProcessResult::Error;
                }
                // Dimensions must be multiple of 16
                if (self.width & 0x0F) != 0 || (self.height & 0x0F) != 0 {
                    return ProcessResult::Error;
                }

                // Parse component info
                for i in 0..d[5] as usize {
                    let dq = &d[i * 3 + 6..];
                    if i == 0 {
                        match dq[1] {
                            0x22 => {
                                self.mcu_mode = 0;
                                self.ycparts = 4;
                            }
                            0x12 => {
                                self.mcu_mode = 1;
                                self.ycparts = 2;
                            }
                            0x21 => {
                                self.mcu_mode = 2;
                                self.ycparts = 2;
                            }
                            0x11 => {
                                self.mcu_mode = 3;
                                self.ycparts = 1;
                            }
                            _ => return ProcessResult::Error,
                        }
                    } else if dq[1] != 0x11 {
                        return ProcessResult::Error;
                    }
                }

                if d[5] == 1 {
                    // Greyscale → convert to 2x1 colour
                    self.greyscale = true;
                    self.mcu_mode = 2;
                    self.ycparts = 2;
                }

                // Calculate MCU count
                let mcu_count: i32 = match self.mcu_mode {
                    0 => (self.width as i32 >> 4) * (self.height as i32 >> 4),
                    1 => (self.width as i32 >> 4) * (self.height as i32 >> 3),
                    2 => (self.width as i32 >> 3) * (self.height as i32 >> 4),
                    3 => (self.width as i32 >> 3) * (self.height as i32 >> 3),
                    _ => return ProcessResult::Error,
                };

                if mcu_count > 0xFFFF || mcu_count < 0 {
                    return ProcessResult::Error;
                }
                self.mcu_count = mcu_count as u16;
            }

            J_SOS => {
                let d = &self.stbls[base..];
                if d[0] != 1 && d[0] != 3 {
                    return ProcessResult::Error;
                }

                // Verify DQT and DHT tables are loaded
                if self.sdqt[0].is_none() || (d[0] > 1 && self.sdqt[1].is_none()) {
                    return ProcessResult::Error;
                }
                if self.sdht[0][0].is_none()
                    || (d[0] > 1 && self.sdht[0][1].is_none())
                    || self.sdht[1][0].is_none()
                    || (d[0] > 1 && self.sdht[1][1].is_none())
                {
                    return ProcessResult::Error;
                }

                // SOS data is followed by image data
                self.state = ProcessorState::Huff;
                return ProcessResult::Ok;
            }

            J_DHT => {
                self.stbl_len += l;
                let mut d_offset = base;
                let mut remaining = l;
                while remaining > 0 {
                    let d0 = self.stbls[d_offset];
                    match d0 {
                        0x00 => self.sdht[0][0] = Some(d_offset),
                        0x01 => self.sdht[0][1] = Some(d_offset),
                        0x10 => self.sdht[1][0] = Some(d_offset),
                        0x11 => self.sdht[1][1] = Some(d_offset),
                        _ => {}
                    }
                    // Calculate table size: 17 header bytes + sum of symbol counts
                    let mut j: usize = 17;
                    for i in 1..=16 {
                        j += self.stbls[d_offset + i] as usize;
                    }
                    if j > remaining {
                        return ProcessResult::Error;
                    }
                    remaining -= j;
                    d_offset += j;
                }
            }

            J_DQT => {
                self.stbl_len += l;
                let mut d_offset = base;
                let mut remaining = l;
                while remaining > 0 {
                    match self.stbls[d_offset] {
                        0x00 => self.sdqt[0] = Some(d_offset),
                        0x01 => self.sdqt[1] = Some(d_offset),
                        _ => {}
                    }
                    if remaining < 65 {
                        return ProcessResult::Error;
                    }
                    remaining -= 65;
                    d_offset += 65;
                }
            }

            J_DRI => {
                let d = &self.stbls[base..];
                self.dri = ((d[0] as u16) << 8) | d[1] as u16;
            }

            _ => {}
        }

        self.state = ProcessorState::Marker;
        ProcessResult::Ok
    }

    // -----------------------------------------------------------------------
    // Gap filling — port of C's ssdv_fill_gap()
    // -----------------------------------------------------------------------

    /// Fill missing MCU blocks with zero-valued coefficients.
    /// Called by the decoder when packet gaps are detected.
    pub fn ssdv_fill_gap(&mut self, next_mcu: u16) {
        if self.mcupart > 0 || self.acpart > 0 {
            // Cleanly end the current MCU part
            if self.acpart > 0 {
                self.ssdv_out_jpeg_int(0, 0);
                self.mcupart += 1;
            }

            // End the current MCU block
            while self.mcupart < self.ycparts + 2 {
                if self.mcupart < self.ycparts {
                    self.component = 0;
                } else {
                    self.component = self.mcupart - self.ycparts + 1;
                }
                self.acpart = 0;
                self.ssdv_out_jpeg_int(0, 0); // DC
                self.acpart = 1;
                self.ssdv_out_jpeg_int(0, 0); // AC
                self.mcupart += 1;
            }

            self.mcu_id += 1;
        }

        // Pad out missing MCUs
        while self.mcu_id < next_mcu {
            for mp in 0..(self.ycparts + 2) {
                self.mcupart = mp;
                if self.mcupart < self.ycparts {
                    self.component = 0;
                } else {
                    self.component = self.mcupart - self.ycparts + 1;
                }
                self.acpart = 0;
                self.ssdv_out_jpeg_int(0, 0); // DC
                self.acpart = 1;
                self.ssdv_out_jpeg_int(0, 0); // AC
            }
            self.mcu_id += 1;
        }
    }

    // -----------------------------------------------------------------------
    // PRNG noise whitening — port of C's ssdv_memset_prng()
    // -----------------------------------------------------------------------

    /// Fill buffer with PRNG noise for whitening.
    /// Uses the same LCG as C: `l = l * 245 + 45`.
    pub fn ssdv_memset_prng(buf: &mut [u8]) {
        let mut l: u8 = 0x00;
        for b in buf.iter_mut() {
            l = l.wrapping_mul(245).wrapping_add(45);
            *b = l;
        }
    }

    // -----------------------------------------------------------------------
    // JPEG output — port of C's ssdv_write_marker / ssdv_out_headers
    // -----------------------------------------------------------------------

    /// Write a JPEG marker with optional data payload.
    /// Port of C's `ssdv_write_marker()`.
    pub fn ssdv_write_marker(&mut self, id: u16, data: &[u8]) {
        self.ssdv_outbits(id, 16);
        if !data.is_empty() {
            self.ssdv_outbits((data.len() as u16) + 2, 16);
            for &b in data {
                self.ssdv_outbits(b as u16, 8);
            }
        }
    }

    /// Output JPEG headers for the decoded image.
    /// Port of C's `ssdv_out_headers()`.
    pub fn ssdv_out_headers(&mut self) {
        self.ssdv_write_marker(J_SOI, &[]);
        self.ssdv_write_marker(J_APP0, &APP0);

        // DQT markers — use destination tables
        if let Some(offset) = self.ddqt[0] {
            let dqt0 = self.dtbls[offset..offset + 65].to_vec();
            self.ssdv_write_marker(J_DQT, &dqt0);
        }
        if let Some(offset) = self.ddqt[1] {
            let dqt1 = self.dtbls[offset..offset + 65].to_vec();
            self.ssdv_write_marker(J_DQT, &dqt1);
        }

        // SOF0 header — build it inline (15 bytes)
        let mut sof0 = [0u8; 15];
        sof0[0] = 8; // Precision
        sof0[1] = (self.height >> 8) as u8;
        sof0[2] = (self.height & 0xFF) as u8;
        sof0[3] = (self.width >> 8) as u8;
        sof0[4] = (self.width & 0xFF) as u8;
        sof0[5] = 3; // Components
        sof0[6] = 1; // Y
        match self.mcu_mode {
            0 => sof0[7] = 0x22,
            1 => sof0[7] = 0x12,
            2 => sof0[7] = 0x21,
            3 => sof0[7] = 0x11,
            _ => {}
        }
        sof0[8] = 0x00;
        sof0[9] = 2; // Cb
        sof0[10] = 0x11;
        sof0[11] = 0x01;
        sof0[12] = 3; // Cr
        sof0[13] = 0x11;
        sof0[14] = 0x01;
        self.ssdv_write_marker(J_SOF0, &sof0);

        // DHT markers — use standard tables from jpeg.rs
        self.ssdv_write_marker(J_DHT, &STD_DHT00);
        self.ssdv_write_marker(J_DHT, &STD_DHT10);
        self.ssdv_write_marker(J_DHT, &STD_DHT01);
        self.ssdv_write_marker(J_DHT, &STD_DHT11);

        self.ssdv_write_marker(J_SOS, &SOS);
    }
}
