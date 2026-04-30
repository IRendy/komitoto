use crate::callsign::encode_callsign;
use crate::error::SsdvError;
use crate::jpeg::{
    J_COM, J_EOI, J_RST0, J_SOF0, J_TEM, STD_DHT00, STD_DHT01, STD_DHT10, STD_DHT11, STD_DQT0,
    STD_DQT1,
};
use crate::packet::{PacketType, PKT_SIZE, PKT_SIZE_CRC, PKT_SIZE_HEADER, PKT_SIZE_RSCODES};
use crate::process::{ProcessResult, ProcessorMode, ProcessorState, SsdvProcessor};
use crate::reed_solomon;

pub struct SsdvEncoder {
    s: SsdvProcessor,
}

impl SsdvEncoder {
    pub fn new(
        type_: PacketType,
        callsign: &str,
        image_id: u8,
        quality: i8,
        pkt_size: usize,
    ) -> Result<Self, SsdvError> {
        let quality = quality.clamp(0, 7) as u8;

        let rs_size = match type_ {
            PacketType::Normal => PKT_SIZE_RSCODES,
            PacketType::Nofec => 0,
        };

        if pkt_size > PKT_SIZE
            || pkt_size - PKT_SIZE_HEADER - PKT_SIZE_CRC - rs_size < 2
        {
            return Err(SsdvError::EncodingError("Invalid packet size".into()));
        }

        let mut s = SsdvProcessor::new(ProcessorMode::Encoding);
        s.image_id = image_id;
        s.callsign = encode_callsign(callsign);
        s.type_ = type_;
        s.quality = quality;
        s.pkt_size = pkt_size;
        s.ssdv_set_packet_conf();
        // Note: out_stuff remains false during encoding (no JPEG byte stuffing in SSDV payload)
        // Byte stuffing is only needed when outputting JPEG headers during decoding

        s.ddqt[0] = s.dload_standard_dqt(&STD_DQT0, quality);
        s.ddqt[1] = s.dload_standard_dqt(&STD_DQT1, quality);
        s.ddht[0][0] = s.dtblcpy(&STD_DHT00);
        s.ddht[0][1] = s.dtblcpy(&STD_DHT01);
        s.ddht[1][0] = s.dtblcpy(&STD_DHT10);
        s.ddht[1][1] = s.dtblcpy(&STD_DHT11);

        Ok(Self { s })
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.s.inp = data.to_vec();
        self.s.inp_pos = 0;
        self.s.in_len = data.len();
    }

    fn set_buffer(&mut self) {
        self.s.out = vec![0u8; self.s.pkt_size];
        self.s.outp = PKT_SIZE_HEADER;
        self.s.out_len = self.s.pkt_size_payload;
        self.s.ssdv_outbits(0, 0);
    }

    pub fn get_packet(&mut self) -> Result<Option<Vec<u8>>, SsdvError> {
        if self.s.state == ProcessorState::Eoi {
            return Ok(None);
        }

        if self.s.out_len == 0 {
            self.set_buffer();
        }

        while self.s.in_len > 0 {
            let b = self.s.inp[self.s.inp_pos];
            self.s.inp_pos += 1;
            self.s.in_len -= 1;

            if self.s.in_skip > 0 {
                self.s.in_skip -= 1;
                continue;
            }

            match self.s.state {
                ProcessorState::Marker => {
                    self.s.marker = (self.s.marker << 8) | b as u16;
                    if self.s.marker == J_TEM
                        || (self.s.marker >= J_RST0 && self.s.marker <= J_EOI)
                    {
                        self.s.marker_len = 0;
                        let r = self.s.ssdv_have_marker();
                        if r != ProcessResult::Ok {
                            return Err(SsdvError::EncodingError(format!(
                                "Marker error: {:?}",
                                r
                            )));
                        }
                    } else if self.s.marker >= J_SOF0 && self.s.marker <= J_COM {
                        self.s.marker_len = 0;
                        self.s.state = ProcessorState::MarkerLen;
                        self.s.needbits = 16;
                    }
                }

                ProcessorState::MarkerLen => {
                    self.s.marker_len = (self.s.marker_len << 8) | b as u16;
                    self.s.needbits -= 8;
                    if self.s.needbits == 0 {
                        self.s.marker_len -= 2;
                        let r = self.s.ssdv_have_marker();
                        if r != ProcessResult::Ok {
                            return Err(SsdvError::EncodingError(format!(
                                "Marker error: {:?}",
                                r
                            )));
                        }
                    }
                }

                ProcessorState::MarkerData => {
                    let offset = self.s.marker_data_start + self.s.marker_data_len;
                    self.s.stbls[offset] = b;
                    self.s.marker_data_len += 1;
                    if self.s.marker_data_len == self.s.marker_len as usize {
                        let r = self.s.ssdv_have_marker_data();
                        if r != ProcessResult::Ok {
                            return Err(SsdvError::EncodingError(format!(
                                "Marker data error: {:?}",
                                r
                            )));
                        }
                    }
                }

                ProcessorState::Huff | ProcessorState::Int => {
                    if b == 0xFF {
                        self.s.in_skip += 1;
                    }
                    self.s.workbits = (self.s.workbits << 8) | b as u32;
                    self.s.worklen += 8;

                    let mut r = ProcessResult::Ok;
                    while r == ProcessResult::Ok {
                        r = self.s.ssdv_process();
                    }

                    if r == ProcessResult::BufferFull || r == ProcessResult::Eoi {
                        let mut mcu_id = self.s.packet_mcu_id;
                        let mut mcu_offset = self.s.packet_mcu_offset;

                        if mcu_offset != 0xFF
                            && mcu_offset as usize >= self.s.pkt_size_payload
                        {
                            mcu_id = 0xFFFF;
                            mcu_offset = 0xFF;
                            self.s.packet_mcu_offset = self
                                .s
                                .packet_mcu_offset
                                .wrapping_sub(self.s.pkt_size_payload as u8);
                        } else {
                            self.s.packet_mcu_id = 0xFFFF;
                            self.s.packet_mcu_offset = 0xFF;
                        }

                        self.s.out[0] = 0x55;
                        self.s.out[1] = 0x66 + self.s.type_ as u8;
                        self.s.out[2] = (self.s.callsign >> 24) as u8;
                        self.s.out[3] = (self.s.callsign >> 16) as u8;
                        self.s.out[4] = (self.s.callsign >> 8) as u8;
                        self.s.out[5] = self.s.callsign as u8;
                        self.s.out[6] = self.s.image_id;
                        self.s.out[7] = (self.s.packet_id >> 8) as u8;
                        self.s.out[8] = (self.s.packet_id & 0xFF) as u8;
                        self.s.out[9] = (self.s.width >> 4) as u8;
                        self.s.out[10] = (self.s.height >> 4) as u8;
                        self.s.out[11] = 0x00;
                        self.s.out[11] |= (self.s.quality.wrapping_sub(4) & 7) << 3;
                        self.s.out[11] |= (if r == ProcessResult::Eoi { 1 } else { 0 }) << 2;
                        self.s.out[11] |= self.s.mcu_mode & 0x03;
                        self.s.out[12] = mcu_offset;
                        self.s.out[13] = (mcu_id >> 8) as u8;
                        self.s.out[14] = (mcu_id & 0xFF) as u8;

                        if self.s.out_len > 0 {
                            let start = self.s.outp;
                            let end = start + self.s.out_len;
                            SsdvProcessor::ssdv_memset_prng(&mut self.s.out[start..end]);
                        }

                        let x = crc32fast::hash(&self.s.out[1..=self.s.pkt_size_crcdata]);
                        let mut i = 1 + self.s.pkt_size_crcdata;
                        self.s.out[i] = ((x >> 24) & 0xFF) as u8;
                        i += 1;
                        self.s.out[i] = ((x >> 16) & 0xFF) as u8;
                        i += 1;
                        self.s.out[i] = ((x >> 8) & 0xFF) as u8;
                        i += 1;
                        self.s.out[i] = (x & 0xFF) as u8;
                        i += 1;

                        if self.s.type_ == PacketType::Normal {
                            let pad = PKT_SIZE - self.s.pkt_size;
                            let mut parity = [0u8; 32];
                            reed_solomon::encode_rs_8(&self.s.out[1..], &mut parity, pad);
                            self.s.out[i..i + 32].copy_from_slice(&parity);
                        }

                        self.s.packet_id += 1;

                        let packet = self.s.out.clone();
                        if r == ProcessResult::Eoi {
                            self.s.state = ProcessorState::Eoi;
                        }
                        return Ok(Some(packet));
                    } else if r != ProcessResult::FeedMe {
                        return Err(SsdvError::EncodingError(format!(
                            "Process error: {:?}",
                            r
                        )));
                    }
                }

                ProcessorState::Eoi => {}
            }
        }

        Err(SsdvError::EncodingError("Need more input data".into()))
    }
}
