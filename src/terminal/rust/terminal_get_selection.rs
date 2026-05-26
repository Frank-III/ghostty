use core::ffi::{c_int, c_void};

use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::selection_copy::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_selection(
    data: c_int,
    has_value: bool,
    selection: *const GhosttySelection,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SELECTION => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe { copy_selection(out.cast::<GhosttySelection>(), selection) }
}
