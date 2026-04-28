# komitoto-sstv

SSTV (Slow-Scan Television) encoding and decoding library for HAM radio, written in Rust.

Supports 14 SSTV modes across Martin, Scottie, Robot, PD, and AVT families, with both RGB and YUV color spaces.

## Supported Modes

| Mode       | Resolution | Color Space | Approx. Duration |
|------------|-----------|-------------|-----------------|
| Martin M1  | 320x256   | RGB         | ~30s            |
| Martin M2  | 320x256   | RGB         | ~15s            |
| Scottie S1 | 320x256   | RGB         | ~28s            |
| Scottie S2 | 320x256   | RGB         | ~18s            |
| Robot 36   | 320x240   | YUV         | ~36s            |
| Robot 72   | 320x240   | YUV         | ~72s            |
| PD 50      | 320x256   | YUV         | ~50s            |
| PD 90      | 320x256   | YUV         | ~90s            |
| PD 120     | 640x496   | YUV         | ~120s           |
| PD 160     | 512x400   | YUV         | ~160s           |
| PD 180     | 640x496   | YUV         | ~180s           |
| PD 240     | 640x496   | YUV         | ~240s           |
| PD 290     | 800x616   | YUV         | ~290s           |
| AVT 90     | 320x256   | RGB         | ~90s            |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
komitoto-sstv = "0.1"
```

### Encode an image to SSTV audio

```no_run
use komitoto_sstv::{SstvEncoder, SstvMode};
use komitoto_sstv::image_proc::ResizeStrategy;

let encoder = SstvEncoder::new(SstvMode::MartinM1);
encoder.encode_to_wav("photo.png", "sstv_output.wav", ResizeStrategy::Fit)?;
```

### Decode SSTV audio to an image

```no_run
use komitoto_sstv::{SstvDecoder, SstvMode};

let decoder = SstvDecoder::new(SstvMode::MartinM1);
decoder.decode_to_file("sstv_input.wav", "decoded.png")?;
```

### Resize strategies

- **Crop** — center-crop to target aspect ratio, then scale
- **Fit** — scale proportionally, pad with black bars
- **Stretch** — stretch to exact size (may distort)

## Features

- Phase-continuous tone generation for clean SSTV signals
- FM demodulation-based decoding (robust against low samples-per-pixel modes)
- Goertzel algorithm for tone detection
- WAV and MP3 audio input support
- Automatic resampling for mismatched sample rates
- RGB/YUV color space conversion for Robot and PD modes

## License

MIT
