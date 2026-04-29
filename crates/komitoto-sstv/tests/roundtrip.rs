use image::GenericImageView;
use komitoto_sstv::{SstvEncoder, SstvDecoder, SstvMode};
use komitoto_sstv::image_proc::{prepare_image, ResizeStrategy};

fn make_test_image(width: u32, height: u32) -> image::DynamicImage {
    let mut img = image::ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let r = ((x * 255) / width.max(1)) as u8;
            let g = ((y * 255) / height.max(1)) as u8;
            let b = (((x + y) * 255) / (width + height).max(1)) as u8;
            img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
        }
    }
    image::DynamicImage::ImageRgba8(img)
}

#[test]
fn test_all_modes_roundtrip_dimensions() {
    for mode in SstvMode::all() {
        let (w, h) = mode.resolution();
        let img = make_test_image(w, h);
        let encoder = SstvEncoder::new(*mode);
        let samples = encoder.encode(&img).unwrap();
        let decoder = SstvDecoder::new(*mode);
        let decoded = decoder.decode(&samples).unwrap();
        assert_eq!(decoded.width(), w, "{}: width mismatch", mode.name());
        assert_eq!(decoded.height(), h, "{}: height mismatch", mode.name());
    }
}

#[test]
fn test_all_modes_roundtrip_approximate_pixels() {
    // Use a solid color so FM demodulation artifacts don't fail the test
    let solid = image::DynamicImage::ImageRgba8(
        image::ImageBuffer::from_pixel(320, 256, image::Rgba([128, 64, 192, 255]))
    );
    for mode in SstvMode::all() {
        let (w, h) = mode.resolution();
        let img = prepare_image(&solid, w, h, ResizeStrategy::Stretch);
        let encoder = SstvEncoder::new(*mode);
        let samples = encoder.encode(&img).unwrap();
        let decoder = SstvDecoder::new(*mode);
        let decoded = decoder.decode(&samples).unwrap();
        // Spot-check center pixel is roughly correct (allowing YUV quantization + FM noise)
        // Average-color check is more stable than single-pixel spot-check against FM noise
        let mut avg_r = 0u64;
        let mut avg_g = 0u64;
        let mut avg_b = 0u64;
        let total = (w * h) as u64;
        for y in 0..h {
            for x in 0..w {
                let p = decoded.get_pixel(x, y);
                avg_r += p[0] as u64;
                avg_g += p[1] as u64;
                avg_b += p[2] as u64;
            }
        }
        avg_r /= total;
        avg_g /= total;
        avg_b /= total;
        assert!((avg_r as i32 - 128).abs() <= 20, "{}: avg red too far off (got {})", mode.name(), avg_r);
        assert!((avg_g as i32 - 64).abs() <= 20, "{}: avg green too far off (got {})", mode.name(), avg_g);
        assert!((avg_b as i32 - 192).abs() <= 20, "{}: avg blue too far off (got {})", mode.name(), avg_b);
    }
}
