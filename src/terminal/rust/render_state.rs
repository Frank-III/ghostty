use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::render_state_primitive::*;
use crate::render_state_color::*;
use crate::style::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    cols: u16,
    rows: u16,
    dirty: c_int,
    cursor_visual_style: c_int,
    cursor_visible: bool,
    cursor_blinking: bool,
    cursor_password_input: bool,
    cursor_viewport_has_value: bool,
    cursor_viewport_x: u16,
    cursor_viewport_y: u16,
    cursor_viewport_wide_tail: bool,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor_present: bool,
    cursor: GhosttyColorRgb,
    palette: *const GhosttyColorRgb,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe {
            match key {
                RENDER_DATA_COLS
                | RENDER_DATA_ROWS
                | RENDER_DATA_DIRTY
                | RENDER_DATA_CURSOR_VISUAL_STYLE
                | RENDER_DATA_CURSOR_VISIBLE
                | RENDER_DATA_CURSOR_BLINKING
                | RENDER_DATA_CURSOR_PASSWORD_INPUT
                | RENDER_DATA_CURSOR_VIEWPORT_HAS_VALUE
                | RENDER_DATA_CURSOR_VIEWPORT_X
                | RENDER_DATA_CURSOR_VIEWPORT_Y
                | RENDER_DATA_CURSOR_VIEWPORT_WIDE_TAIL => ghostty_rust_render_state_get_primitive(
                    key,
                    cols,
                    rows,
                    dirty,
                    cursor_visual_style,
                    cursor_visible,
                    cursor_blinking,
                    cursor_password_input,
                    cursor_viewport_has_value,
                    cursor_viewport_x,
                    cursor_viewport_y,
                    cursor_viewport_wide_tail,
                    out,
                ),
                RENDER_DATA_COLOR_BACKGROUND
                | RENDER_DATA_COLOR_FOREGROUND
                | RENDER_DATA_COLOR_CURSOR
                | RENDER_DATA_COLOR_CURSOR_HAS_VALUE
                | RENDER_DATA_COLOR_PALETTE => ghostty_rust_render_state_get_color(
                    key,
                    background,
                    foreground,
                    cursor_present,
                    cursor,
                    palette,
                    out,
                ),
                _ => RENDER_RESULT_INVALID_VALUE,
            }
        };

        if result != RENDER_RESULT_SUCCESS {
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

    RENDER_RESULT_SUCCESS
}
