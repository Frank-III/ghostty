use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyMousePosition {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct GhosttyMouseSize {
    pub(crate) screen_width: u32,
    pub(crate) screen_height: u32,
    pub(crate) cell_width: u32,
    pub(crate) cell_height: u32,
    pub(crate) padding_top: u32,
    pub(crate) padding_bottom: u32,
    pub(crate) padding_right: u32,
    pub(crate) padding_left: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct GhosttyMouseCell {
    pub(crate) x: u16,
    pub(crate) y: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct GhosttyMousePixels {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

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

pub(crate) unsafe fn mouse_encoder_setopt_mode(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    if out.is_null() {
        return;
    }

    unsafe {
        if value != current && !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
        ptr::write(out, value);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_size(
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
    out_screen_width: *mut u32,
    out_screen_height: *mut u32,
    out_cell_width: *mut u32,
    out_cell_height: *mut u32,
    out_padding_top: *mut u32,
    out_padding_bottom: *mut u32,
    out_padding_right: *mut u32,
    out_padding_left: *mut u32,
    last_cell_present: *mut bool,
) {
    unsafe {
        if !out_screen_width.is_null() {
            ptr::write(out_screen_width, screen_width);
        }
        if !out_screen_height.is_null() {
            ptr::write(out_screen_height, screen_height);
        }
        if !out_cell_width.is_null() {
            ptr::write(out_cell_width, cell_width);
        }
        if !out_cell_height.is_null() {
            ptr::write(out_cell_height, cell_height);
        }
        if !out_padding_top.is_null() {
            ptr::write(out_padding_top, padding_top);
        }
        if !out_padding_bottom.is_null() {
            ptr::write(out_padding_bottom, padding_bottom);
        }
        if !out_padding_right.is_null() {
            ptr::write(out_padding_right, padding_right);
        }
        if !out_padding_left.is_null() {
            ptr::write(out_padding_left, padding_left);
        }
        if !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_bool(
    option: c_int,
    value: bool,
    any_button_pressed: *mut bool,
    track_last_cell: *mut bool,
    last_cell_present: *mut bool,
) {
    unsafe {
        match option {
            MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED => {
                if !any_button_pressed.is_null() {
                    ptr::write(any_button_pressed, value);
                }
            }
            MOUSE_ENCODER_OPT_TRACK_LAST_CELL => {
                if !track_last_cell.is_null() {
                    ptr::write(track_last_cell, value);
                }
                if !value && !last_cell_present.is_null() {
                    ptr::write(last_cell_present, false);
                }
            }
            _ => {}
        }
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

