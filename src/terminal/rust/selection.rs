use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::render::*;
use crate::selection_copy::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use core::ffi::{c_int, c_void};
use core::{mem, ptr};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyGridRef {
    pub(crate) size: usize,
    pub(crate) node: *mut c_void,
    pub(crate) x: u16,
    pub(crate) y: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttySelection {
    pub(crate) size: usize,
    pub(crate) start: GhosttyGridRef,
    pub(crate) end: GhosttyGridRef,
    pub(crate) rectangle: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyTerminalScrollbar {
    pub(crate) total: u64,
    pub(crate) offset: u64,
    pub(crate) len: u64,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_write(
    has_value: bool,
    selection: *const GhosttySelection,
    out: *mut GhosttySelection,
) -> c_int {
    unsafe { selection_write_impl(has_value, selection, out) }
}

pub(crate) unsafe fn selection_write_impl(
    has_value: bool,
    selection: *const GhosttySelection,
    out: *mut GhosttySelection,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe { copy_selection(out, selection) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_write_order(
    order: c_int,
    out: *mut c_int,
) -> c_int {
    unsafe { selection_write_order_impl(order, out) }
}

pub(crate) unsafe fn selection_write_order_impl(order: c_int, out: *mut c_int) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    match order {
        SELECTION_ORDER_FORWARD
        | SELECTION_ORDER_REVERSE
        | SELECTION_ORDER_MIRRORED_FORWARD
        | SELECTION_ORDER_MIRRORED_REVERSE => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    unsafe {
        ptr::write(out, order);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_write_bool(value: bool, out: *mut bool) -> c_int {
    unsafe { selection_write_bool_impl(value, out) }
}

pub(crate) unsafe fn selection_write_bool_impl(value: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_equal(
    terminal: *mut c_void,
    a: *const GhosttySelection,
    b: *const GhosttySelection,
    out: *mut bool,
) -> c_int {
    unsafe { selection_equal_impl(terminal, a, b, out) }
}

pub(crate) unsafe fn selection_equal_impl(
    terminal: *mut c_void,
    a: *const GhosttySelection,
    b: *const GhosttySelection,
    out: *mut bool,
) -> c_int {
    if terminal.is_null() || a.is_null() || b.is_null() || out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let a_start = unsafe { ptr::read(core::ptr::addr_of!((*a).start)) };
    let a_end = unsafe { ptr::read(core::ptr::addr_of!((*a).end)) };
    let b_start = unsafe { ptr::read(core::ptr::addr_of!((*b).start)) };
    let b_end = unsafe { ptr::read(core::ptr::addr_of!((*b).end)) };
    if !grid_ref_valid(a_start)
        || !grid_ref_valid(a_end)
        || !grid_ref_valid(b_start)
        || !grid_ref_valid(b_end)
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let a_rectangle = unsafe { ptr::read(core::ptr::addr_of!((*a).rectangle)) };
    let b_rectangle = unsafe { ptr::read(core::ptr::addr_of!((*b).rectangle)) };
    unsafe {
        ptr::write(
            out,
            grid_ref_equal(a_start, b_start)
                && grid_ref_equal(a_end, b_end)
                && a_rectangle == b_rectangle,
        );
    }

    GHOSTTY_SUCCESS
}
