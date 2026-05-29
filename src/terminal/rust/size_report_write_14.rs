use crate::constants::*;
use crate::simple::*;
use crate::size_report_pixels::*;

pub(crate) unsafe fn write_size_report_14(size: GhosttySizeReportSize, out: *mut u8) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1b[4;");
        write_decimal(out, &mut offset, height_pixels(size));
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, width_pixels(size));
        write_bytes(out, &mut offset, b"t");
    }
}
