/// Maximum number of characters in a callsign
pub const MAX_CALLSIGN: usize = 6;

/// Encode a callsign string to a base-40 u32 value.
/// Only A-Z and 0-9 characters are encoded; others are treated as 0.
/// Maximum 6 characters.
pub fn encode_callsign(callsign: &str) -> u32 {
    let chars: Vec<char> = callsign.chars().take(MAX_CALLSIGN).collect();
    let mut x: u32 = 0;
    for &c in chars.iter().rev() {
        x = x.saturating_mul(40);
        if c >= 'A' && c <= 'Z' {
            x += (c as u32) - ('A' as u32) + 14;
        } else if c >= 'a' && c <= 'z' {
            x += (c as u32) - ('a' as u32) + 14;
        } else if c >= '0' && c <= '9' {
            x += (c as u32) - ('0' as u32) + 1;
        }
        // Other characters map to 0 (no addition)
    }
    x
}

/// Decode a base-40 u32 value back to a callsign string.
/// Returns empty string if the code is invalid (> 0xF423FFFF).
pub fn decode_callsign(code: u32) -> String {
    if code > 0xF423FFFF {
        return String::new();
    }

    let mut result = String::new();
    let mut code = code;
    while code > 0 {
        let s = code % 40;
        let c = if s == 0 {
            '-'
        } else if s < 11 {
            (b'0' + (s - 1) as u8) as char
        } else if s < 14 {
            '-'
        } else {
            (b'A' + (s - 14) as u8) as char
        };
        result.push(c);
        code /= 40;
    }

    result
}

/// Validate a callsign for SSDV encoding.
/// Returns Ok(()) if valid, Err with reason if not.
pub fn validate_callsign(callsign: &str) -> Result<(), String> {
    let cs = callsign.trim();
    if cs.is_empty() {
        return Err("Callsign is empty".into());
    }
    if cs.len() > MAX_CALLSIGN {
        return Err(format!("Callsign too long (max {} characters)", MAX_CALLSIGN));
    }
    for c in cs.chars() {
        if !c.is_ascii_uppercase() && !c.is_ascii_lowercase() && !c.is_ascii_digit() {
            return Err(format!("Invalid character '{}' in callsign (only A-Z, 0-9 allowed)", c));
        }
    }
    let encoded = encode_callsign(cs);
    if encoded > 0xF423FFFF {
        return Err("Callsign encodes to a value too large for base-40".into());
    }
    Ok(())
}

/// Check if a callsign is a valid amateur radio callsign (ITU format).
/// ITU format: prefix (1-2 letters) + digit + suffix (1-3 letters/digits)
pub fn is_valid_ham_callsign(callsign: &str) -> bool {
    let cs = callsign.trim().to_uppercase();
    let cs = cs.as_str();
    let chars: Vec<char> = cs.chars().collect();
    let len = chars.len();

    if len < 3 || len > 6 {
        return false;
    }

    // Must start with at least 1 letter (prefix)
    if !chars[0].is_ascii_alphabetic() {
        return false;
    }

    // Find the digit that separates prefix from suffix
    let digit_pos = {
        let mut found = None;
        for i in 0..len {
            if chars[i].is_ascii_digit() {
                found = Some(i);
                break;
            }
        }
        match found {
            Some(pos) => pos,
            None => return false,
        }
    };

    // Prefix: 1-2 letters before the digit
    if digit_pos == 0 || digit_pos > 2 {
        return false;
    }
    for i in 0..digit_pos {
        if !chars[i].is_ascii_alphabetic() {
            return false;
        }
    }

    // Suffix: 1-3 alphanumeric characters after the digit
    let suffix_len = len - digit_pos - 1;
    if suffix_len < 1 || suffix_len > 3 {
        return false;
    }
    for i in (digit_pos + 1)..len {
        if !chars[i].is_ascii_alphanumeric() {
            return false;
        }
    }

    true
}

/// Get the ITU country/region for a callsign prefix
pub fn itu_prefix_info(callsign: &str) -> &'static str {
    let cs = callsign.trim().to_uppercase();
    let first = cs.chars().next().unwrap_or(' ');

    match first {
        'A' => "United States",
        'B' => "China",
        'C' => if cs.len() > 1 && cs.chars().nth(1) == Some('E') { "Cuba" } else { "China" },
        'D' => "Germany",
        'E' => "Spain",
        'F' => "France",
        'G' => "England",
        'H' => if cs.len() > 1 && cs.chars().nth(1) == Some('A') { "Hungary" } else { "Switzerland" },
        'I' => "Italy",
        'J' => "Japan",
        'K' => "United States",
        'L' => if cs.len() > 1 && cs.chars().nth(1) == Some('U') { "Luxembourg" } else { "Norway" },
        'M' => "England",
        'N' => "United States",
        'O' => if cs.len() > 1 && cs.chars().nth(1) == Some('M') { "Oman" } else { "Finland" },
        'P' => if cs.len() > 1 && cs.chars().nth(1) == Some('Y') { "Paraguay" } else { "Netherlands" },
        'Q' => "Unknown",
        'R' => "Russia",
        'S' => if cs.len() > 1 && cs.chars().nth(1) == Some('U') { "Uruguay" } else { "Sweden" },
        'T' => if cs.len() > 1 && cs.chars().nth(1) == Some('U') { "Tuvalu" } else { "Italy" },
        'U' => "Russia",
        'V' => if cs.len() > 1 && cs.chars().nth(1) == Some('E') { "Canada" }
               else if cs.chars().nth(1) == Some('K') { "India" }
               else if cs.chars().nth(1) == Some('R') { "Australia" }
               else { "Unknown" },
        'W' => "United States",
        'X' => if cs.len() > 1 && cs.chars().nth(1) == Some('T') { "New Zealand" } else { "Unknown" },
        'Y' => if cs.len() > 1 && cs.chars().nth(1) == Some('B') { "Indonesia" } else { "Unknown" },
        'Z' => if cs.len() > 1 && cs.chars().nth(1) == Some('L') { "New Zealand" }
               else if cs.chars().nth(1) == Some('S') { "Zimbabwe" }
               else { "Unknown" },
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let calls = "BD7ACE";
        let encoded = encode_callsign(calls);
        let decoded = decode_callsign(encoded);
        assert_eq!(decoded, calls);
    }

    #[test]
    fn test_encode_callsign_numeric() {
        let calls = "VE3RTL";
        let encoded = encode_callsign(calls);
        assert!(encoded <= 0xF423FFFF);
        let decoded = decode_callsign(encoded);
        assert_eq!(decoded, calls);
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate_callsign("BD7ACE").is_ok());
        assert!(validate_callsign("W1AW").is_ok());
    }

    #[test]
    fn test_validate_too_long() {
        assert!(validate_callsign("TOOLONG1").is_err());
    }

    #[test]
    fn test_validate_invalid_chars() {
        assert!(validate_callsign("BD7-ACE").is_err());
    }

    #[test]
    fn test_is_valid_ham() {
        assert!(is_valid_ham_callsign("BD7ACE"));
        assert!(is_valid_ham_callsign("W1AW"));
        assert!(is_valid_ham_callsign("JA1AA"));
        assert!(is_valid_ham_callsign("VR2X"));
        assert!(!is_valid_ham_callsign("AB"));      // too short
        assert!(!is_valid_ham_callsign("123"));      // no letters
        assert!(!is_valid_ham_callsign("ABCDEFG"));  // too long, no digit
    }

    #[test]
    fn test_itu_prefix() {
        assert_eq!(itu_prefix_info("BD7ACE"), "China");
        assert_eq!(itu_prefix_info("W1AW"), "United States");
        assert_eq!(itu_prefix_info("JA1AA"), "Japan");
    }
}
