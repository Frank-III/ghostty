use core::ffi::c_void;

pub(crate) unsafe fn key_event_field<T>(event: *mut c_void, offset: usize) -> *mut T {
    unsafe { event.cast::<u8>().add(offset).cast::<T>() }
}
