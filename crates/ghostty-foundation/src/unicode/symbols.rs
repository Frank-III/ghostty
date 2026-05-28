//! Symbol classification for renderer font constraints.
//!
//! Ported from `src/unicode/symbols_table.zig` / `src/renderer/cell.zig::isSymbol`.

pub use super::symbols_table::get as is_symbol;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enclosed_alphanumeric_is_symbol() {
        assert!(is_symbol(0x2460)); // ① circled digit one
        assert!(is_symbol(0x1F680)); // 🚀 rocket
    }

    #[test]
    fn basic_latin_not_symbol() {
        assert!(!is_symbol(b'a' as u32));
        assert!(!is_symbol(0x4E00)); // CJK ideograph (wide, not symbol table)
    }

    #[test]
    fn out_of_range_not_symbol() {
        assert!(!is_symbol(0x110000));
    }
}
