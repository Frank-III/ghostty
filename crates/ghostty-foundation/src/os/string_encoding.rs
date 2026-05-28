//! String decoding/encoding helpers ported from `src/os/string_encoding.zig`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeError;

/// Decode a string encoded like bash `printf %q`.
pub fn printf_q_decode(buf: &str) -> Result<String, DecodeError> {
    let data = strip_shell_quotes(buf)?;
    let mut out = String::with_capacity(data.len());
    let mut src = 0;
    while src < data.len() {
        let b = data.as_bytes()[src];
        if b != b'\\' {
            out.push(char::from(b));
            src += 1;
            continue;
        }
        if src + 1 >= data.len() {
            return Err(DecodeError);
        }
        let esc = data.as_bytes()[src + 1];
        let ch = match esc {
            b' ' | b'\\' | b'"' | b'\'' | b'$' => char::from(esc),
            b'e' => '\x1b',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'v' => '\x0b',
            _ => return Err(DecodeError),
        };
        out.push(ch);
        src += 2;
    }
    Ok(out)
}

fn strip_shell_quotes(buf: &str) -> Result<&str, DecodeError> {
    if buf.starts_with("$'") {
        if buf.len() < 3 || !buf.ends_with('\'') {
            return Err(DecodeError);
        }
        return Ok(&buf[2..buf.len() - 1]);
    }
    if buf.starts_with('\'') {
        if buf.len() < 2 || !buf.ends_with('\'') {
            return Err(DecodeError);
        }
        return Ok(&buf[1..buf.len() - 1]);
    }
    Ok(buf)
}

/// Decode URL percent-encoded bytes.
pub fn url_percent_decode(buf: &str) -> Result<Vec<u8>, DecodeError> {
    let mut out = Vec::with_capacity(buf.len());
    let bytes = buf.as_bytes();
    let mut src = 0;
    while src < bytes.len() {
        if bytes[src] != b'%' {
            out.push(bytes[src]);
            src += 1;
            continue;
        }
        if src + 2 >= bytes.len() {
            return Err(DecodeError);
        }
        let hi = hex_nibble(bytes[src + 1])?;
        let lo = hex_nibble(bytes[src + 2])?;
        out.push(hi << 4 | lo);
        src += 3;
    }
    Ok(out)
}

fn hex_nibble(c: u8) -> Result<u8, DecodeError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(DecodeError),
    }
}

fn is_valid_uri_percent_char(c: u8) -> bool {
    match c {
        b' ' | b';' | b'=' => false,
        _ => c.is_ascii_graphic(),
    }
}

/// Write `data` after URI percent-encoding (same rules as Zig `urlPercentEncode`).
pub fn url_percent_encode(data: &str) -> String {
    let mut out = String::with_capacity(data.len());
    for &b in data.as_bytes() {
        if is_valid_uri_percent_char(b) {
            out.push(char::from(b));
        } else {
            use std::fmt::Write as _;
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printf_q_backslash_space() {
        assert_eq!(
            printf_q_decode("bobr\\ kurwa").unwrap(),
            "bobr kurwa"
        );
    }

    #[test]
    fn printf_q_newline() {
        assert_eq!(printf_q_decode("bobr\\nkurwa").unwrap(), "bobr\nkurwa");
    }

    #[test]
    fn printf_q_invalid_escape() {
        assert!(printf_q_decode("bobr\\dkurwa").is_err());
    }

    #[test]
    fn printf_q_trailing_backslash() {
        assert!(printf_q_decode("bobr kurwa\\").is_err());
    }

    #[test]
    fn printf_q_dollar_quote() {
        assert_eq!(
            printf_q_decode("$'bobr kurwa'").unwrap(),
            "bobr kurwa"
        );
    }

    #[test]
    fn printf_q_single_quote() {
        assert_eq!(printf_q_decode("'bobr kurwa'").unwrap(), "bobr kurwa");
    }

    #[test]
    fn printf_q_unclosed_dollar_quote() {
        assert!(printf_q_decode("$'bobr kurwa").is_err());
    }

    #[test]
    fn printf_q_empty_dollar_quote() {
        assert!(printf_q_decode("$'").is_err());
    }

    #[test]
    fn printf_q_unclosed_single_quote() {
        assert!(printf_q_decode("'bobr kurwa").is_err());
    }

    #[test]
    fn printf_q_lone_single_quote() {
        assert!(printf_q_decode("'").is_err());
    }

    #[test]
    fn url_percent_single_bytes() {
        for c in 0u8..=255 {
            let encoded = format!("%{c:02x}");
            let decoded = url_percent_decode(&encoded).unwrap();
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded[0], c);

            let encoded_upper = format!("%{c:02X}");
            let decoded_upper = url_percent_decode(&encoded_upper).unwrap();
            assert_eq!(decoded_upper, [c]);
        }
    }

    #[test]
    fn url_percent_space() {
        assert_eq!(
            url_percent_decode("bobr%20kurwa").unwrap(),
            b"bobr kurwa".as_slice()
        );
    }

    #[test]
    fn url_percent_invalid_digit() {
        assert!(url_percent_decode("bobr%2kurwa").is_err());
    }

    #[test]
    fn url_percent_missing_digits() {
        assert!(url_percent_decode("bobr%kurwa").is_err());
    }

    #[test]
    fn url_percent_double_percent() {
        assert!(url_percent_decode("bobr%%kurwa").is_err());
    }

    #[test]
    fn url_percent_trailing_encoded_space() {
        assert_eq!(
            url_percent_decode("bobr%20kurwa%20").unwrap(),
            b"bobr kurwa ".as_slice()
        );
    }

    #[test]
    fn url_percent_incomplete_trailing() {
        assert!(url_percent_decode("bobr%20kurwa%2").is_err());
        assert!(url_percent_decode("bobr%20kurwa%").is_err());
    }

    #[test]
    fn url_percent_encode_roundtrip_printable() {
        let sample = "bobr kurwa;foo=bar";
        let encoded = url_percent_encode(sample);
        assert_eq!(
            url_percent_decode(&encoded).unwrap(),
            sample.as_bytes()
        );
    }
}
