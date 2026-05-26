use core::ffi::{c_int, c_void};
use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::selection_copy::*;
use crate::simple::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_style(
    data: c_int,
    style: *const GhosttyStyle,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_CURSOR_STYLE => unsafe { copy_style(out.cast::<GhosttyStyle>(), style) },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scrollbar(
    data: c_int,
    total: u64,
    offset: u64,
    len: u64,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SCROLLBAR => unsafe {
            write_scrollbar(out.cast::<GhosttyTerminalScrollbar>(), total, offset, len)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}
