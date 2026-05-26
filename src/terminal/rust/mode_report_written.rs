use core::ptr;

pub(crate) unsafe fn mode_report_written(out_written: *mut usize, total: usize) {
    unsafe {
        ptr::write(out_written, total);
    }
}
