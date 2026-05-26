use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::style::*;
use crate::style_attrs_copy::*;
use crate::style_color::*;
use crate::style_color_copy::*;
use crate::style_write::*;
use crate::scrollbar_write::*;

pub(crate) use crate::scrollbar_write::*;
pub(crate) use crate::style_attrs_copy::*;
pub(crate) use crate::style_color::*;
pub(crate) use crate::style_color_copy::*;

pub(crate) unsafe fn copy_style(dst: *mut GhosttyStyle, src: *const GhosttyStyle) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).fg_color),
            core::ptr::addr_of!((*src).fg_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).bg_color),
            core::ptr::addr_of!((*src).bg_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).underline_color),
            core::ptr::addr_of!((*src).underline_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        copy_style_attrs(dst, src);
    }

    GHOSTTY_SUCCESS
}
