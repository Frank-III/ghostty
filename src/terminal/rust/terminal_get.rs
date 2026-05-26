use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;

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
pub unsafe extern "C" fn ghostty_rust_terminal_get_scalar(
    data: c_int,
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_pending_wrap: bool,
    active_screen: c_int,
    cursor_visible: bool,
    kitty_keyboard_flags: u8,
    mouse_tracking: bool,
    total_rows: usize,
    scrollback_rows: usize,
    width_px: u32,
    height_px: u32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    match data {
        TERMINAL_DATA_COLS => unsafe { write_out(out, cols) },
        TERMINAL_DATA_ROWS => unsafe { write_out(out, rows) },
        TERMINAL_DATA_CURSOR_X => unsafe { write_out(out, cursor_x) },
        TERMINAL_DATA_CURSOR_Y => unsafe { write_out(out, cursor_y) },
        TERMINAL_DATA_CURSOR_PENDING_WRAP => unsafe { write_out(out, cursor_pending_wrap) },
        TERMINAL_DATA_ACTIVE_SCREEN => unsafe { write_out(out, active_screen) },
        TERMINAL_DATA_CURSOR_VISIBLE => unsafe { write_out(out, cursor_visible) },
        TERMINAL_DATA_KITTY_KEYBOARD_FLAGS => unsafe { write_out(out, kitty_keyboard_flags) },
        TERMINAL_DATA_MOUSE_TRACKING => unsafe { write_out(out, mouse_tracking) },
        TERMINAL_DATA_TOTAL_ROWS => unsafe { write_out(out, total_rows) },
        TERMINAL_DATA_SCROLLBACK_ROWS => unsafe { write_out(out, scrollback_rows) },
        TERMINAL_DATA_WIDTH_PX => unsafe { write_out(out, width_px) },
        TERMINAL_DATA_HEIGHT_PX => unsafe { write_out(out, height_px) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scalar_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_pending_wrap: bool,
    active_screen: c_int,
    cursor_visible: bool,
    kitty_keyboard_flags: u8,
    mouse_tracking: bool,
    total_rows: usize,
    scrollback_rows: usize,
    width_px: u32,
    height_px: u32,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe {
            ghostty_rust_terminal_get_scalar(
                key,
                cols,
                rows,
                cursor_x,
                cursor_y,
                cursor_pending_wrap,
                active_screen,
                cursor_visible,
                kitty_keyboard_flags,
                mouse_tracking,
                total_rows,
                scrollback_rows,
                width_px,
                height_px,
                out,
            )
        };

        if result != GHOSTTY_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, i);
                }
            }
            return result;
        }

        i += 1;
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    GHOSTTY_SUCCESS
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
