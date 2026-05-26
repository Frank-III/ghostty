use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::style::*;
use crate::style_write::*;

pub(crate) unsafe fn copy_style_color(
    dst: *mut GhosttyStyleColor,
    src: *const GhosttyStyleColor,
) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        let tag = ptr::read(core::ptr::addr_of!((*src).tag));
        ptr::write(core::ptr::addr_of_mut!((*dst).tag), tag);
        ptr::write(core::ptr::addr_of_mut!((*dst).value.padding), 0);

        match tag {
            STYLE_COLOR_NONE => {}
            STYLE_COLOR_PALETTE => {
                ptr::write(
                    core::ptr::addr_of_mut!((*dst).value.palette),
                    ptr::read(core::ptr::addr_of!((*src).value.palette)),
                );
            }
            STYLE_COLOR_RGB => {
                write_rgb(
                    core::ptr::addr_of_mut!((*dst).value.rgb).cast::<core::ffi::c_void>(),
                    core::ptr::addr_of!((*src).value.rgb),
                );
            }
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}
