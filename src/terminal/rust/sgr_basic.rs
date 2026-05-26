use core::ffi::c_int;

use crate::sgr_constants::*;

pub(crate) fn basic_sgr_tag(first: u16) -> Option<c_int> {
    match first {
        0 => Some(SGR_UNSET),
        1 => Some(SGR_BOLD),
        2 => Some(SGR_FAINT),
        3 => Some(SGR_ITALIC),
        5 | 6 => Some(SGR_BLINK),
        7 => Some(SGR_INVERSE),
        8 => Some(SGR_INVISIBLE),
        9 => Some(SGR_STRIKETHROUGH),
        21 => Some(SGR_UNDERLINE),
        22 => Some(SGR_RESET_BOLD),
        23 => Some(SGR_RESET_ITALIC),
        25 => Some(SGR_RESET_BLINK),
        27 => Some(SGR_RESET_INVERSE),
        28 => Some(SGR_RESET_INVISIBLE),
        29 => Some(SGR_RESET_STRIKETHROUGH),
        39 => Some(SGR_RESET_FG),
        49 => Some(SGR_RESET_BG),
        53 => Some(SGR_OVERLINE),
        55 => Some(SGR_RESET_OVERLINE),
        59 => Some(SGR_RESET_UNDERLINE_COLOR),
        _ => None,
    }
}
