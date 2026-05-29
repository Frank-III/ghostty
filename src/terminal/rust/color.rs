use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::render::*;
use crate::selection::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use core::ffi::{c_int, c_void};
use core::{mem, ptr};

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_color_rgb_get(
    color: GhosttyColorRgb,
    r: *mut u8,
    g: *mut u8,
    b: *mut u8,
) {
    unsafe { color_rgb_get_impl(color, r, g, b) }
}

pub(crate) unsafe fn color_rgb_get_impl(
    color: GhosttyColorRgb,
    r: *mut u8,
    g: *mut u8,
    b: *mut u8,
) {
    unsafe {
        ptr::write(r, color.r);
        ptr::write(g, color.g);
        ptr::write(b, color.b);
    }
}
