use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::mode_report_len::*;
use crate::mode_report_state::*;
use crate::mode_report_tag::*;
use crate::mode_report_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mode_report_encode(
    tag: u16,
    state: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    if !mode_report_state_valid(state) {
        return GHOSTTY_INVALID_VALUE;
    }

    let report_tag = mode_report_tag(tag);
    let state_value = state as u64;
    let total = mode_report_len(report_tag.value, report_tag.ansi, state_value);

    unsafe {
        ptr::write(out_written, total);
    }

    if out.is_null() || out_len < total {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_mode_report(out, report_tag.value, report_tag.ansi, state_value);
    }

    GHOSTTY_SUCCESS
}
