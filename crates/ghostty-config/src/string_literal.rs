//! Zig-style config string literals (`src/config/string.zig`).

use crate::error::ConfigError;

/// Parse a config string literal into `out` (output never larger than input).
pub fn parse(out: &mut String, bytes: &str) -> Result<(), ConfigError> {
    out.clear();
    let mut src = 0;
    let data = bytes.as_bytes();
    while src < data.len() {
        let b = data[src];
        if b != b'\\' {
            out.push(char::from(b));
            src += 1;
            continue;
        }
        src += 1;
        if src >= data.len() {
            return Err(ConfigError::InvalidValue);
        }
        match data[src] {
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'\\' => out.push('\\'),
            b'"' => out.push('"'),
            b'\'' => out.push('\''),
            b'u' => {
                src += 1;
                let cp = parse_unicode_escape(data, &mut src)?;
                let mut buf = [0u8; 4];
                let n = char::encode_utf8(cp, &mut buf).len();
                out.push_str(
                    std::str::from_utf8(&buf[..n]).map_err(|_| ConfigError::InvalidValue)?,
                );
            }
            _ => return Err(ConfigError::InvalidValue),
        }
        src += 1;
    }
    Ok(())
}

fn parse_unicode_escape(data: &[u8], src: &mut usize) -> Result<char, ConfigError> {
    if *src >= data.len() || data[*src] != b'{' {
        return Err(ConfigError::InvalidValue);
    }
    *src += 1;
    let start = *src;
    while *src < data.len() && data[*src] != b'}' {
        if !data[*src].is_ascii_hexdigit() {
            return Err(ConfigError::InvalidValue);
        }
        *src += 1;
    }
    if *src >= data.len() {
        return Err(ConfigError::InvalidValue);
    }
    let hex = std::str::from_utf8(&data[start..*src]).map_err(|_| ConfigError::InvalidValue)?;
    let cp = u32::from_str_radix(hex, 16).map_err(|_| ConfigError::InvalidValue)?;
    char::from_u32(cp).ok_or(ConfigError::InvalidValue)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_to(s: &str) -> String {
        let mut out = String::new();
        parse(&mut out, s).unwrap();
        out
    }

    #[test]
    fn empty() {
        assert_eq!(parse_to(""), "");
    }

    #[test]
    fn no_escapes() {
        assert_eq!(parse_to("hello world"), "hello world");
    }

    #[test]
    fn escapes() {
        assert_eq!(parse_to("hello\\nworld"), "hello\nworld");
        assert_eq!(parse_to("hello\\u{1F601}world"), "hello😁world");
    }
}
