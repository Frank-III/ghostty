use core::ptr;

use crate::paste_bytes::*;

pub(crate) unsafe fn sanitize_paste_data(data: *mut u8, len: usize, bracketed: bool) {
    let mut offset = 0usize;
    while offset < len {
        let byte = unsafe { ptr::read(data.add(offset)) };
        if paste_strip_byte(byte) || (!bracketed && byte == b'\n') {
            unsafe {
                ptr::write(
                    data.add(offset),
                    if !bracketed && byte == b'\n' {
                        b'\r'
                    } else {
                        b' '
                    },
                );
            }
        }
        offset += 1;
    }
}
