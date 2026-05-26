use core::ptr;

use crate::selection::*;

pub(crate) unsafe fn write_scrollbar(
    out: *mut GhosttyTerminalScrollbar,
    total: u64,
    offset: u64,
    len: u64,
) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).total), total);
        ptr::write(core::ptr::addr_of_mut!((*out).offset), offset);
        ptr::write(core::ptr::addr_of_mut!((*out).len), len);
    }
}
