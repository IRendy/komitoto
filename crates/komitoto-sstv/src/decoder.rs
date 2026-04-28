use image::{DynamicImage, ImageBuffer, Rgba};
use crate::error::SstvError;
use crate::mode::SstvMode;
use crate::spec::{from_mode, ChannelType, ColorSpace};
use crate::dsp::{frequency_to_brightness, goertzel, fm_demodulate};
use crate::audio::{read_audio, resample};
use crate::colorspace::yuv_to_rgb;

/// SSTV decoder. Converts SSTV audio waveforms back to images.
pub struct SstvDecoder {
    mode: SstvMode,
}

impl SstvDecoder {
    pub fn new(mode: SstvMode) -> Self {
        Self { mode }
    }

    /// Decode PCM samples into a `DynamicImage`.
    pub fn decode(&self, samples: &[f32]) -> Result<DynamicImage, SstvError> {
        let spec = from_mode(self.mode);
        let sr = spec.sample_rate();
        let (width, height) = spec.resolution();

        // Find the leader tone start (1900 Hz)
        // Only search the first 2 seconds — the leader is always near the start,
        // and scanning the whole file picks up false 1900Hz peaks from image data.
        let window_size = (crate::spec::DURATION_LEADER / 1000.0 * sr as f64).round() as usize;
        let search_limit = (2.0 * sr as f64).round() as usize; // 2 seconds
        let search_end = samples.len().min(search_limit).saturating_sub(window_size);
        let step = sr as usize / 100; // 10ms steps

        // First pass: find maximum 1900Hz energy to establish threshold
        let mut max_energy = 0.0f64;
        for pos in (0..search_end).step_by(step) {
            let energy = goertzel(&samples[pos..pos + window_size], crate::spec::FREQ_LEADER, sr);
            if energy > max_energy {
                max_energy = energy;
            }
        }

        // Second pass: find the FIRST position above 50% of peak energy.
        // This correctly finds the first leader, not the second one.
        let threshold = max_energy * 0.5;
        let mut leader_pos = 0;
        for pos in (0..search_end).step_by(step) {
            let energy = goertzel(&samples[pos..pos + window_size], crate::spec::FREQ_LEADER, sr);
            if energy >= threshold {
                leader_pos = pos;
                break;
            }
        }

        // Move past header: leader + break + leader + VIS (10 bits)
        let leader_samples = (crate::spec::DURATION_LEADER / 1000.0 * sr as f64).round() as usize;
        let break_samples = (10.0 / 1000.0 * sr as f64).round() as usize;
        let vis_samples = (crate::spec::DURATION_VIS_BIT * 10.0 / 1000.0 * sr as f64).round() as usize;
        let mut offset = leader_pos + leader_samples + break_samples + leader_samples + vis_samples;

        // Create image buffer with alpha=255 (fully opaque) so decoded images are visible
        let mut img = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        let channels = spec.channel_order();
        let color_space = spec.color_space();
        let tx_lines = height / spec.lines_per_iteration();

        // Pre-compute scanline sample counts per channel
        let ch_scan_samples: Vec<usize> = (0..channels.len())
            .map(|i| (spec.channel_scan_duration_ms(i) / 1000.0 * sr as f64).round() as usize)
            .collect();

        let sync_samples = (spec.sync_duration_ms() / 1000.0 * sr as f64).round() as usize;

        // Storage for YUV decoding
        let mut y_plane: Option<Vec<Vec<u8>>> = None;
        let mut ry_plane: Option<Vec<Vec<u8>>> = None;
        let mut by_plane: Option<Vec<Vec<u8>>> = None;

        if color_space == ColorSpace::Yuv {
            y_plane = Some(vec![vec![0u8; width as usize]; height as usize]);
            ry_plane = Some(vec![vec![0u8; width as usize]; height as usize]);
            by_plane = Some(vec![vec![0u8; width as usize]; height as usize]);
        }

        for y in 0..tx_lines {
            for (ch_idx, ch_type) in channels.iter().enumerate() {
                // Skip sync before channel if needed
                if spec.has_sync_before_channel(ch_idx) {
                    offset += sync_samples;
                }

                // Skip per-channel separator gap
                let ch_sep_samples = (spec.channel_separator_ms(ch_idx) / 1000.0 * sr as f64).round() as usize;
                offset += ch_sep_samples;

                let scan_samples = ch_scan_samples[ch_idx];
                if offset + scan_samples > samples.len() {
                    return Err(SstvError::DecodingError("Unexpected end of audio during decode".into()));
                }

                let block = &samples[offset..offset + scan_samples];
                let display_y = match ch_type {
                    ChannelType::LumaEven => y * spec.lines_per_iteration() + 1,
                    _ => y * spec.lines_per_iteration(),
                };

                match color_space {
                    ColorSpace::Rgb => {
                        let channel_idx = match ch_type {
                            ChannelType::Red => 0,
                            ChannelType::Green => 1,
                            ChannelType::Blue => 2,
                            _ => unreachable!(),
                        };
                        let channel_vals = self.decode_channel_values(block, width as usize);
                        for (x, val) in channel_vals {
                            let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, display_y as u32);
                            pixel[channel_idx] = val;
                        }
                    }
                    ColorSpace::Yuv => {
                        let effective_y = if height == 240 { display_y & !1 } else { display_y };
                        match ch_type {
                            ChannelType::Luma | ChannelType::LumaEven => {
                                if let Some(ref mut plane) = y_plane {
                                    self.decode_channel_block(block, width as usize, |x, val| {
                                        plane[display_y as usize][x] = val;
                                    });
                                }
                            }
                            ChannelType::ChromaRY => {
                                if let Some(ref mut plane) = ry_plane {
                                    let vals = self.decode_channel_values(block, width as usize);
                                    for (x, val) in &vals {
                                        plane[effective_y as usize][*x] = *val;
                                    }
                                    // For line-pair modes, copy chroma to the even display line
                                    if spec.lines_per_iteration() == 2 && (effective_y as usize) + 1 < height as usize {
                                        for (x, val) in &vals {
                                            plane[effective_y as usize + 1][*x] = *val;
                                        }
                                    }
                                }
                            }
                            ChannelType::ChromaBY => {
                                if let Some(ref mut plane) = by_plane {
                                    let vals = self.decode_channel_values(block, width as usize);
                                    for (x, val) in &vals {
                                        plane[effective_y as usize][*x] = *val;
                                    }
                                    // For line-pair modes, copy chroma to the even display line
                                    if spec.lines_per_iteration() == 2 && (effective_y as usize) + 1 < height as usize {
                                        for (x, val) in &vals {
                                            plane[effective_y as usize + 1][*x] = *val;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                offset += scan_samples;

                // Skip mid-line sync after this channel (e.g., Robot 36 after R-Y)
                if spec.has_sync_after_channel(ch_idx) {
                    let fp_samples = (spec.front_porch_ms() / 1000.0 * sr as f64).round() as usize;
                    offset += fp_samples;
                    offset += sync_samples;
                    let bp_samples = (spec.back_porch_ms() / 1000.0 * sr as f64).round() as usize;
                    offset += bp_samples;
                }
            }

            // Skip trailing fp + sync + bp after all channels in the line
            if spec.has_sync_after_line() {
                let fp_samples = (spec.front_porch_ms() / 1000.0 * sr as f64).round() as usize;
                offset += fp_samples;
                offset += sync_samples;
                let bp_samples = (spec.back_porch_ms() / 1000.0 * sr as f64).round() as usize;
                offset += bp_samples;
            }
        }

        // Convert YUV to RGB if needed
        if color_space == ColorSpace::Yuv {
            if let (Some(y_p), Some(ry_p), Some(by_p)) = (&y_plane, &ry_plane, &by_plane) {
                let h = height as usize;
                for y in 0..h {
                    // For Robot 36, chroma is at half resolution - duplicate lines
                    let ry_line = if h == 240 { (y / 2) * 2 } else { y };
                    let by_line = ry_line;

                    for x in 0..width as usize {
                        let y_val = y_p[y][x];
                        let ry_val = ry_p[ry_line.min(h - 1)][x];
                        let by_val = by_p[by_line.min(h - 1)][x];
                        let (r, g, b) = yuv_to_rgb(y_val, ry_val, by_val);
                        let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, y as u32);
                        pixel[0] = r;
                        pixel[1] = g;
                        pixel[2] = b;
                    }
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(img))
    }

    /// Decode an audio file (WAV or MP3) into a `DynamicImage`.
    /// Automatically resamples to the mode's expected sample rate if needed.
    pub fn decode_audio(&self, path: &str) -> Result<DynamicImage, SstvError> {
        let (samples, file_rate) = read_audio(path)?;
        let spec = from_mode(self.mode);
        let target_rate = spec.sample_rate();

        let samples = if file_rate != target_rate {
            resample(&samples, file_rate, target_rate)
        } else {
            samples
        };

        self.decode(&samples)
    }

    /// Decode a WAV file into a `DynamicImage` (legacy alias for decode_audio).
    #[deprecated(since = "0.2.0", note = "Use decode_audio instead")]
    pub fn decode_wav(&self, wav_path: &str) -> Result<DynamicImage, SstvError> {
        self.decode_audio(wav_path)
    }

    /// Decode an audio file and save the image to a file.
    pub fn decode_to_file(&self, audio_path: &str, img_path: &str) -> Result<(), SstvError> {
        let img = self.decode_audio(audio_path)?;
        img.save(img_path)?;
        Ok(())
    }

    /// Decode a block and call callback for each pixel.
    fn decode_channel_block<F>(&self, block: &[f32], width: usize, mut set_pixel: F)
    where
        F: FnMut(usize, u8),
    {
        let channel_vals = self.decode_channel_values(block, width);
        for (x, val) in channel_vals {
            set_pixel(x, val);
        }
    }

    /// Decode pixel brightness values from a scanline block using FM demodulation.
    ///
    /// Per-pixel Goertzel fails when samples-per-pixel is low (e.g., ~5 for
    /// Martin M1, where both 1500 Hz and 2300 Hz map to the same DFT bin k=1).
    /// Instead, FM demodulate the entire block to get instantaneous frequency
    /// at each sample, then average over each pixel's sample range.
    fn decode_channel_values(&self, block: &[f32], width: usize) -> Vec<(usize, u8)> {
        let sr = from_mode(self.mode).sample_rate();
        let freq_signal = fm_demodulate(block, sr);

        let mut values = Vec::with_capacity(width);
        for x in 0..width {
            // Average instantaneous frequency over this pixel's sample range
            let start = (x * block.len()) / width;
            let end = ((x + 1) * block.len()) / width;
            let start = start.max(1); // skip first sample (FM demod boundary artifact)
            let end = end.max(start + 1).min(freq_signal.len());

            let avg_freq: f64 = freq_signal[start..end].iter().sum::<f64>() / (end - start) as f64;
            let value = frequency_to_brightness(avg_freq);
            values.push((x, value));
        }

        values
    }
}
