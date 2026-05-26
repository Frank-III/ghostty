use core::ffi::c_int;

use crate::constants::*;
use crate::size_report_write_14::*;
use crate::size_report_write_16::*;
use crate::size_report_write_18::*;
use crate::size_report_write_2048::*;

pub(crate) unsafe fn write_size_report(style: c_int, size: GhosttySizeReportSize, out: *mut u8) {
    match style {
        SIZE_REPORT_MODE_2048 => unsafe {
            write_size_report_2048(size, out);
        },
        SIZE_REPORT_CSI_14_T => unsafe {
            write_size_report_14(size, out);
        },
        SIZE_REPORT_CSI_16_T => unsafe {
            write_size_report_16(size, out);
        },
        SIZE_REPORT_CSI_18_T => unsafe {
            write_size_report_18(size, out);
        },
        _ => {}
    }
}
