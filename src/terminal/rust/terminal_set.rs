use core::ffi::c_int;

use crate::constants::*;
use crate::early::*;
use crate::terminal::*;
use crate::terminal_options::*;

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_set(has_terminal: bool, option: c_int) -> c_int {
    terminal_set_impl(has_terminal, option)
}

pub(crate) fn terminal_set_impl(has_terminal: bool, option: c_int) -> c_int {
    if !has_terminal {
        return GHOSTTY_INVALID_VALUE;
    }

    match option {
        TERMINAL_OPT_USERDATA
        | TERMINAL_OPT_WRITE_PTY
        | TERMINAL_OPT_BELL
        | TERMINAL_OPT_ENQUIRY
        | TERMINAL_OPT_XTVERSION
        | TERMINAL_OPT_TITLE_CHANGED
        | TERMINAL_OPT_SIZE
        | TERMINAL_OPT_COLOR_SCHEME
        | TERMINAL_OPT_DEVICE_ATTRIBUTES
        | TERMINAL_OPT_TITLE
        | TERMINAL_OPT_PWD
        | TERMINAL_OPT_COLOR_FOREGROUND
        | TERMINAL_OPT_COLOR_BACKGROUND
        | TERMINAL_OPT_COLOR_CURSOR
        | TERMINAL_OPT_COLOR_PALETTE
        | TERMINAL_OPT_KITTY_IMAGE_STORAGE_LIMIT
        | TERMINAL_OPT_KITTY_IMAGE_MEDIUM_FILE
        | TERMINAL_OPT_KITTY_IMAGE_MEDIUM_TEMP_FILE
        | TERMINAL_OPT_KITTY_IMAGE_MEDIUM_SHARED_MEM
        | TERMINAL_OPT_APC_MAX_BYTES
        | TERMINAL_OPT_APC_MAX_BYTES_KITTY
        | TERMINAL_OPT_SELECTION => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}
