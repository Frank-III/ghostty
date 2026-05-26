use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::style::*;
use crate::style_write::*;

pub(crate) unsafe fn write_style_color_rgb(
    color: *const GhosttyStyleColor,
    palette_color: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    if color.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match ptr::read(core::ptr::addr_of!((*color).tag)) {
            STYLE_COLOR_NONE => GHOSTTY_INVALID_VALUE,
            STYLE_COLOR_PALETTE => {
                write_rgb(out, palette_color);
                GHOSTTY_SUCCESS
            }
            STYLE_COLOR_RGB => {
                let rgb = core::ptr::addr_of!((*color).value.rgb);
                write_rgb(out, rgb);
                GHOSTTY_SUCCESS
            }
            _ => GHOSTTY_INVALID_VALUE,
        }
    }
}
