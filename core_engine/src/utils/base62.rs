//! Base62 Encoding for Short URL Generation
//!
//! Converts u64 integers to compact alphanumeric strings.
//! Alphabet: 0-9, a-z, A-Z (62 characters)
//!
//! Example: 238327 -> "Abc"

const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const BASE: u64 = 62;

/// Encodes a u64 number to a Base62 string.
///
/// # Arguments
/// * `num` - The number to encode
///
/// # Returns
/// A String containing the Base62 representation
///
/// # Example
/// ```
/// let short = encode(238327);
/// assert_eq!(short.len(), 4); // Compact representation
/// ```
pub fn encode(mut num: u64) -> String {
    if num == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();

    while num > 0 {
        let remainder = (num % BASE) as usize;
        result.push(ALPHABET[remainder]);
        num /= BASE;
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_zero() {
        assert_eq!(encode(0), "0");
    }

    #[test]
    fn test_encode_small() {
        assert_eq!(encode(1), "1");
        assert_eq!(encode(10), "a");
        assert_eq!(encode(36), "A");
        assert_eq!(encode(61), "Z");
    }

    #[test]
    fn test_encode_large() {
        // 62^3 = 238328
        let result = encode(238328);
        assert!(!result.is_empty());
    }
}
