use crate::simple::*;

pub(crate) unsafe fn write_mode_report(out: *mut u8, value: u64, ansi: bool, state: u64) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1B[");
        if !ansi {
            write_bytes(out, &mut offset, b"?");
        }
        write_decimal(out, &mut offset, value);
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, state);
        write_bytes(out, &mut offset, b"$y");
    }
}
