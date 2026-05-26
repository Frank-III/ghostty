use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::mouse_setopt::*;
use crate::simple::*;
use crate::style::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_event(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_mode(value, current, out, last_cell_present);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_format(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_mode(value, current, out, last_cell_present);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_from_terminal(
    event: c_int,
    format: c_int,
    out_event: *mut c_int,
    out_format: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        if !out_event.is_null() {
            ptr::write(out_event, event);
        }
        if !out_format.is_null() {
            ptr::write(out_format, format);
        }
        if !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_reset(last_cell_present: *mut bool) {
    if last_cell_present.is_null() {
        return;
    }

    unsafe {
        ptr::write(last_cell_present, false);
    }
}
