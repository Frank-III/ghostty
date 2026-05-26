use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use crate::terminal_get_scalar::*;

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_get(
    has_terminal: bool,
    data: c_int,
    has_out: bool,
) -> c_int {
    if !has_terminal || !has_out {
        return GHOSTTY_INVALID_VALUE;
    }

    match data {
        TERMINAL_DATA_COLS
        | TERMINAL_DATA_ROWS
        | TERMINAL_DATA_CURSOR_X
        | TERMINAL_DATA_CURSOR_Y
        | TERMINAL_DATA_CURSOR_PENDING_WRAP
        | TERMINAL_DATA_ACTIVE_SCREEN
        | TERMINAL_DATA_CURSOR_VISIBLE
        | TERMINAL_DATA_KITTY_KEYBOARD_FLAGS
        | TERMINAL_DATA_SCROLLBAR
        | TERMINAL_DATA_CURSOR_STYLE
        | TERMINAL_DATA_MOUSE_TRACKING
        | TERMINAL_DATA_TITLE
        | TERMINAL_DATA_PWD
        | TERMINAL_DATA_TOTAL_ROWS
        | TERMINAL_DATA_SCROLLBACK_ROWS
        | TERMINAL_DATA_WIDTH_PX
        | TERMINAL_DATA_HEIGHT_PX
        | TERMINAL_DATA_COLOR_FOREGROUND
        | TERMINAL_DATA_COLOR_BACKGROUND
        | TERMINAL_DATA_COLOR_CURSOR
        | TERMINAL_DATA_COLOR_PALETTE
        | TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_CURSOR_DEFAULT
        | TERMINAL_DATA_COLOR_PALETTE_DEFAULT
        | TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM
        | TERMINAL_DATA_KITTY_GRAPHICS
        | TERMINAL_DATA_SELECTION => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_string(
    data: c_int,
    ptr: *const u8,
    len: usize,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_TITLE | TERMINAL_DATA_PWD => unsafe { write_borrowed_string(out, ptr, len) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_style(
    data: c_int,
    style: *const GhosttyStyle,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_CURSOR_STYLE => unsafe { copy_style(out.cast::<GhosttyStyle>(), style) },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scrollbar(
    data: c_int,
    total: u64,
    offset: u64,
    len: u64,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SCROLLBAR => unsafe {
            write_scrollbar(out.cast::<GhosttyTerminalScrollbar>(), total, offset, len)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_kitty_image(
    data: c_int,
    enabled: bool,
    storage_limit: u64,
    medium_file: bool,
    medium_temp_file: bool,
    medium_shared_mem: bool,
    out: *mut c_void,
) -> c_int {
    if !enabled {
        return GHOSTTY_NO_VALUE;
    }

    match data {
        TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT => unsafe { write_out(out, storage_limit) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE => unsafe { write_out(out, medium_file) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE => unsafe { write_out(out, medium_temp_file) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM => unsafe { write_out(out, medium_shared_mem) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_color(
    data: c_int,
    has_value: bool,
    r: u8,
    g: u8,
    b: u8,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_COLOR_FOREGROUND
        | TERMINAL_DATA_COLOR_BACKGROUND
        | TERMINAL_DATA_COLOR_CURSOR
        | TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_CURSOR_DEFAULT => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        write_rgb_value(out.cast::<GhosttyColorRgb>(), r, g, b);
    }
    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_palette(
    data: c_int,
    palette: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_COLOR_PALETTE | TERMINAL_DATA_COLOR_PALETTE_DEFAULT => unsafe {
            copy_palette(out.cast::<GhosttyColorRgb>(), palette)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_pointer(
    data: c_int,
    has_value: bool,
    value: *mut c_void,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_KITTY_GRAPHICS => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        ptr::write(out.cast::<*mut c_void>(), value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_selection(
    data: c_int,
    has_value: bool,
    selection: *const GhosttySelection,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SELECTION => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe { copy_selection(out.cast::<GhosttySelection>(), selection) }
}
