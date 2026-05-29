/// Kitty defines "Escape code safe UTF-8" as valid UTF-8 with the
/// additional requirement of not containing any C0 escape codes
/// (0x00-0x1f), DEL (0x7f) and C1 escape codes (0x80-0x9f).
///
/// Used by OSC 66 (text sizing) and OSC 99 (Kitty notifications).
///
/// See: https://sw.kovidgoyal.net/kitty/desktop-notifications/#safe-utf8
pub fn is_safe_utf8(s: &[u8]) -> bool {
    let len = s.len();
    let mut i: usize = 0;

    while i < len {
        let b0 = unsafe { *s.get_unchecked(i) };

        let (cp, seq_len) = if b0 <= 0x7F {
            (b0 as u32, 1usize)
        } else if b0 & 0xE0 == 0xC0 {
            if i + 1 >= len {
                return false;
            }
            let b1 = unsafe { *s.get_unchecked(i + 1) };
            if b1 & 0xC0 != 0x80 {
                return false;
            }
            let cp = ((b0 as u32 & 0x1F) << 6) | (b1 as u32 & 0x3F);
            if cp < 0x80 {
                return false;
            }
            (cp, 2)
        } else if b0 & 0xF0 == 0xE0 {
            if i + 2 >= len {
                return false;
            }
            let b1 = unsafe { *s.get_unchecked(i + 1) };
            let b2 = unsafe { *s.get_unchecked(i + 2) };
            if b1 & 0xC0 != 0x80 || b2 & 0xC0 != 0x80 {
                return false;
            }
            let cp = ((b0 as u32 & 0x0F) << 12) | ((b1 as u32 & 0x3F) << 6) | (b2 as u32 & 0x3F);
            if cp < 0x800 {
                return false;
            }
            if cp >= 0xD800 && cp <= 0xDFFF {
                return false;
            }
            (cp, 3)
        } else if b0 & 0xF8 == 0xF0 {
            if i + 3 >= len {
                return false;
            }
            let b1 = unsafe { *s.get_unchecked(i + 1) };
            let b2 = unsafe { *s.get_unchecked(i + 2) };
            let b3 = unsafe { *s.get_unchecked(i + 3) };
            if b1 & 0xC0 != 0x80 || b2 & 0xC0 != 0x80 || b3 & 0xC0 != 0x80 {
                return false;
            }
            let cp = ((b0 as u32 & 0x07) << 18)
                | ((b1 as u32 & 0x3F) << 12)
                | ((b2 as u32 & 0x3F) << 6)
                | (b3 as u32 & 0x3F);
            if cp < 0x10000 || cp > 0x10FFFF {
                return false;
            }
            (cp, 4)
        } else {
            return false;
        };

        match cp {
            0x00..=0x1F | 0x7F | 0x80..=0x9F => return false,
            _ => {}
        }

        i += seq_len;
    }

    true
}
