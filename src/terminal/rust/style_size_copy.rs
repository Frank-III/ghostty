use core::ptr;

use crate::style::*;

pub(crate) unsafe fn copy_style_size(dst: *mut GhosttyStyle, src: *const GhosttyStyle) {
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );
    }
}
