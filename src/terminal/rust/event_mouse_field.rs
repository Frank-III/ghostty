use core::ffi::c_void;

use crate::constants::*;

pub(crate) unsafe fn mouse_event_field<T>(event: *mut c_void, offset: usize) -> *mut T {
    unsafe {
        event
            .cast::<u8>()
            .add(MOUSE_EVENT_EVENT_OFFSET + offset)
            .cast::<T>()
    }
}
