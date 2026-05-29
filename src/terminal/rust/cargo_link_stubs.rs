//! No-op Zig bridge symbols for cargo-linked `ghostty-vt` integration tests.

use core::ffi::c_void;

use crate::constants::GhosttySizeReportSize;

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_write_pty(
    _wrapper: *mut c_void,
    _ptr: *const u8,
    _len: usize,
) {
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_bell(_wrapper: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_title_changed(_wrapper: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_enquiry(_wrapper: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_xtversion(_wrapper: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_device_attributes(
    _wrapper: *mut c_void,
    _req: u8,
) {
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_color_scheme(_wrapper: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_query_size(
    _wrapper: *mut c_void,
    _out: *mut GhosttySizeReportSize,
) -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_system_png_available() -> bool {
    false
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_system_decode_png(
    _data: *const u8,
    _len: usize,
    _out_ptr: *mut *mut u8,
    _out_len: *mut usize,
) -> i32 {
    -1
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_pwd_items(
    _terminal: *const c_void,
    _out_ptr: *mut *const u8,
    _out_len: *mut usize,
) {
}
