use core::ptr;

pub(crate) unsafe fn write_focus_sequence(out: *mut u8, seq: &[u8; 3]) {
    unsafe {
        ptr::copy_nonoverlapping(seq.as_ptr(), out, seq.len());
    }
}
