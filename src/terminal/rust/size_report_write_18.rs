use crate::constants::*;
use crate::simple::*;

pub(crate) unsafe fn write_size_report_18(size: GhosttySizeReportSize, out: *mut u8) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1b[8;");
        write_decimal(out, &mut offset, u64::from(size.rows));
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, u64::from(size.columns));
        write_bytes(out, &mut offset, b"t");
    }
}
