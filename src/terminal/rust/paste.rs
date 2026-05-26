use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::paste_bytes::*;
use crate::paste_len::*;
use crate::paste_safe::*;
use crate::paste_sanitize::*;
use crate::paste_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_paste_is_safe(data: *const u8, len: usize) -> bool {
    if data.is_null() {
        return true;
    }

    unsafe { paste_data_is_safe(data, len) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_paste_encode(
    data: *mut u8,
    data_len: usize,
    bracketed: bool,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let actual_data_len = if data.is_null() { 0 } else { data_len };

    if !data.is_null() {
        unsafe {
            sanitize_paste_data(data, actual_data_len, bracketed);
        }
    }

    let total = paste_encoded_len(actual_data_len, bracketed);

    unsafe {
        ptr::write(out_written, total);
    }

    if out_len < total || (total > 0 && out.is_null()) {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_paste(out, data, actual_data_len, bracketed);
    }

    GHOSTTY_SUCCESS
}
