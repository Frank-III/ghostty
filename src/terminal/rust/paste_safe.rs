use core::ptr;

use crate::constants::*;
use crate::paste_bytes::*;

pub(crate) unsafe fn paste_data_is_safe(data: *const u8, len: usize) -> bool {
    let mut offset = 0usize;
    while offset < len {
        let byte = unsafe { ptr::read(data.add(offset)) };
        if byte == b'\n' || unsafe { matches_bytes_at(data, len, offset, PASTE_END) } {
            return false;
        }

        offset += 1;
    }

    true
}
