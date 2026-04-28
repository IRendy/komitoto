use image::DynamicImage;
use crate::error::SstvError;
use crate::mode::SstvMode;
use crate::spec::{from_mode, ChannelType, ColorSpace, FREQ_SYNC, FREQ_BLACK};
use crate::dsp::{brightness_to_frequency, generate_tone_phase};
use crate::image_proc::{prepare_image, ResizeStrategy};
use crate::colorspace::rgb_to_yuv;

/// SSTV encoder. Converts images to SSTV audio waveforms.
pub struct SstvEncoder {
    mode: SstvMode,
}

impl SstvEncoder {
    /// Create a new encoder for the given SSTV mode.
    pub fn new(mode: SstvMode) -> Self {
        Self { mode }
    }

    /// Encode a `DynamicImage` into raw PCM audio samples (f32, normalized to [-1.0, 1.0]).
    pub fn encode(&self, img: &DynamicImage) -> Result<Vec<f32>, SstvError> {
        let spec = from_mode(self.mode);
        let sr = spec.sample_rate();

        let mut samples = Vec::new();
        let mut phase = 0.0f64;

        // SSTV header: Leader → Break → Leader → VIS
        let (leader, p) = generate_tone_phase(crate::spec::FREQ_LEADER, crate::spec::DURATION_LEADER, sr, phase);
        samples.extend(leader);
        phase = p;
        // Break: 1200 Hz, 10ms (signals start of VIS code)
        let (brk, p) = generate_tone_phase(crate::spec::FREQ_SYNC, 10.0, sr, phase);
        samples.extend(brk);
        phase = p;
        // Second leader
        let (leader2, p) = generate_tone_phase(crate::spec::FREQ_LEADER, crate::spec::DURATION_LEADER, sr, phase);
        samples.extend(leader2);
        phase = p;

        // VIS code - QSSTV style: 8 data bits (no separate parity), bit=1 -> 1100Hz, bit=0 -> 1300Hz
        let vis_code = spec.vis_code();
        // Start bit: 1200 Hz
        let (start, p) = generate_tone_phase(crate::spec::FREQ_SYNC, crate::spec::DURATION_VIS_BIT, sr, phase);
        samples.extend(start);
        phase = p;
        // 8 data bits (LSB first)
        for bit_idx in 0..8 {
            let bit = (vis_code >> bit_idx) & 1;
            // QSSTV: bit=1 -> 1100Hz, bit=0 -> 1300Hz
            let freq = if bit == 1 {
                crate::spec::FREQ_VIS_BIT_0
            } else {
                crate::spec::FREQ_VIS_BIT_1
            };
            let (tone, p) = generate_tone_phase(freq, crate::spec::DURATION_VIS_BIT, sr, phase);
            samples.extend(tone);
            phase = p;
        }
        // Stop bit: 1200 Hz
        let (stop, p) = generate_tone_phase(crate::spec::FREQ_SYNC, crate::spec::DURATION_VIS_BIT, sr, phase);
        samples.extend(stop);
        phase = p;

        // Scan lines
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let channels = spec.channel_order();
        let color_space = spec.color_space();
        let tx_lines = height / spec.lines_per_iteration();

        for y in 0..tx_lines {
            for (ch_idx, ch_type) in channels.iter().enumerate() {
                // Sync before channel if needed
                if spec.has_sync_before_channel(ch_idx) {
                    let (sync, p) = generate_tone_phase(FREQ_SYNC, spec.sync_duration_ms(), sr, phase);
                    samples.extend(sync);
                    phase = p;
                }

                // Per-channel separator gap before this channel
                let sep_ms = spec.channel_separator_ms(ch_idx);
                if sep_ms > 0.0 {
                    let sep_freq = spec.channel_separator_freq(ch_idx);
                    let (sep, p) = generate_tone_phase(sep_freq, sep_ms, sr, phase);
                    samples.extend(sep);
                    phase = p;
                }

                // Encode the channel scanline
                let display_y = match ch_type {
                    ChannelType::LumaEven => y * spec.lines_per_iteration() + 1,
                    _ => y * spec.lines_per_iteration(),
                };
                let scan_ms = spec.channel_scan_duration_ms(ch_idx);
                let (ch_samples, p) = match color_space {
                    ColorSpace::Rgb => self.encode_rgb_scanline(&rgba, width, display_y, ch_type, scan_ms, sr, phase),
                    ColorSpace::Yuv => self.encode_yuv_scanline(&rgba, width, height, display_y, ch_type, scan_ms, sr, phase),
                };
                phase = p;
                samples.extend(ch_samples);

                // Mid-line sync after this channel (e.g., Robot 36 after R-Y)
                if spec.has_sync_after_channel(ch_idx) {
                    let fp_ms = spec.front_porch_ms();
                    if fp_ms > 0.0 {
                        let (fp, p) = generate_tone_phase(FREQ_BLACK, fp_ms, sr, phase);
                        samples.extend(fp);
                        phase = p;
                    }
                    let (sync, p) = generate_tone_phase(FREQ_SYNC, spec.sync_duration_ms(), sr, phase);
                    samples.extend(sync);
                    phase = p;
                    let bp_ms = spec.back_porch_ms();
                    if bp_ms > 0.0 {
                        let (bp, p) = generate_tone_phase(FREQ_BLACK, bp_ms, sr, phase);
                        samples.extend(bp);
                        phase = p;
                    }
                }
            }

            // Trailing sync after all channels in the line
            if spec.has_sync_after_line() {
                // Front porch: black-level gap before sync pulse
                let fp_ms = spec.front_porch_ms();
                if fp_ms > 0.0 {
                    let (fp, p) = generate_tone_phase(FREQ_BLACK, fp_ms, sr, phase);
                    samples.extend(fp);
                    phase = p;
                }
                // Sync pulse
                let (sync, p) = generate_tone_phase(FREQ_SYNC, spec.sync_duration_ms(), sr, phase);
                samples.extend(sync);
                phase = p;
                // Back porch: black-level gap after sync pulse
                let bp_ms = spec.back_porch_ms();
                if bp_ms > 0.0 {
                    let (bp, p) = generate_tone_phase(FREQ_BLACK, bp_ms, sr, phase);
                    samples.extend(bp);
                    phase = p;
                }
            }
        }

        Ok(samples)
    }

    /// Encode an image file with the given resize strategy.
    pub fn encode_file(&self, path: &str, strategy: ResizeStrategy) -> Result<Vec<f32>, SstvError> {
        let img = image::open(path)?;
        let (w, h) = self.mode.resolution();
        let prepared = prepare_image(&img, w, h, strategy);
        self.encode(&prepared)
    }

    /// Encode an image file and write to a WAV file.
    pub fn encode_to_wav(
        &self,
        img_path: &str,
        wav_path: &str,
        strategy: ResizeStrategy,
    ) -> Result<(), SstvError> {
        let samples = self.encode_file(img_path, strategy)?;
        let spec = from_mode(self.mode);
        crate::audio::write_wav(wav_path, &samples, spec.sample_rate())
    }

    fn encode_rgb_scanline(
        &self,
        rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        width: u32,
        y: u32,
        ch_type: &ChannelType,
        duration_ms: f64,
        sr: u32,
        start_phase: f64,
    ) -> (Vec<f32>, f64) {
        let channel_idx = match ch_type {
            ChannelType::Red => 0,
            ChannelType::Green => 1,
            ChannelType::Blue => 2,
            _ => panic!("Invalid RGB channel type"),
        };
        self.encode_pixel_scanline(rgba, width, y, channel_idx, duration_ms, sr, start_phase)
    }

    fn encode_yuv_scanline(
        &self,
        rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        width: u32,
        height: u32,
        y: u32,
        ch_type: &ChannelType,
        duration_ms: f64,
        sr: u32,
        start_phase: f64,
    ) -> (Vec<f32>, f64) {
        let y_even = y & !1; // For half-res chroma, use even line

        match ch_type {
            ChannelType::Luma | ChannelType::LumaEven => {
                // Y channel: encode luminance for each pixel
                self.encode_pixel_scanline(rgba, width, y, 255, duration_ms, sr, start_phase)
            }
            ChannelType::ChromaRY => {
                // R-Y chroma: for Robot 36, half vertical resolution
                let effective_y = if height == 240 { y_even } else { y };
                self.encode_chroma_scanline(rgba, width, effective_y, 0, duration_ms, sr, start_phase)
            }
            ChannelType::ChromaBY => {
                let effective_y = if height == 240 { y_even } else { y };
                self.encode_chroma_scanline(rgba, width, effective_y, 1, duration_ms, sr, start_phase)
            }
            _ => panic!("Invalid YUV channel type"),
        }
    }

    /// Encode a scanline where each pixel's value at `channel_idx` determines frequency.
    /// For the special value 255, computes luminance from RGB.
    fn encode_pixel_scanline(
        &self,
        rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        width: u32,
        y: u32,
        channel_idx: u8,
        duration_ms: f64,
        sr: u32,
        start_phase: f64,
    ) -> (Vec<f32>, f64) {
        let mut samples = Vec::new();
        let mut phase = start_phase;
        let num_samples = ((duration_ms / 1000.0) * sr as f64).round() as usize;
        let samples_per_pixel = num_samples as f64 / width as f64;

        let mut sample_idx = 0;
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            let value = if channel_idx == 255 {
                // Luminance
                (0.299 * pixel[0] as f64 + 0.587 * pixel[1] as f64 + 0.114 * pixel[2] as f64) as u8
            } else {
                pixel[channel_idx as usize]
            };
            let freq = brightness_to_frequency(value);
            let phase_step = 2.0 * std::f64::consts::PI * freq / sr as f64;

            let start = (x as f64 * samples_per_pixel).round() as usize;
            let end = ((x as f64 + 1.0) * samples_per_pixel).round() as usize;
            let count = end - start;

            for _ in 0..count {
                if sample_idx >= num_samples {
                    break;
                }
                samples.push(phase.sin() as f32);
                phase += phase_step;
                sample_idx += 1;
            }
        }

        // Pad to exact count
        while samples.len() < num_samples {
            samples.push(phase.sin() as f32);
            phase += 2.0 * std::f64::consts::PI * FREQ_BLACK / sr as f64;
        }

        (samples, phase)
    }

    /// Encode chroma (R-Y or B-Y) scanline from RGB pixels.
    fn encode_chroma_scanline(
        &self,
        rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        width: u32,
        y: u32,
        chroma_idx: usize, // 0 = R-Y, 1 = B-Y
        duration_ms: f64,
        sr: u32,
        start_phase: f64,
    ) -> (Vec<f32>, f64) {
        let mut samples = Vec::new();
        let mut phase = start_phase;
        let num_samples = ((duration_ms / 1000.0) * sr as f64).round() as usize;
        let samples_per_pixel = num_samples as f64 / width as f64;

        let mut sample_idx = 0;
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            let (_y_val, ry, by) = rgb_to_yuv(pixel[0], pixel[1], pixel[2]);
            let value = if chroma_idx == 0 { ry } else { by };
            let freq = brightness_to_frequency(value);
            let phase_step = 2.0 * std::f64::consts::PI * freq / sr as f64;

            let start = (x as f64 * samples_per_pixel).round() as usize;
            let end = ((x as f64 + 1.0) * samples_per_pixel).round() as usize;
            let count = end - start;

            for _ in 0..count {
                if sample_idx >= num_samples {
                    break;
                }
                samples.push(phase.sin() as f32);
                phase += phase_step;
                sample_idx += 1;
            }
        }

        while samples.len() < num_samples {
            samples.push(phase.sin() as f32);
            phase += 2.0 * std::f64::consts::PI * FREQ_BLACK / sr as f64;
        }

        (samples, phase)
    }
}
