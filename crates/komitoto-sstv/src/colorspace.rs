/// YUV conversion for Robot SSTV modes.
///
/// Robot SSTV encodes Y, R-Y, B-Y as brightness values (0-255) for
/// frequency modulation. The chroma difference signals (R-Y, B-Y) can
/// be negative, so they are offset by +128 before encoding.

fn clamp(v: f64) -> u8 {
    v.round().max(0.0).min(255.0) as u8
}

/// Convert RGB to YUV brightness values for SSTV encoding.
/// Returns (Y, R-Y+128, B-Y+128) all clamped to 0-255.
pub fn rgb_to_yuv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let rf = r as f64;
    let gf = g as f64;
    let bf = b as f64;

    let y = 0.299 * rf + 0.587 * gf + 0.114 * bf;
    let ry = 0.701 * rf - 0.587 * gf - 0.114 * bf;
    let by = -0.299 * rf - 0.587 * gf + 0.886 * bf;

    (clamp(y), clamp(ry + 128.0), clamp(by + 128.0))
}

/// Convert YUV brightness values back to RGB.
/// Expects Y direct (0-255), and R-Y/B-Y offset by +128.
pub fn yuv_to_rgb(y: u8, ry_stored: u8, by_stored: u8) -> (u8, u8, u8) {
    let yf = y as f64;
    let ry = ry_stored as f64 - 128.0;
    let by = by_stored as f64 - 128.0;

    let r = yf + ry;
    let b = yf + by;
    let g = (yf - 0.299 * r - 0.114 * b) / 0.587;

    (clamp(r), clamp(g), clamp(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_yuv_roundtrip() {
        // Use mid-range colors that don't clip in YUV encoding.
        // SSTV chroma (R-Y, B-Y) is offset by +128 and clamped to 0-255,
        // so extreme RGB values lose information.
        let colors = [
            (128, 128, 128),
            (64, 128, 192),
            (200, 100, 50),
            (50, 100, 200),
            (100, 150, 75),
            (180, 90, 140),
            (30, 200, 100),
        ];

        for (r, g, b) in colors {
            let (y, ry, by) = rgb_to_yuv(r, g, b);
            let (r2, g2, b2) = yuv_to_rgb(y, ry, by);
            assert!(
                (r as i32 - r2 as i32).abs() <= 3
                    && (g as i32 - g2 as i32).abs() <= 3
                    && (b as i32 - b2 as i32).abs() <= 3,
                "RGB roundtrip failed for ({},{},{}): got ({},{},{})  yuv=({}, {}, {})",
                r, g, b, r2, g2, b2, y, ry, by
            );
        }
    }
}
