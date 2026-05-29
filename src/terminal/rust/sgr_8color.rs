use core::ffi::c_int;

use crate::sgr_attr::*;
use crate::sgr_constants::*;
use crate::sgr_write::*;

pub(crate) unsafe fn write_sgr_8_color(result: *mut GhosttySgrAttribute, first: u16) -> bool {
    unsafe {
        if first >= 30 && first <= 37 {
            write_sgr_c_int(result, SGR_8_FG, c_int::from(first - 30));
            return true;
        }
        if first >= 40 && first <= 47 {
            write_sgr_c_int(result, SGR_8_BG, c_int::from(first - 40));
            return true;
        }
        if first >= 90 && first <= 97 {
            write_sgr_c_int(result, SGR_8_BRIGHT_FG, c_int::from(first - 82));
            return true;
        }
        if first >= 100 && first <= 107 {
            write_sgr_c_int(result, SGR_8_BRIGHT_BG, c_int::from(first - 92));
            return true;
        }
    }

    false
}
