use image::DynamicImage;

/// Strategy for resizing an image to a target resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeStrategy {
    /// Center-crop to target aspect ratio, then scale to exact size.
    Crop,
    /// Scale proportionally to fit within target, pad with black.
    Fit,
    /// Stretch to exact target size (may distort aspect ratio).
    Stretch,
}

impl std::str::FromStr for ResizeStrategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "crop" => Ok(Self::Crop),
            "fit" => Ok(Self::Fit),
            "stretch" => Ok(Self::Stretch),
            _ => Err(format!("Unknown resize strategy: '{}'. Use: crop, fit, stretch", s)),
        }
    }
}

/// Prepare an image for SSTV encoding by resizing to the target resolution.
pub fn prepare_image(
    img: &DynamicImage,
    target_w: u32,
    target_h: u32,
    strategy: ResizeStrategy,
) -> DynamicImage {
    match strategy {
        ResizeStrategy::Crop => resize_crop(img, target_w, target_h),
        ResizeStrategy::Fit => resize_fit(img, target_w, target_h),
        ResizeStrategy::Stretch => img.resize_exact(target_w, target_h, image::imageops::Lanczos3),
    }
}

fn resize_crop(img: &DynamicImage, target_w: u32, target_h: u32) -> DynamicImage {
    let (src_w, src_h) = (img.width(), img.height());
    let target_ratio = target_w as f64 / target_h as f64;
    let src_ratio = src_w as f64 / src_h as f64;

    let (crop_w, crop_h) = if src_ratio > target_ratio {
        // Image is wider: crop width
        let w = (src_h as f64 * target_ratio).round() as u32;
        (w, src_h)
    } else {
        // Image is taller: crop height
        let h = (src_w as f64 / target_ratio).round() as u32;
        (src_w, h)
    };

    let x = (src_w - crop_w) / 2;
    let y = (src_h - crop_h) / 2;

    let cropped = img.crop_imm(x, y, crop_w, crop_h);
    cropped.resize_exact(target_w, target_h, image::imageops::Lanczos3)
}

fn resize_fit(img: &DynamicImage, target_w: u32, target_h: u32) -> DynamicImage {
    let (src_w, src_h) = (img.width(), img.height());
    let target_ratio = target_w as f64 / target_h as f64;
    let src_ratio = src_w as f64 / src_h as f64;

    let (scaled_w, scaled_h) = if src_ratio > target_ratio {
        // Scale to width
        let h = (target_w as f64 / src_ratio).round() as u32;
        (target_w, h)
    } else {
        // Scale to height
        let w = (target_h as f64 * src_ratio).round() as u32;
        (w, target_h)
    };

    let scaled = img.resize_exact(scaled_w, scaled_h, image::imageops::Lanczos3);

    // Center on black canvas
    let mut canvas = image::ImageBuffer::new(target_w, target_h);
    let paste_x = (target_w - scaled_w) / 2;
    let paste_y = (target_h - scaled_h) / 2;
    image::imageops::overlay(&mut canvas, &scaled, paste_x as i64, paste_y as i64);
    DynamicImage::ImageRgba8(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_stretch_resizes_to_exact_target() {
        let img = DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(640, 480, Rgba([255, 0, 0, 255])),
        );
        let result = prepare_image(&img, 320, 256, ResizeStrategy::Stretch);
        assert_eq!(result.width(), 320);
        assert_eq!(result.height(), 256);
    }

    #[test]
    fn test_crop_preserves_aspect_ratio() {
        let img = DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(800, 600, Rgba([0, 255, 0, 255])),
        );
        let result = prepare_image(&img, 320, 256, ResizeStrategy::Crop);
        assert_eq!(result.width(), 320);
        assert_eq!(result.height(), 256);
    }

    #[test]
    fn test_fit_does_not_exceed_target() {
        let img = DynamicImage::ImageRgba8(
            image::ImageBuffer::from_pixel(100, 100, Rgba([0, 0, 255, 255])),
        );
        let result = prepare_image(&img, 320, 256, ResizeStrategy::Fit);
        assert_eq!(result.width(), 320);
        assert_eq!(result.height(), 256); // canvas is target size
    }
}
