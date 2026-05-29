//! No-op Zig bridge symbols for cargo-linked `ghostty-vt` integration tests.

use core::ffi::{c_int, c_void};

use crate::allocator::GhosttyAllocator;
use crate::constants::GhosttySizeReportSize;
use crate::effects_wrapper::GhosttyVtEffectWrapper;
use crate::style::GhosttyColorRgb;
use crate::style::GhosttyStyle;

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_write_pty(
    wrapper: *mut c_void,
    ptr: *const u8,
    len: usize,
) {
    unsafe {
        if wrapper.is_null() || ptr.is_null() || len == 0 {
            return;
        }
        let data = core::slice::from_raw_parts(ptr, len);
        GhosttyVtEffectWrapper::dispatch_write_pty(wrapper, data);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_bell(wrapper: *mut c_void) {
    unsafe {
        GhosttyVtEffectWrapper::dispatch_bell(wrapper);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_title_changed(wrapper: *mut c_void) {
    unsafe {
        GhosttyVtEffectWrapper::dispatch_title_changed(wrapper);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_enquiry(wrapper: *mut c_void) {
    unsafe {
        GhosttyVtEffectWrapper::dispatch_report_enquiry(wrapper);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_xtversion(wrapper: *mut c_void) {
    unsafe {
        GhosttyVtEffectWrapper::dispatch_report_xtversion(wrapper);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_device_attributes(
    wrapper: *mut c_void,
    req: u8,
) {
    unsafe {
        GhosttyVtEffectWrapper::dispatch_report_device_attributes(wrapper, req);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_report_color_scheme(wrapper: *mut c_void) {
    unsafe {
        GhosttyVtEffectWrapper::dispatch_report_color_scheme(wrapper);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_query_size(
    wrapper: *mut c_void,
    out: *mut GhosttySizeReportSize,
) -> bool {
    unsafe {
        if out.is_null() {
            return false;
        }
        GhosttyVtEffectWrapper::dispatch_query_size(wrapper, &mut *out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_terminal_wrapper_clipboard_contents(
    wrapper: *mut c_void,
    kind: u8,
    ptr: *const u8,
    len: usize,
) {
    unsafe {
        if ptr.is_null() {
            return;
        }
        let data = core::slice::from_raw_parts(ptr, len);
        GhosttyVtEffectWrapper::dispatch_clipboard_contents(wrapper, kind, data);
    }
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

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_render_owned_begin(
    _state: *mut c_void,
    _alloc: *const GhosttyAllocator,
    _rows: u16,
    _cols: u16,
    _screen_key: u8,
    _dirty: c_int,
    _cursor_visual_style: u8,
    _cursor_visible: bool,
    _cursor_blinking: bool,
    _cursor_password_input: bool,
    _cursor_active_x: u16,
    _cursor_active_y: u16,
    _cursor_style: *const c_void,
    _cursor_cell: *const c_void,
    _viewport_pin: *const c_void,
    _background: GhosttyColorRgb,
    _foreground: GhosttyColorRgb,
    _cursor_present: bool,
    _cursor: GhosttyColorRgb,
    _palette: *const GhosttyColorRgb,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_render_owned_row(
    _state: *mut c_void,
    _alloc: *const GhosttyAllocator,
    _y: u16,
    _pin: *const c_void,
    _row: *const c_void,
    _cells: *const c_void,
    _cols: u16,
    _dirty: bool,
    _selection_present: bool,
    _selection_start: u16,
    _selection_end: u16,
    _is_cursor_row: bool,
    _cursor_x: u16,
    _cursor_wide_tail: bool,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_render_owned_cell_style(
    _state: *mut c_void,
    _y: u16,
    _x: u16,
    _style: *const GhosttyStyle,
) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_render_owned_end(
    _state: *mut c_void,
    _any_dirty: bool,
) -> c_int {
    0
}
