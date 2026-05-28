use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;

pub(crate) const GHOSTTY_SUCCESS: c_int = 0;
pub(crate) const GHOSTTY_OUT_OF_MEMORY: c_int = -1;
pub(crate) const GHOSTTY_INVALID_VALUE: c_int = -2;
pub(crate) const GHOSTTY_OUT_OF_SPACE: c_int = -3;
pub(crate) const GHOSTTY_NO_VALUE: c_int = -4;

pub(crate) const GHOSTTY_FOCUS_LOST: c_int = 1;
