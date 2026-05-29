use crate::constants::*;
use crate::paste_bytes::*;
use crate::simple::*;

pub(crate) unsafe fn write_paste(out: *mut u8, data: *const u8, data_len: usize, bracketed: bool) {
    let mut out_offset = 0usize;
    if bracketed {
        unsafe {
            write_bytes(out, &mut out_offset, PASTE_START);
        }
    }
    if data_len > 0 {
        unsafe {
            copy_data_bytes(out, &mut out_offset, data, data_len);
        }
    }
    if bracketed {
        unsafe {
            write_bytes(out, &mut out_offset, PASTE_END);
        }
    }
}
