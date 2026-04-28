//! Generate test results for all SSTV modes: encode an image → WAV → decode back → PNG
use komitoto_sstv::{SstvEncoder, SstvDecoder, SstvMode, image_proc::ResizeStrategy};
use image::{DynamicImage, Rgba, ImageBuffer};

fn make_test_image() -> DynamicImage {
    let width = 320u32;
    let height = 256u32;
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let r = ((x * 255) / width) as u8;
            let g = ((y * 255) / height) as u8;
            let b = (((x + y) * 255) / (width + height)) as u8;
            // Add a white border
            let border = x < 4 || x >= width - 4 || y < 4 || y >= height - 4;
            let pixel = if border {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([r, g, b, 255])
            };
            img.put_pixel(x, y, pixel);
        }
    }
    DynamicImage::ImageRgba8(img)
}

fn main() {
    let out_dir = std::path::Path::new("sstv_test_results");
    std::fs::create_dir_all(out_dir).unwrap();

    let test_img = make_test_image();
    let src_path = out_dir.join("source.png");
    test_img.save(&src_path).unwrap();
    println!("Saved source image: {}", src_path.display());

    let modes = SstvMode::all();
    let mut ok_count = 0;
    let mut err_count = 0;

    for mode in modes {
        let mode_name = mode.name().to_lowercase().replace(' ', "");
        let wav_path = out_dir.join(format!("{}_encoded.wav", mode_name));
        let png_path = out_dir.join(format!("{}_decoded.png", mode_name));

        print!("{:20} encode ... ", mode_name);

        let encoder = SstvEncoder::new(*mode);
        let spec = komitoto_sstv::spec::from_mode(*mode);
        let (w, h) = spec.resolution();
        let prepared = komitoto_sstv::image_proc::prepare_image(&test_img, w, h, ResizeStrategy::Fit);

        match encoder.encode(&prepared) {
            Ok(samples) => {
                // Save WAV
                match komitoto_sstv::audio::write_wav(wav_path.to_str().unwrap(), &samples, spec.sample_rate()) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("WAV write error: {}", e);
                        err_count += 1;
                        continue;
                    }
                }

                let wav_secs = samples.len() as f64 / spec.sample_rate() as f64;
                print!("OK ({:.1}s)  decode ... ", wav_secs);

                let decoder = SstvDecoder::new(*mode);
                match decoder.decode(&samples) {
                    Ok(decoded_img) => {
                        match decoded_img.save(&png_path) {
                            Ok(_) => {
                                println!("OK -> {}", png_path.display());
                                ok_count += 1;
                            }
                            Err(e) => {
                                println!("PNG save error: {}", e);
                                err_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("DECODE ERROR: {}", e);
                        err_count += 1;
                    }
                }
            }
            Err(e) => {
                println!("ENCODE ERROR: {}", e);
                err_count += 1;
            }
        }
    }

    println!("\nDone: {} OK, {} errors out of {} modes", ok_count, err_count, modes.len());
}
