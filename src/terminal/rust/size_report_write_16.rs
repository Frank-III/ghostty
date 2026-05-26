use crate::constants::*;
use crate::simple::*;

pub(crate) unsafe fn write_size_report_16(size: GhosttySizeReportSize, out: *mut u8) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1b[6;");
        write_decimal(out, &mut offset, u64::from(size.cell_height));
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, u64::from(size.cell_width));
        write_bytes(out, &mut offset, b"t");
    }
}
