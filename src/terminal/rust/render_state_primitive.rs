use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_get_primitive(
    data: c_int,
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
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_DATA_COLS => ptr::write(out.cast::<u16>(), cols),
            RENDER_DATA_ROWS => ptr::write(out.cast::<u16>(), rows),
            RENDER_DATA_DIRTY => ptr::write(out.cast::<c_int>(), dirty),
            RENDER_DATA_CURSOR_VISUAL_STYLE => {
                let style = match cursor_visual_style {
                    RENDER_CURSOR_STYLE_BAR
                    | RENDER_CURSOR_STYLE_BLOCK
                    | RENDER_CURSOR_STYLE_UNDERLINE
                    | RENDER_CURSOR_STYLE_BLOCK_HOLLOW => cursor_visual_style,
                    _ => return RENDER_RESULT_INVALID_VALUE,
                };
                ptr::write(out.cast::<c_int>(), style);
            }
            RENDER_DATA_CURSOR_VISIBLE => ptr::write(out.cast::<bool>(), cursor_visible),
            RENDER_DATA_CURSOR_BLINKING => ptr::write(out.cast::<bool>(), cursor_blinking),
            RENDER_DATA_CURSOR_PASSWORD_INPUT => {
                ptr::write(out.cast::<bool>(), cursor_password_input);
            }
            RENDER_DATA_CURSOR_VIEWPORT_HAS_VALUE => {
                ptr::write(out.cast::<bool>(), cursor_viewport_has_value);
            }
            RENDER_DATA_CURSOR_VIEWPORT_X => {
                if !cursor_viewport_has_value {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                ptr::write(out.cast::<u16>(), cursor_viewport_x);
            }
            RENDER_DATA_CURSOR_VIEWPORT_Y => {
                if !cursor_viewport_has_value {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                ptr::write(out.cast::<u16>(), cursor_viewport_y);
            }
            RENDER_DATA_CURSOR_VIEWPORT_WIDE_TAIL => {
                if !cursor_viewport_has_value {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                ptr::write(out.cast::<bool>(), cursor_viewport_wide_tail);
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_set_dirty(
    value: c_int,
    out: *mut c_int,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_state_set(
    has_state: bool,
    option: c_int,
    has_value: bool,
) -> c_int {
    if !has_state || !has_value {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match option {
        RENDER_STATE_SET_DIRTY => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}
