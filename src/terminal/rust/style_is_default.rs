use core::{mem, ptr};

use crate::constants::*;
use crate::style::*;

pub(crate) unsafe fn style_is_default(style: *const GhosttyStyle) -> bool {
    unsafe {
        ptr::read(core::ptr::addr_of!((*style).size)) == mem::size_of::<GhosttyStyle>()
            && ptr::read(core::ptr::addr_of!((*style).fg_color.tag)) == STYLE_COLOR_NONE
            && ptr::read(core::ptr::addr_of!((*style).bg_color.tag)) == STYLE_COLOR_NONE
            && ptr::read(core::ptr::addr_of!((*style).underline_color.tag)) == STYLE_COLOR_NONE
            && !ptr::read(core::ptr::addr_of!((*style).bold))
            && !ptr::read(core::ptr::addr_of!((*style).italic))
            && !ptr::read(core::ptr::addr_of!((*style).faint))
            && !ptr::read(core::ptr::addr_of!((*style).blink))
            && !ptr::read(core::ptr::addr_of!((*style).inverse))
            && !ptr::read(core::ptr::addr_of!((*style).invisible))
            && !ptr::read(core::ptr::addr_of!((*style).strikethrough))
            && !ptr::read(core::ptr::addr_of!((*style).overline))
            && ptr::read(core::ptr::addr_of!((*style).underline)) == 0
    }
}
