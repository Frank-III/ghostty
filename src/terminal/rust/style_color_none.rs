use core::ptr;

use crate::constants::*;
use crate::style::*;

pub(crate) unsafe fn write_style_color_none(color: *mut GhosttyStyleColor) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*color).tag), STYLE_COLOR_NONE);
        ptr::write(core::ptr::addr_of_mut!((*color).value.padding), 0);
    }
}
