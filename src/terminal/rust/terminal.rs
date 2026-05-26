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
use crate::style::*;

pub(crate) const TERMINAL_OPT_USERDATA: c_int = 0;
pub(crate) const TERMINAL_OPT_WRITE_PTY: c_int = 1;
pub(crate) const TERMINAL_OPT_BELL: c_int = 2;
pub(crate) const TERMINAL_OPT_ENQUIRY: c_int = 3;
pub(crate) const TERMINAL_OPT_XTVERSION: c_int = 4;
pub(crate) const TERMINAL_OPT_TITLE_CHANGED: c_int = 5;
pub(crate) const TERMINAL_OPT_SIZE: c_int = 6;
pub(crate) const TERMINAL_OPT_COLOR_SCHEME: c_int = 7;
pub(crate) const TERMINAL_OPT_DEVICE_ATTRIBUTES: c_int = 8;
pub(crate) const TERMINAL_OPT_TITLE: c_int = 9;
pub(crate) const TERMINAL_OPT_PWD: c_int = 10;
pub(crate) const TERMINAL_OPT_COLOR_FOREGROUND: c_int = 11;
pub(crate) const TERMINAL_OPT_COLOR_BACKGROUND: c_int = 12;
pub(crate) const TERMINAL_OPT_COLOR_CURSOR: c_int = 13;
pub(crate) const TERMINAL_OPT_COLOR_PALETTE: c_int = 14;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_STORAGE_LIMIT: c_int = 15;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_FILE: c_int = 16;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_TEMP_FILE: c_int = 17;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_SHARED_MEM: c_int = 18;
pub(crate) const TERMINAL_OPT_APC_MAX_BYTES: c_int = 19;
pub(crate) const TERMINAL_OPT_APC_MAX_BYTES_KITTY: c_int = 20;
pub(crate) const TERMINAL_OPT_SELECTION: c_int = 21;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPointCoordinate {
    pub(crate) x: u16,
    pub(crate) y: u32,
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_new(cols: u16, rows: u16) -> c_int {
    if cols == 0 || rows == 0 {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_reset(has_terminal: bool) -> bool {
    has_terminal
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_point_from_grid_ref(
    has_point: bool,
    coord: GhosttyPointCoordinate,
    out: *mut GhosttyPointCoordinate,
) -> c_int {
    if !has_point {
        return GHOSTTY_NO_VALUE;
    }

    if !out.is_null() {
        unsafe {
            ptr::write(out, coord);
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_point_from_grid_ref_input(
    has_terminal: bool,
    has_ref: bool,
) -> c_int {
    if !has_terminal || !has_ref {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_grid_ref(
    has_pin: bool,
    node: *mut c_void,
    x: u16,
    y: u16,
    out_ref: *mut GhosttyGridRef,
) -> c_int {
    if !has_pin {
        return GHOSTTY_INVALID_VALUE;
    }

    if !out_ref.is_null() {
        unsafe {
            ptr::write(
                core::ptr::addr_of_mut!((*out_ref).size),
                mem::size_of::<GhosttyGridRef>(),
            );
            ptr::write(core::ptr::addr_of_mut!((*out_ref).node), node);
            ptr::write(core::ptr::addr_of_mut!((*out_ref).x), x);
            ptr::write(core::ptr::addr_of_mut!((*out_ref).y), y);
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_grid_ref_track_input(
    has_terminal: bool,
    has_out: bool,
) -> c_int {
    if !has_terminal || !has_out {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_mode_get(
    has_terminal: bool,
    has_mode: bool,
    value: bool,
    out: *mut bool,
) -> c_int {
    if !has_terminal || !has_mode {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_mode_set(has_terminal: bool, has_mode: bool) -> c_int {
    if !has_terminal || !has_mode {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_resize(
    has_terminal: bool,
    cols: u16,
    rows: u16,
    cell_width_px: u32,
    cell_height_px: u32,
    out_width_px: *mut u32,
    out_height_px: *mut u32,
) -> c_int {
    if !has_terminal || cols == 0 || rows == 0 || out_width_px.is_null() || out_height_px.is_null()
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let width = (u64::from(cols) * u64::from(cell_width_px)).min(u64::from(u32::MAX)) as u32;
    let height = (u64::from(rows) * u64::from(cell_height_px)).min(u64::from(u32::MAX)) as u32;

    unsafe {
        ptr::write(out_width_px, width);
        ptr::write(out_height_px, height);
    }

    GHOSTTY_SUCCESS
}
