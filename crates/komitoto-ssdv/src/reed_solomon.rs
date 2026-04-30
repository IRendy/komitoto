//! Reed-Solomon RS(255,223) encoder/decoder
//! Ported from Phil Karn KA9Q's implementation (LGPL)
//! Tweaked by Philip Heron for SSDV use

const NN: i32 = 255;
const NROOTS: usize = 32;
const FCR: i32 = 112;
const PRIM: i32 = 11;
const IPRIM: i32 = 116;
const A0: i32 = NN; // Special value encoding zero in index form

#[rustfmt::skip]
const ALPHA_TO: [u8; 256] = [
    0x01,0x02,0x04,0x08,0x10,0x20,0x40,0x80,0x87,0x89,0x95,0xAD,0xDD,0x3D,0x7A,0xF4,
    0x6F,0xDE,0x3B,0x76,0xEC,0x5F,0xBE,0xFB,0x71,0xE2,0x43,0x86,0x8B,0x91,0xA5,0xCD,
    0x1D,0x3A,0x74,0xE8,0x57,0xAE,0xDB,0x31,0x62,0xC4,0x0F,0x1E,0x3C,0x78,0xF0,0x67,
    0xCE,0x1B,0x36,0x6C,0xD8,0x37,0x6E,0xDC,0x3F,0x7E,0xFC,0x7F,0xFE,0x7B,0xF6,0x6B,
    0xD6,0x2B,0x56,0xAC,0xDF,0x39,0x72,0xE4,0x4F,0x9E,0xBB,0xF1,0x65,0xCA,0x13,0x26,
    0x4C,0x98,0xB7,0xE9,0x55,0xAA,0xD3,0x21,0x42,0x84,0x8F,0x99,0xB5,0xED,0x5D,0xBA,
    0xF3,0x61,0xC2,0x03,0x06,0x0C,0x18,0x30,0x60,0xC0,0x07,0x0E,0x1C,0x38,0x70,0xE0,
    0x47,0x8E,0x9B,0xB1,0xE5,0x4D,0x9A,0xB3,0xE1,0x45,0x8A,0x93,0xA1,0xC5,0x0D,0x1A,
    0x34,0x68,0xD0,0x27,0x4E,0x9C,0xBF,0xF9,0x75,0xEA,0x53,0xA6,0xCB,0x11,0x22,0x44,
    0x88,0x97,0xA9,0xD5,0x2D,0x5A,0xB4,0xEF,0x59,0xB2,0xE3,0x41,0x82,0x83,0x81,0x85,
    0x8D,0x9D,0xBD,0xFD,0x7D,0xFA,0x73,0xE6,0x4B,0x96,0xAB,0xD1,0x25,0x4A,0x94,0xAF,
    0xD9,0x35,0x6A,0xD4,0x2F,0x5E,0xBC,0xFF,0x79,0xF2,0x63,0xC6,0x0B,0x16,0x2C,0x58,
    0xB0,0xE7,0x49,0x92,0xA3,0xC1,0x05,0x0A,0x14,0x28,0x50,0xA0,0xC7,0x09,0x12,0x24,
    0x48,0x90,0xA7,0xC9,0x15,0x2A,0x54,0xA8,0xD7,0x29,0x52,0xA4,0xCF,0x19,0x32,0x64,
    0xC8,0x17,0x2E,0x5C,0xB8,0xF7,0x69,0xD2,0x23,0x46,0x8C,0x9F,0xB9,0xF5,0x6D,0xDA,
    0x33,0x66,0xCC,0x1F,0x3E,0x7C,0xF8,0x77,0xEE,0x5B,0xB6,0xEB,0x51,0xA2,0xC3,0x00,
];

#[rustfmt::skip]
const INDEX_OF: [u8; 256] = [
    0xFF,0x00,0x01,0x63,0x02,0xC6,0x64,0x6A,0x03,0xCD,0xC7,0xBC,0x65,0x7E,0x6B,0x2A,
    0x04,0x8D,0xCE,0x4E,0xC8,0xD4,0xBD,0xE1,0x66,0xDD,0x7F,0x31,0x6C,0x20,0x2B,0xF3,
    0x05,0x57,0x8E,0xE8,0xCF,0xAC,0x4F,0x83,0xC9,0xD9,0xD5,0x41,0xBE,0x94,0xE2,0xB4,
    0x67,0x27,0xDE,0xF0,0x80,0xB1,0x32,0x35,0x6D,0x45,0x21,0x12,0x2C,0x0D,0xF4,0x38,
    0x06,0x9B,0x58,0x1A,0x8F,0x79,0xE9,0x70,0xD0,0xC2,0xAD,0xA8,0x50,0x75,0x84,0x48,
    0xCA,0xFC,0xDA,0x8A,0xD6,0x54,0x42,0x24,0xBF,0x98,0x95,0xF9,0xE3,0x5E,0xB5,0x15,
    0x68,0x61,0x28,0xBA,0xDF,0x4C,0xF1,0x2F,0x81,0xE6,0xB2,0x3F,0x33,0xEE,0x36,0x10,
    0x6E,0x18,0x46,0xA6,0x22,0x88,0x13,0xF7,0x2D,0xB8,0x0E,0x3D,0xF5,0xA4,0x39,0x3B,
    0x07,0x9E,0x9C,0x9D,0x59,0x9F,0x1B,0x08,0x90,0x09,0x7A,0x1C,0xEA,0xA0,0x71,0x5A,
    0xD1,0x1D,0xC3,0x7B,0xAE,0x0A,0xA9,0x91,0x51,0x5B,0x76,0x72,0x85,0xA1,0x49,0xEB,
    0xCB,0x7C,0xFD,0xC4,0xDB,0x1E,0x8B,0xD2,0xD7,0x92,0x55,0xAA,0x43,0x0B,0x25,0xAF,
    0xC0,0x73,0x99,0x77,0x96,0x5C,0xFA,0x52,0xE4,0xEC,0x5F,0x4A,0xB6,0xA2,0x16,0x86,
    0x69,0xC5,0x62,0xFE,0x29,0x7D,0xBB,0xCC,0xE0,0xD3,0x4D,0x8C,0xF2,0x1F,0x30,0xDC,
    0x82,0xAB,0xE7,0x56,0xB3,0x93,0x40,0xD8,0x34,0xB0,0xEF,0x26,0x37,0x0C,0x11,0x44,
    0x6F,0x78,0x19,0x9A,0x47,0x74,0xA7,0xC1,0x23,0x53,0x89,0xFB,0x14,0x5D,0xF8,0x97,
    0x2E,0x4B,0xB9,0x60,0x0F,0xED,0x3E,0xE5,0xF6,0x87,0xA5,0x17,0x3A,0xA3,0x3C,0xB7,
];

#[rustfmt::skip]
const GENPOLY: [u8; 33] = [
    0x00,0xF9,0x3B,0x42,0x04,0x2B,0x7E,0xFB,0x61,0x1E,0x03,0xD5,0x32,0x42,0xAA,0x05,
    0x18,0x05,0xAA,0x42,0x32,0xD5,0x03,0x1E,0x61,0xFB,0x7E,0x2B,0x04,0x42,0x3B,0xF9,
    0x00,
];

fn mod255(x: i32) -> i32 {
    let mut x = x;
    while x >= 255 {
        x -= 255;
        x = (x >> 8) + (x & 255);
    }
    x
}

/// Encode Reed-Solomon parity bytes.
/// `data` is the input data (NN - NROOTS - pad bytes),
/// `parity` is the output 32 parity bytes,
/// `pad` is the number of padded (virtual) bytes (for shortened codes).
pub fn encode_rs_8(data: &[u8], parity: &mut [u8; NROOTS], pad: usize) {
    parity.fill(0);

    let data_len = (NN as usize - NROOTS) - pad;
    for i in 0..data_len {
        let feedback = INDEX_OF[(data[i] ^ parity[0]) as usize];
        if feedback != A0 as u8 {
            let fb = feedback as i32;
            for j in 1..NROOTS {
                parity[j] ^= ALPHA_TO[mod255(fb + GENPOLY[NROOTS - j] as i32) as usize];
            }
        }

        // Shift
        parity.copy_within(1..NROOTS, 0);
        if feedback != A0 as u8 {
            parity[NROOTS - 1] = ALPHA_TO[mod255(feedback as i32 + GENPOLY[0] as i32) as usize];
        } else {
            parity[NROOTS - 1] = 0;
        }
    }
}

/// Decode Reed-Solomon errors in the data.
/// `data` is modified in-place with corrected values.
/// `pad` is the number of padded bytes.
/// Returns the number of errors corrected, or -1 if uncorrectable.
pub fn decode_rs_8(data: &mut [u8], pad: i32) -> i32 {
    let mut lambda = [0u8; NROOTS + 1];
    let mut s = [0u8; NROOTS];
    let mut b = [0u8; NROOTS + 1];
    let mut t = [0u8; NROOTS + 1];
    let mut omega = [0u8; NROOTS + 1];
    let mut root = [0u8; NROOTS];
    let mut reg = [0u8; NROOTS + 1];
    let mut loc = [0i32; NROOTS];

    if pad < 0 || pad > 222 {
        return -1;
    }

    let nn_pad = (NN - pad) as usize;

    // Form the syndromes
    for i in 0..NROOTS {
        s[i] = data[0];
    }
    for j in 1..nn_pad {
        for i in 0..NROOTS {
            if s[i] == 0 {
                s[i] = data[j];
            } else {
                s[i] = data[j] ^ ALPHA_TO[mod255(INDEX_OF[s[i] as usize] as i32 + (FCR + i as i32) * PRIM) as usize];
            }
        }
    }

    // Convert syndromes to index form, checking for nonzero condition
    let mut syn_error = false;
    for i in 0..NROOTS {
        syn_error |= s[i] != 0;
        s[i] = INDEX_OF[s[i] as usize];
    }

    if !syn_error {
        return 0;
    }

    lambda[1..].fill(0);
    lambda[0] = 1;

    for i in 0..=NROOTS {
        b[i] = INDEX_OF[lambda[i] as usize];
    }

    // Berlekamp-Massey algorithm
    let mut r: i32 = 0;
    let mut el: i32 = 0;
    while { r += 1; r <= NROOTS as i32 } {
        let mut discr_r: u8 = 0;
        for i in 0..r as usize {
            if lambda[i] != 0 && s[r as usize - i - 1] != A0 as u8 {
                discr_r ^= ALPHA_TO[mod255(INDEX_OF[lambda[i] as usize] as i32 + s[r as usize - i - 1] as i32) as usize];
            }
        }
        discr_r = INDEX_OF[discr_r as usize];

        if discr_r == A0 as u8 {
            b.copy_within(0..NROOTS, 1);
            b[0] = A0 as u8;
        } else {
            t[0] = lambda[0];
            for i in 0..NROOTS {
                if b[i] != A0 as u8 {
                    t[i + 1] = lambda[i + 1] ^ ALPHA_TO[mod255(discr_r as i32 + b[i] as i32) as usize];
                } else {
                    t[i + 1] = lambda[i + 1];
                }
            }

            if 2 * el <= r + 0 - 1 {
                el = r + 0 - el;
                for i in 0..=NROOTS {
                    b[i] = if lambda[i] == 0 {
                        A0 as u8
                    } else {
                        mod255(INDEX_OF[lambda[i] as usize] as i32 - discr_r as i32 + NN) as u8
                    };
                }
            } else {
                b.copy_within(0..NROOTS, 1);
                b[0] = A0 as u8;
            }

            lambda.copy_from_slice(&t);
        }
    }

    // Convert lambda to index form and compute deg(lambda)
    let mut deg_lambda = 0;
    for i in 0..=NROOTS {
        lambda[i] = INDEX_OF[lambda[i] as usize];
        if lambda[i] != A0 as u8 {
            deg_lambda = i;
        }
    }

    // Chien search
    reg[1..].copy_from_slice(&lambda[1..]);
    let mut count = 0;
    let mut k = IPRIM - 1;
    for i in 1..=NN as i32 {
        let mut q: u8 = 1;
        for j in (1..=deg_lambda).rev() {
            if reg[j] != A0 as u8 {
                reg[j] = mod255(reg[j] as i32 + j as i32) as u8;
                q ^= ALPHA_TO[reg[j] as usize];
            }
        }

        if q != 0 {
            k = mod255(k + IPRIM);
            continue;
        }

        root[count] = i as u8;
        loc[count] = k;
        count += 1;
        if count == deg_lambda {
            break;
        }
        k = mod255(k + IPRIM);
    }

    if deg_lambda != count {
        return -1;
    }

    // Compute err+eras evaluator poly omega(x)
    let deg_omega = deg_lambda - 1;
    for i in 0..=deg_omega {
        let mut tmp: u8 = 0;
        for j in (0..=i).rev() {
            if s[i - j] != A0 as u8 && lambda[j] != A0 as u8 {
                tmp ^= ALPHA_TO[mod255(s[i - j] as i32 + lambda[j] as i32) as usize];
            }
        }
        omega[i] = INDEX_OF[tmp as usize];
    }

    // Compute error values
    for j in (0..count).rev() {
        let mut num1: u8 = 0;
        for i in (0..=deg_omega as i32).rev() {
            if omega[i as usize] != A0 as u8 {
                num1 ^= ALPHA_TO[mod255(omega[i as usize] as i32 + i * root[j] as i32) as usize];
            }
        }
        let num2 = ALPHA_TO[mod255(root[j] as i32 * (FCR - 1) + NN) as usize];
        let mut den: u8 = 0;

        let mut i = std::cmp::min(deg_lambda, NROOTS - 1) & !1;
        while i > 0 {
            // i is even here (we step by -2 from an even start)
            if lambda[i + 1] != A0 as u8 {
                den ^= ALPHA_TO[mod255(lambda[i + 1] as i32 + i as i32 * root[j] as i32) as usize];
            }
            if i >= 2 { i -= 2; } else { break; }
        }
        // Also check i==0 case (even)
        if lambda[1] != A0 as u8 {
            den ^= ALPHA_TO[mod255(lambda[1] as i32) as usize];
        }

        if num1 != 0 && loc[j] >= pad {
            let idx = mod255(
                INDEX_OF[num1 as usize] as i32 +
                INDEX_OF[num2 as usize] as i32 +
                NN -
                INDEX_OF[den as usize] as i32
            );
            data[loc[j] as usize - pad as usize] ^= ALPHA_TO[idx as usize];
        }
    }

    count as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rs_encode_decode_no_errors() {
        let mut data = [0u8; 223];
        for i in 0..223 {
            data[i] = (i * 7 + 13) as u8;
        }
        let mut parity = [0u8; NROOTS];
        encode_rs_8(&data, &mut parity, 0);

        // Build full codeword
        let mut codeword = [0u8; 255];
        codeword[..223].copy_from_slice(&data);
        codeword[223..].copy_from_slice(&parity);

        // Decode with no errors
        let result = decode_rs_8(&mut codeword, 0);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_rs_encode_decode_with_errors() {
        let mut data = [0u8; 223];
        for i in 0..223 {
            data[i] = (i * 7 + 13) as u8;
        }
        let mut parity = [0u8; NROOTS];
        encode_rs_8(&data, &mut parity, 0);

        let mut codeword = [0u8; 255];
        codeword[..223].copy_from_slice(&data);
        codeword[223..].copy_from_slice(&parity);

        // Introduce 5 errors
        codeword[10] ^= 0xFF;
        codeword[50] ^= 0x42;
        codeword[100] ^= 0x37;
        codeword[150] ^= 0xAB;
        codeword[200] ^= 0x11;

        let result = decode_rs_8(&mut codeword, 0);
        assert!(result >= 0);
        assert_eq!(&codeword[..223], &data[..]);
    }
}
