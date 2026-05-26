use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;

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
    unsafe {
        terminal_get_scalar_impl(
            data,
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
    }
}

pub(crate) unsafe fn terminal_get_scalar_impl(
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
    unsafe {
        terminal_get_scalar_multi_impl(
            count,
            keys,
            values,
            out_written,
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
        )
    }
}

pub(crate) unsafe fn terminal_get_scalar_multi_impl(
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
            terminal_get_scalar_impl(
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
