//! No-op Zig bridge symbols for cargo-linked `ghostty-vt` integration tests.

use core::ffi::{c_int, c_void};

use crate::allocator::GhosttyAllocator;
use crate::constants::GhosttySizeReportSize;
use crate::style::GhosttyColorRgb;
use crate::style::GhosttyStyle;

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
