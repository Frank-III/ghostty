//! Bounds-check-free byte/slice helpers for no_std panic=abort links.

#[inline]
pub(crate) fn find_byte(data: &[u8], b: u8) -> Option<usize> {
    let mut i = 0;
    while i < data.len() {
        if get_u8(data, i) == Some(b) {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[inline]
pub(crate) fn find_byte_from(data: &[u8], b: u8, start: usize) -> Option<usize> {
    let mut i = start;
    while i < data.len() {
        if get_u8(data, i) == Some(b) {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[inline]
pub(crate) fn subslice(data: &[u8], start: usize, end: usize) -> &[u8] {
    if start > end || end > data.len() {
        return &[];
    }
    unsafe { data.get_unchecked(start..end) }
}

#[inline]
pub(crate) fn subslice_from(data: &[u8], start: usize) -> &[u8] {
    if start >= data.len() {
        return &[];
    }
    unsafe { data.get_unchecked(start..data.len()) }
}

#[inline]
pub(crate) fn subslice_len(data: &[u8], len: usize) -> &[u8] {
    subslice(data, 0, len.min(data.len()))
}

#[inline]
pub(crate) fn subslice_u16(data: &[u16], len: usize) -> &[u16] {
    let len = len.min(data.len());
    unsafe { core::slice::from_raw_parts(data.as_ptr(), len) }
}

#[inline]
pub(crate) fn get_u8(data: &[u8], idx: usize) -> Option<u8> {
    if idx < data.len() {
        Some(unsafe { *data.get_unchecked(idx) })
    } else {
        None
    }
}

#[inline]
pub(crate) fn get_u16(data: &[u16], idx: usize) -> Option<u16> {
    if idx < data.len() {
        Some(unsafe { *data.get_unchecked(idx) })
    } else {
        None
    }
}

#[inline]
pub(crate) fn get_u16_bounded(data: &[u16], idx: usize, len: usize) -> Option<u16> {
    if idx < len && idx < data.len() {
        Some(unsafe { *data.get_unchecked(idx) })
    } else {
        None
    }
}

#[inline]
pub(crate) fn is_valid_utf8(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        if b <= 0x7F {
            i += 1;
            continue;
        }
        if (b & 0xE0) == 0xC0 {
            if i + 1 >= bytes.len() {
                return false;
            }
            let b1 = unsafe { *bytes.get_unchecked(i + 1) };
            if (b1 & 0xC0) != 0x80 || b < 0xC2 {
                return false;
            }
            i += 2;
            continue;
        }
        if (b & 0xF0) == 0xE0 {
            if i + 2 >= bytes.len() {
                return false;
            }
            let b1 = unsafe { *bytes.get_unchecked(i + 1) };
            let b2 = unsafe { *bytes.get_unchecked(i + 2) };
            if (b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80 {
                return false;
            }
            i += 3;
            continue;
        }
        if (b & 0xF8) == 0xF0 {
            if i + 3 >= bytes.len() {
                return false;
            }
            let b1 = unsafe { *bytes.get_unchecked(i + 1) };
            let b2 = unsafe { *bytes.get_unchecked(i + 2) };
            let b3 = unsafe { *bytes.get_unchecked(i + 3) };
            if (b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80 || (b3 & 0xC0) != 0x80 || b > 0xF4 {
                return false;
            }
            i += 4;
            continue;
        }
        return false;
    }
    true
}

#[inline]
pub(crate) fn bytes_to_str(bytes: &[u8]) -> &str {
    if is_valid_utf8(bytes) {
        unsafe { core::str::from_utf8_unchecked(bytes) }
    } else {
        ""
    }
}

#[inline]
pub(crate) fn bytes_to_str_opt(bytes: &[u8]) -> Option<&str> {
    if is_valid_utf8(bytes) {
        Some(unsafe { core::str::from_utf8_unchecked(bytes) })
    } else {
        None
    }
}
