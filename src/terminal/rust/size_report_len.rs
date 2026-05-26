use core::ffi::c_int;

use crate::constants::*;
use crate::simple::*;
use crate::size_report_pixels::*;

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
