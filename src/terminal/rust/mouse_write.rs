use core::ptr;

pub(crate) use crate::mouse_sequence_len::*;
pub(crate) use crate::mouse_sequence_write::*;

pub(crate) unsafe fn mouse_write_ptr<T>(out: *mut T, value: T) {
    unsafe {
        ptr::write(out, value);
    }
}

pub(crate) unsafe fn mouse_write_ptr_if_present<T>(out: *mut T, value: T) {
    if out.is_null() {
        return;
    }

    unsafe {
        mouse_write_ptr(out, value);
    }
}
