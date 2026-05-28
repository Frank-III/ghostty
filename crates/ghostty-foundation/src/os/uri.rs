//! URI helpers ported from `src/os/uri.zig`.

/// Returns true when `s` is a MAC address in the form `12:34:56:ab:cd:ef`.
pub fn is_valid_mac_address(s: &str) -> bool {
    if s.len() != 17 {
        return false;
    }

    for (i, c) in s.bytes().enumerate() {
        if i % 3 == 2 {
            if c != b':' {
                return false;
            }
        } else if !c.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::is_valid_mac_address;

    #[test]
    fn valid_addresses() {
        assert!(is_valid_mac_address("01:23:45:67:89:Aa"));
        assert!(is_valid_mac_address("Aa:Bb:Cc:Dd:Ee:Ff"));
    }

    #[test]
    fn invalid_addresses() {
        assert!(!is_valid_mac_address(""));
        assert!(!is_valid_mac_address("00:23:45"));
        assert!(!is_valid_mac_address("00:23:45:Xx:Yy:Zz"));
        assert!(!is_valid_mac_address("01-23-45-67-89-Aa"));
        assert!(!is_valid_mac_address("01:23:45:67:89:Aa:Bb"));
    }
}
