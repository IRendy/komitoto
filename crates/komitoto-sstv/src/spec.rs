use crate::SstvMode;

/// Channel types for SSTV encoding order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Red,
    Green,
    Blue,
    Luma,     // Y (luminance)
    LumaEven, // Y even line (for PD modes)
    ChromaRY, // R-Y (chrominance)
    ChromaBY, // B-Y (chrominance)
}

/// Color space used by a mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgb,
    Yuv,
}

/// Universal SSTV frequencies (Hz).
pub const FREQ_LEADER: f64 = 1900.0;
pub const FREQ_VIS_BIT_0: f64 = 1100.0;
pub const FREQ_VIS_BIT_1: f64 = 1300.0;
pub const FREQ_SYNC: f64 = 1200.0;
pub const FREQ_BLACK: f64 = 1500.0;
pub const FREQ_WHITE: f64 = 2300.0;
pub const DURATION_LEADER: f64 = 300.0;
pub const DURATION_VIS_BIT: f64 = 30.0;
pub const SAMPLE_RATE: u32 = 48000;

/// Trait for SSTV mode specifications.
///
/// Each SSTV mode (Martin, Scottie, Robot, PD) implements this trait
/// to define its timing parameters, channel order, and color space.
pub trait ModeSpec {
    /// Sample rate in Hz used for audio encoding/decoding.
    fn sample_rate(&self) -> u32;
    /// Native image resolution as (width, height) in pixels.
    fn resolution(&self) -> (u32, u32);
    /// VIS (Vertical Interval Signaling) code that identifies this mode.
    fn vis_code(&self) -> u8;
    /// Human-readable mode name (e.g., "Martin M1").
    fn name(&self) -> &'static str;
    /// Duration of the horizontal sync pulse in milliseconds.
    fn sync_duration_ms(&self) -> f64;
    /// Duration of the separator tone between channels, if applicable.
    fn separator_duration_ms(&self) -> Option<f64>;
    /// Ordered list of channel types transmitted per scan line.
    fn channel_order(&self) -> &'static [ChannelType];
    /// Duration of the scan segment for the channel at the given index, in milliseconds.
    fn channel_scan_duration_ms(&self, idx: usize) -> f64;
    /// Whether a separator tone precedes the channel at the given index.
    fn has_separator_before_channel(&self, idx: usize) -> bool;
    /// Whether a sync pulse precedes the channel at the given index.
    fn has_sync_before_channel(&self, idx: usize) -> bool;
    /// Whether a sync pulse follows after all channels in a line.
    fn has_sync_after_line(&self) -> bool { false }
    /// Number of display lines processed per scanline iteration (1 for most modes, 2 for PD modes).
    fn lines_per_iteration(&self) -> u32 { 1 }
    /// Color space used by this mode (RGB or YUV).
    fn color_space(&self) -> ColorSpace;
    /// Front porch duration in ms — black-level gap after last channel, before sync pulse.
    /// Used by PD modes (PD240 has fp=2.0ms, others have 0).
    fn front_porch_ms(&self) -> f64 { 0.0 }
    /// Back porch duration in ms — black-level gap after sync pulse, before next line.
    /// Used by PD modes (~2.0-2.08ms) and Robot modes (~2.5-3.0ms).
    fn back_porch_ms(&self) -> f64 { 0.0 }
    /// Whether a sync pulse (fp + sync + bp) should be emitted after this specific channel.
    /// Used by Robot 36 for its mid-line sync between R-Y and Y-even.
    fn has_sync_after_channel(&self, _idx: usize) -> bool { false }
    /// Per-channel gap duration in ms before this channel begins (0.0 = no gap).
    /// Default delegates to has_separator_before_channel + separator_duration_ms for backward compat.
    /// Override for modes that need different gap durations per channel (e.g., Robot bp vs blank).
    fn channel_separator_ms(&self, idx: usize) -> f64 {
        if self.has_separator_before_channel(idx) {
            self.separator_duration_ms().unwrap_or(0.0)
        } else {
            0.0
        }
    }
    /// Frequency (Hz) for the gap before this channel.
    /// Default is FREQ_BLACK (1500Hz). Robot 36 uses FREQ_WHITE (2300Hz) for the
    /// blank gap before B-Y (after Y-even) so decoders can distinguish even from odd lines.
    fn channel_separator_freq(&self, _idx: usize) -> f64 { FREQ_BLACK }

    /// Total samples for full transmission (leader + VIS + all scanlines).
    fn total_samples(&self) -> usize {
        let sr = self.sample_rate() as f64;
        let ms = |dur: f64| ((dur / 1000.0) * sr).round() as usize;

        let (_width, height) = self.resolution();
        let num_channels = self.channel_order().len();
        let tx_lines = height / self.lines_per_iteration();

        let mut total = 0;
        // Header: leader + break + second leader + VIS
        total += ms(DURATION_LEADER);
        total += ms(10.0); // break
        total += ms(DURATION_LEADER);
        total += ms(DURATION_VIS_BIT * 10.0); // 1 start + 8 data + 1 stop

        for _y in 0..tx_lines {
            for ch_idx in 0..num_channels {
                if self.has_sync_before_channel(ch_idx) {
                    total += ms(self.sync_duration_ms());
                }
                total += ms(self.channel_separator_ms(ch_idx));
                total += ms(self.channel_scan_duration_ms(ch_idx));
                if self.has_sync_after_channel(ch_idx) {
                    total += ms(self.front_porch_ms());
                    total += ms(self.sync_duration_ms());
                    total += ms(self.back_porch_ms());
                }
            }
            if self.has_sync_after_line() {
                total += ms(self.front_porch_ms());
                total += ms(self.sync_duration_ms());
                total += ms(self.back_porch_ms());
            }
        }
        total
    }
}

// ─── Martin M1 ───────────────────────────────────────────────────────────

pub struct MartinM1Spec;
impl MartinM1Spec {
    pub const VIS_CODE: u8 = 0xAC;
    pub const SYNC_MS: f64 = 4.862;
    pub const SEP_MS: f64 = 0.572;
    pub const SCAN_MS: f64 = 146.432;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Green, ChannelType::Blue, ChannelType::Red];
}
impl ModeSpec for MartinM1Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 256) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "Martin M1" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { Some(Self::SEP_MS) }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { true }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn color_space(&self) -> ColorSpace { ColorSpace::Rgb }
}

// ─── Martin M2 ───────────────────────────────────────────────────────────

pub struct MartinM2Spec;
impl MartinM2Spec {
    pub const VIS_CODE: u8 = 40;
    pub const SYNC_MS: f64 = 4.862;
    pub const SEP_MS: f64 = 0.572;
    pub const SCAN_MS: f64 = 73.216;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Green, ChannelType::Blue, ChannelType::Red];
}
impl ModeSpec for MartinM2Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 256) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "Martin M2" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { Some(Self::SEP_MS) }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { true }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn color_space(&self) -> ColorSpace { ColorSpace::Rgb }
}

// ─── Scottie S1 ──────────────────────────────────────────────────────────

pub struct ScottieS1Spec;
impl ScottieS1Spec {
    pub const VIS_CODE: u8 = 60;
    pub const SYNC_MS: f64 = 9.0;
    pub const SEP_MS: f64 = 1.5;
    pub const SCAN_MS: f64 = 138.240;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Green, ChannelType::Blue, ChannelType::Red];
}
impl ModeSpec for ScottieS1Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 256) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "Scottie S1" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { Some(Self::SEP_MS) }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { true }
    fn has_sync_before_channel(&self, idx: usize) -> bool { idx == 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Rgb }
}

// ─── Scottie S2 ──────────────────────────────────────────────────────────

pub struct ScottieS2Spec;
impl ScottieS2Spec {
    pub const VIS_CODE: u8 = 0xB8;
    pub const SYNC_MS: f64 = 9.0;
    pub const SEP_MS: f64 = 1.5;
    pub const SCAN_MS: f64 = 88.064;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Green, ChannelType::Blue, ChannelType::Red];
}
impl ModeSpec for ScottieS2Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 256) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "Scottie S2" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { Some(Self::SEP_MS) }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { true }
    fn has_sync_before_channel(&self, idx: usize) -> bool { idx == 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Rgb }
}

// ─── Robot 36 ────────────────────────────────────────────────────────────

pub struct Robot36Spec;
impl Robot36Spec {
    pub const VIS_CODE: u8 = 0x88;
    pub const SYNC_MS: f64 = 9.0;
    pub const FP_MS: f64 = 0.0; // TX value (fptx)
    pub const BP_MS: f64 = 3.0; // TX value (bptx)
    pub const BLANK_MS: f64 = 5.4; // TX value (blanktx)
    // visibleLineLength = (lineLength - fp - bp - blank - sync) / 3.0
    // lineLength = 36002/240 = 150.008ms
    // vis = (150.008 - 0 - 3.0 - 5.4 - 9.0) / 3.0 = 44.203ms
    pub const Y_SCAN_MS: f64 = 88.406; // 2 * visibleLineLength
    pub const CHROMA_SCAN_MS: f64 = 44.203; // visibleLineLength
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::LumaEven, ChannelType::ChromaBY];
    pub const SEPARATOR_DURATIONS: &'static [f64] = &[0.0, Self::BLANK_MS, 0.0, Self::BLANK_MS];
    pub const SCAN_DURATIONS: &'static [f64] = &[Self::Y_SCAN_MS, Self::CHROMA_SCAN_MS, Self::Y_SCAN_MS, Self::CHROMA_SCAN_MS];
}
impl ModeSpec for Robot36Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 240) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "Robot 36" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, idx: usize) -> f64 { Self::SCAN_DURATIONS[idx] }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
    fn has_sync_after_channel(&self, idx: usize) -> bool { idx == 1 } // mid-line sync after R-Y
    fn channel_separator_ms(&self, idx: usize) -> f64 { Self::SEPARATOR_DURATIONS[idx] }
    fn channel_separator_freq(&self, idx: usize) -> f64 {
        // Blank after Y-even (before B-Y) uses 2300Hz so decoders can identify even lines
        if idx == 3 { FREQ_WHITE } else { FREQ_BLACK }
    }
}

// ─── Robot 72 ────────────────────────────────────────────────────────────

pub struct Robot72Spec;
impl Robot72Spec {
    pub const VIS_CODE: u8 = 0x0C;
    pub const SYNC_MS: f64 = 9.0;
    pub const FP_MS: f64 = 0.4; // TX value (fptx)
    pub const BP_MS: f64 = 2.5; // TX value (bptx)
    pub const BLANK_MS: f64 = 6.0; // TX value (blanktx)
    // visibleLineLength = (lineLength - fp - bp - 2*blank - sync) / 4.0
    // lineLength = 72005/240 = 300.021ms
    // vis = (300.021 - 0.4 - 2.5 - 12.0 - 9.0) / 4.0 = 69.030ms
    pub const Y_SCAN_MS: f64 = 138.060; // 2 * visibleLineLength
    pub const CHROMA_SCAN_MS: f64 = 69.030; // visibleLineLength
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY];
    pub const SEPARATOR_DURATIONS: &'static [f64] = &[0.0, Self::BLANK_MS, Self::BLANK_MS];
    pub const SCAN_DURATIONS: &'static [f64] = &[Self::Y_SCAN_MS, Self::CHROMA_SCAN_MS, Self::CHROMA_SCAN_MS];
}
impl ModeSpec for Robot72Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 240) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "Robot 72" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, idx: usize) -> f64 { Self::SCAN_DURATIONS[idx] }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
    fn channel_separator_ms(&self, idx: usize) -> f64 { Self::SEPARATOR_DURATIONS[idx] }
}

// ─── PD 50 ───────────────────────────────────────────────────────────────

pub struct Pd50Spec;
impl Pd50Spec {
    pub const VIS_CODE: u8 = 0xDD;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 0.0;
    pub const BP_MS: f64 = 2.08;
    /// visibleLineLength = (imageTime/dataLines - fp - bp - sync) / 4
    /// = (49687.70/128 - 0 - 2.08 - 20) / 4 = 91.526 ms
    pub const SCAN_MS: f64 = 91.526;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd50Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 256) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 50" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

// ─── PD 90 ───────────────────────────────────────────────────────────────

pub struct Pd90Spec;
impl Pd90Spec {
    pub const VIS_CODE: u8 = 0x63;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 0.0;
    pub const BP_MS: f64 = 2.08;
    /// = (89995.00/128 - 0 - 2.08 - 20) / 4 = 170.252 ms
    pub const SCAN_MS: f64 = 170.252;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd90Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (320, 256) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 90" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

// ─── PD 120 ──────────────────────────────────────────────────────────────

pub struct Pd120Spec;
impl Pd120Spec {
    pub const VIS_CODE: u8 = 0x5F;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 0.0;
    pub const BP_MS: f64 = 2.08;
    /// = (126111.50/248 - 0 - 2.08 - 20) / 4 = 121.609 ms
    pub const SCAN_MS: f64 = 121.609;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd120Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (640, 496) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 120" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

// ─── PD 160 ──────────────────────────────────────────────────────────────

pub struct Pd160Spec;
impl Pd160Spec {
    pub const VIS_CODE: u8 = 0xE2;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 0.0;
    pub const BP_MS: f64 = 2.00;
    /// = (160894.20/200 - 0 - 2.00 - 20) / 4 = 195.618 ms
    pub const SCAN_MS: f64 = 195.618;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd160Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (512, 400) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 160" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

// ─── PD 180 ──────────────────────────────────────────────────────────────

pub struct Pd180Spec;
impl Pd180Spec {
    pub const VIS_CODE: u8 = 0x60;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 0.0;
    pub const BP_MS: f64 = 2.00;
    /// = (187064.50/248 - 0 - 2.00 - 20) / 4 = 183.073 ms
    pub const SCAN_MS: f64 = 183.073;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd180Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (640, 496) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 180" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

// ─── PD 240 ──────────────────────────────────────────────────────────────

pub struct Pd240Spec;
impl Pd240Spec {
    pub const VIS_CODE: u8 = 0xE1;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 2.00;
    pub const BP_MS: f64 = 2.00;
    /// = (248017.00/248 - 2.00 - 2.00 - 20) / 4 = 244.017 ms
    pub const SCAN_MS: f64 = 244.017;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd240Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (640, 496) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 240" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

// ─── PD 290 ──────────────────────────────────────────────────────────────

pub struct Pd290Spec;
impl Pd290Spec {
    pub const VIS_CODE: u8 = 0xDE;
    pub const SYNC_MS: f64 = 20.0;
    pub const FP_MS: f64 = 0.0;
    pub const BP_MS: f64 = 2.00;
    /// = (288702.00/308 - 0 - 2.00 - 20) / 4 = 228.836 ms
    pub const SCAN_MS: f64 = 228.836;
    pub const CHANNELS: &'static [ChannelType] = &[ChannelType::Luma, ChannelType::ChromaRY, ChannelType::ChromaBY, ChannelType::LumaEven];
}
impl ModeSpec for Pd290Spec {
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn resolution(&self) -> (u32, u32) { (800, 616) }
    fn vis_code(&self) -> u8 { Self::VIS_CODE }
    fn name(&self) -> &'static str { "PD 290" }
    fn sync_duration_ms(&self) -> f64 { Self::SYNC_MS }
    fn separator_duration_ms(&self) -> Option<f64> { None }
    fn channel_order(&self) -> &'static [ChannelType] { Self::CHANNELS }
    fn channel_scan_duration_ms(&self, _idx: usize) -> f64 { Self::SCAN_MS }
    fn has_separator_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_before_channel(&self, _idx: usize) -> bool { false }
    fn has_sync_after_line(&self) -> bool { true }
    fn lines_per_iteration(&self) -> u32 { 2 }
    fn color_space(&self) -> ColorSpace { ColorSpace::Yuv }
    fn front_porch_ms(&self) -> f64 { Self::FP_MS }
    fn back_porch_ms(&self) -> f64 { Self::BP_MS }
}

/// Get the ModeSpec for any SstvMode.
pub fn from_mode(mode: SstvMode) -> &'static dyn ModeSpec {
    match mode {
        SstvMode::MartinM1 => &MartinM1Spec,
        SstvMode::MartinM2 => &MartinM2Spec,
        SstvMode::ScottieS1 => &ScottieS1Spec,
        SstvMode::ScottieS2 => &ScottieS2Spec,
        SstvMode::Robot36 => &Robot36Spec,
        SstvMode::Robot72 => &Robot72Spec,
        SstvMode::Pd50 => &Pd50Spec,
        SstvMode::Pd90 => &Pd90Spec,
        SstvMode::Pd120 => &Pd120Spec,
        SstvMode::Pd160 => &Pd160Spec,
        SstvMode::Pd180 => &Pd180Spec,
        SstvMode::Pd240 => &Pd240Spec,
        SstvMode::Pd290 => &Pd290Spec,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_modes_have_valid_specs() {
        for mode in SstvMode::all() {
            let spec = from_mode(*mode);
            assert!(spec.sample_rate() > 0, "{} has no sample rate", spec.name());
            let (w, h) = spec.resolution();
            assert!(w > 0 && h > 0, "{} has no resolution", spec.name());
            assert!(spec.vis_code() > 0, "{} has no VIS code", spec.name());
            let ch_count = spec.channel_order().len();
            assert!(ch_count >= 3, "{} should have at least 3 channels", spec.name());
            assert!(spec.total_samples() > 0, "{} has no total samples", spec.name());
        }
    }

    #[test]
    fn test_martin_m1_total_samples() {
        let spec = MartinM1Spec;
        let total = spec.total_samples();
        // ~115 seconds at 48000 Hz (includes leader+break+leader+VIS header).
        let expected = (115.0 * 48000.0) as usize;
        assert!((total as i64 - expected as i64).abs() < 50000, "Martin M1 total samples {} vs expected ~{}", total, expected);
    }

    #[test]
    fn test_scottie_s1_structure() {
        let spec = ScottieS1Spec;
        // Standard Scottie: Sep→Green, Sep→Blue, Sync+Sep→Red
        assert!(spec.has_separator_before_channel(0));
        assert!(spec.has_separator_before_channel(1));
        assert!(spec.has_separator_before_channel(2));
        assert!(!spec.has_sync_before_channel(0));
        assert!(!spec.has_sync_before_channel(1));
        assert!(spec.has_sync_before_channel(2));
    }

    #[test]
    fn test_pd_modes_have_no_separator() {
        for mode in [SstvMode::Pd50, SstvMode::Pd90, SstvMode::Pd120, SstvMode::Pd160, SstvMode::Pd180, SstvMode::Pd240, SstvMode::Pd290] {
            let spec = from_mode(mode);
            assert!(spec.separator_duration_ms().is_none());
            assert_eq!(spec.color_space(), ColorSpace::Yuv);
            assert_eq!(spec.channel_order().len(), 4);
            assert_eq!(spec.lines_per_iteration(), 2);
            assert!(spec.has_sync_after_line());
            // PD modes use 20ms sync pulse
            assert!((spec.sync_duration_ms() - 20.0).abs() < 0.01,
                "{} sync should be 20ms, got {}", spec.name(), spec.sync_duration_ms());
            // PD modes have back porch (at least 2ms)
            assert!(spec.back_porch_ms() >= 2.0,
                "{} back_porch should be >= 2ms, got {}", spec.name(), spec.back_porch_ms());
            for i in 0..4 {
                assert!(!spec.has_sync_before_channel(i), "{} should not have sync before channel {}", spec.name(), i);
                assert!(!spec.has_separator_before_channel(i));
            }
        }
    }

    #[test]
    fn test_pd240_has_front_porch() {
        let spec = from_mode(SstvMode::Pd240);
        assert!((spec.front_porch_ms() - 2.0).abs() < 0.01,
            "PD240 front_porch should be 2ms, got {}", spec.front_porch_ms());
    }

    #[test]
    fn test_pd_mode_total_time() {
        // Verify PD50 total time ≈ 49.7s
        let spec = from_mode(SstvMode::Pd50);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 49.7).abs() < 1.0,
            "PD50 total time should be ~49.7s, got {:.1}s", total_sec);

        // Verify PD90 total time ≈ 90.0s
        let spec = from_mode(SstvMode::Pd90);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 90.0).abs() < 1.0,
            "PD90 total time should be ~90.0s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_robot_modes_structure() {
        // Robot 36: line-pair mode with mid-line sync
        let spec = from_mode(SstvMode::Robot36);
        assert_eq!(spec.color_space(), ColorSpace::Yuv);
        assert_eq!(spec.resolution(), (320, 240));
        assert_eq!(spec.lines_per_iteration(), 2);
        assert_eq!(spec.channel_order().len(), 4); // Y, R-Y, Y-even, B-Y
        assert!(spec.has_sync_after_channel(1)); // mid-line sync after R-Y
        assert!(spec.has_sync_after_line()); // end-of-pair sync
        assert!((spec.sync_duration_ms() - 9.0).abs() < 0.01);
        assert!(spec.back_porch_ms() > 0.0);
        assert!(spec.channel_separator_ms(1) > 0.0); // blank between Y-odd and R-Y
        assert!(spec.channel_separator_ms(3) > 0.0); // blank between Y-even and B-Y

        // Robot 72: single-line mode with sync at end
        let spec = from_mode(SstvMode::Robot72);
        assert_eq!(spec.color_space(), ColorSpace::Yuv);
        assert_eq!(spec.resolution(), (320, 240));
        assert_eq!(spec.lines_per_iteration(), 1);
        assert_eq!(spec.channel_order().len(), 3); // Y, R-Y, B-Y
        assert!(spec.has_sync_after_line());
        assert!((spec.sync_duration_ms() - 9.0).abs() < 0.01);
        assert!(spec.back_porch_ms() > 0.0);
        assert!(spec.channel_separator_ms(1) > 0.0); // blank between Y and R-Y
        assert!(spec.channel_separator_ms(2) > 0.0); // blank between R-Y and B-Y
    }

    #[test]
    fn test_robot36_total_time() {
        let spec = from_mode(SstvMode::Robot36);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 36.0).abs() < 1.0,
            "Robot36 total time should be ~36s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_robot72_total_time() {
        let spec = from_mode(SstvMode::Robot72);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 72.0).abs() < 1.0,
            "Robot72 total time should be ~72s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_all_modes_have_unique_vis_codes() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for mode in SstvMode::all() {
            let spec = from_mode(*mode);
            assert!(
                seen.insert(spec.vis_code()),
                "Duplicate VIS code 0x{:02X} for {}",
                spec.vis_code(),
                spec.name()
            );
        }
    }

    #[test]
    fn test_martin_m2_total_time() {
        let spec = MartinM2Spec;
        let total = spec.total_samples();
        let expected = (59.0 * 48000.0) as usize;
        assert!((total as i64 - expected as i64).abs() < 50000,
            "Martin M2 total samples {} vs expected ~{}", total, expected);
    }

    #[test]
    fn test_scottie_s1_total_time() {
        let spec = ScottieS1Spec;
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 111.0).abs() < 2.0,
            "Scottie S1 total time should be ~111s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_scottie_s2_total_time() {
        let spec = ScottieS2Spec;
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 72.0).abs() < 2.0,
            "Scottie S2 total time should be ~72s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_pd120_total_time() {
        let spec = from_mode(SstvMode::Pd120);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 127.0).abs() < 2.0,
            "PD120 total time should be ~127s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_pd160_total_time() {
        let spec = from_mode(SstvMode::Pd160);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 161.0).abs() < 2.0,
            "PD160 total time should be ~161s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_pd180_total_time() {
        let spec = from_mode(SstvMode::Pd180);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 188.0).abs() < 2.0,
            "PD180 total time should be ~188s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_pd240_total_time() {
        let spec = from_mode(SstvMode::Pd240);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 249.0).abs() < 2.0,
            "PD240 total time should be ~249s, got {:.1}s", total_sec);
    }

    #[test]
    fn test_pd290_total_time() {
        let spec = from_mode(SstvMode::Pd290);
        let total_sec = spec.total_samples() as f64 / spec.sample_rate() as f64;
        assert!((total_sec - 290.0).abs() < 2.0,
            "PD290 total time should be ~290s, got {:.1}s", total_sec);
    }
}
