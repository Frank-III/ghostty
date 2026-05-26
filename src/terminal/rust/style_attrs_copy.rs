use core::ptr;

use crate::style::*;

pub(crate) unsafe fn copy_style_attrs(dst: *mut GhosttyStyle, src: *const GhosttyStyle) {
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).bold),
            ptr::read(core::ptr::addr_of!((*src).bold)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).italic),
            ptr::read(core::ptr::addr_of!((*src).italic)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).faint),
            ptr::read(core::ptr::addr_of!((*src).faint)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).blink),
            ptr::read(core::ptr::addr_of!((*src).blink)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).inverse),
            ptr::read(core::ptr::addr_of!((*src).inverse)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).invisible),
            ptr::read(core::ptr::addr_of!((*src).invisible)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).strikethrough),
            ptr::read(core::ptr::addr_of!((*src).strikethrough)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).overline),
            ptr::read(core::ptr::addr_of!((*src).overline)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).underline),
            ptr::read(core::ptr::addr_of!((*src).underline)),
        );
    }
}
