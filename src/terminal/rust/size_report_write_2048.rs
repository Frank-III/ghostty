use crate::constants::*;
use crate::size_report_pixels::*;
use crate::simple::*;

pub(crate) unsafe fn write_size_report_2048(size: GhosttySizeReportSize, out: *mut u8) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1B[48;");
        write_decimal(out, &mut offset, u64::from(size.rows));
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, u64::from(size.columns));
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, height_pixels(size));
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, width_pixels(size));
        write_bytes(out, &mut offset, b"t");
    }
}
