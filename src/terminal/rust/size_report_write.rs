use core::ffi::c_int;

use crate::constants::*;
use crate::size_report_pixels::*;
use crate::simple::*;

pub(crate) fn size_report_len(style: c_int, size: GhosttySizeReportSize) -> Option<usize> {
    let rows = u64::from(size.rows);
    let columns = u64::from(size.columns);
    let height = height_pixels(size);
    let width = width_pixels(size);

    match style {
        SIZE_REPORT_MODE_2048 => Some(
            b"\x1B[48;".len()
                + decimal_len(rows)
                + 1
                + decimal_len(columns)
                + 1
                + decimal_len(height)
                + 1
                + decimal_len(width)
                + 1,
        ),
        SIZE_REPORT_CSI_14_T => {
            Some(b"\x1b[4;".len() + decimal_len(height) + 1 + decimal_len(width) + 1)
        }
        SIZE_REPORT_CSI_16_T => Some(
            b"\x1b[6;".len()
                + decimal_len(u64::from(size.cell_height))
                + 1
                + decimal_len(u64::from(size.cell_width))
                + 1,
        ),
        SIZE_REPORT_CSI_18_T => {
            Some(b"\x1b[8;".len() + decimal_len(rows) + 1 + decimal_len(columns) + 1)
        }
        _ => None,
    }
}

pub(crate) unsafe fn write_size_report(style: c_int, size: GhosttySizeReportSize, out: *mut u8) {
    let mut offset = 0usize;
    let rows = u64::from(size.rows);
    let columns = u64::from(size.columns);

    match style {
        SIZE_REPORT_MODE_2048 => unsafe {
            write_bytes(out, &mut offset, b"\x1B[48;");
            write_decimal(out, &mut offset, rows);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, columns);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, height_pixels(size));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, width_pixels(size));
            write_bytes(out, &mut offset, b"t");
        },
        SIZE_REPORT_CSI_14_T => unsafe {
            write_bytes(out, &mut offset, b"\x1b[4;");
            write_decimal(out, &mut offset, height_pixels(size));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, width_pixels(size));
            write_bytes(out, &mut offset, b"t");
        },
        SIZE_REPORT_CSI_16_T => unsafe {
            write_bytes(out, &mut offset, b"\x1b[6;");
            write_decimal(out, &mut offset, u64::from(size.cell_height));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(size.cell_width));
            write_bytes(out, &mut offset, b"t");
        },
        SIZE_REPORT_CSI_18_T => unsafe {
            write_bytes(out, &mut offset, b"\x1b[8;");
            write_decimal(out, &mut offset, rows);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, columns);
            write_bytes(out, &mut offset, b"t");
        },
        _ => {}
    }
}
