use core::ffi::c_void;
use core::ptr;

pub(crate) unsafe fn osc_command_data_write_string(value: *const u8, out: *mut c_void) {
    unsafe {
        ptr::write(out.cast::<*const u8>(), value);
    }
}
