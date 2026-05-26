use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_color_rgb_get(
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
