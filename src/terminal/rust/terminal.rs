use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_get(
    has_terminal: bool,
    data: c_int,
    has_out: bool,
) -> c_int {
    if !has_terminal || !has_out {
        return GHOSTTY_INVALID_VALUE;
    }

    match data {
        TERMINAL_DATA_COLS
        | TERMINAL_DATA_ROWS
        | TERMINAL_DATA_CURSOR_X
        | TERMINAL_DATA_CURSOR_Y
        | TERMINAL_DATA_CURSOR_PENDING_WRAP
        | TERMINAL_DATA_ACTIVE_SCREEN
        | TERMINAL_DATA_CURSOR_VISIBLE
        | TERMINAL_DATA_KITTY_KEYBOARD_FLAGS
        | TERMINAL_DATA_SCROLLBAR
        | TERMINAL_DATA_CURSOR_STYLE
        | TERMINAL_DATA_MOUSE_TRACKING
        | TERMINAL_DATA_TITLE
        | TERMINAL_DATA_PWD
        | TERMINAL_DATA_TOTAL_ROWS
        | TERMINAL_DATA_SCROLLBACK_ROWS
        | TERMINAL_DATA_WIDTH_PX
        | TERMINAL_DATA_HEIGHT_PX
        | TERMINAL_DATA_COLOR_FOREGROUND
        | TERMINAL_DATA_COLOR_BACKGROUND
        | TERMINAL_DATA_COLOR_CURSOR
        | TERMINAL_DATA_COLOR_PALETTE
        | TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_CURSOR_DEFAULT
        | TERMINAL_DATA_COLOR_PALETTE_DEFAULT
        | TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM
        | TERMINAL_DATA_KITTY_GRAPHICS
        | TERMINAL_DATA_SELECTION => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}
