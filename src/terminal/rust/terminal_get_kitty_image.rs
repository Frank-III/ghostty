use core::ffi::{c_int, c_void};

use crate::constants::*;
use crate::early::*;
use crate::simple::*;
use crate::terminal::*;
use crate::style_write::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_kitty_image(
    data: c_int,
    enabled: bool,
    storage_limit: u64,
    medium_file: bool,
    medium_temp_file: bool,
    medium_shared_mem: bool,
    out: *mut c_void,
) -> c_int {
    if !enabled {
        return GHOSTTY_NO_VALUE;
    }

    match data {
        TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT => unsafe { write_out(out, storage_limit) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE => unsafe { write_out(out, medium_file) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE => unsafe { write_out(out, medium_temp_file) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM => unsafe { write_out(out, medium_shared_mem) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}
