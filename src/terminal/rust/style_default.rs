use core::{mem, ptr};

use crate::style::*;
use crate::style_color_none::*;

pub(crate) unsafe fn write_style_default(result: *mut GhosttyStyle) {
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*result).size),
            mem::size_of::<GhosttyStyle>(),
        );
        write_style_color_none(core::ptr::addr_of_mut!((*result).fg_color));
        write_style_color_none(core::ptr::addr_of_mut!((*result).bg_color));
        write_style_color_none(core::ptr::addr_of_mut!((*result).underline_color));
        ptr::write(core::ptr::addr_of_mut!((*result).bold), false);
        ptr::write(core::ptr::addr_of_mut!((*result).italic), false);
        ptr::write(core::ptr::addr_of_mut!((*result).faint), false);
        ptr::write(core::ptr::addr_of_mut!((*result).blink), false);
        ptr::write(core::ptr::addr_of_mut!((*result).inverse), false);
        ptr::write(core::ptr::addr_of_mut!((*result).invisible), false);
        ptr::write(core::ptr::addr_of_mut!((*result).strikethrough), false);
        ptr::write(core::ptr::addr_of_mut!((*result).overline), false);
        ptr::write(core::ptr::addr_of_mut!((*result).underline), 0);
    }
}
