use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::simple::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mode_report_encode(
    tag: u16,
    state: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    if !(0..=4).contains(&state) {
        return GHOSTTY_INVALID_VALUE;
    }

    let value = u64::from(tag & MODE_VALUE_MASK);
    let ansi = (tag & MODE_ANSI_MASK) != 0;
    let state_value = state as u64;
    let total = mode_report_len(value, ansi, state_value);

    unsafe {
        ptr::write(out_written, total);
    }

    if out.is_null() || out_len < total {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_mode_report(out, value, ansi, state_value);
    }

    GHOSTTY_SUCCESS
}

pub(crate) fn mode_report_len(value: u64, ansi: bool, state: u64) -> usize {
    b"\x1B[".len()
        + if ansi { 0 } else { 1 }
        + decimal_len(value)
        + 1
        + decimal_len(state)
        + b"$y".len()
}

pub(crate) unsafe fn write_mode_report(out: *mut u8, value: u64, ansi: bool, state: u64) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1B[");
        if !ansi {
            write_bytes(out, &mut offset, b"?");
        }
        write_decimal(out, &mut offset, value);
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, state);
        write_bytes(out, &mut offset, b"$y");
    }
}
