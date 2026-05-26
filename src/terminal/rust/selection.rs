use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;

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

pub(crate) fn grid_ref_valid(value: GhosttyGridRef) -> bool {
    !value.node.is_null()
}

pub(crate) fn grid_ref_equal(a: GhosttyGridRef, b: GhosttyGridRef) -> bool {
    a.node == b.node && a.x == b.x && a.y == b.y
}

pub(crate) unsafe fn copy_grid_ref(dst: *mut GhosttyGridRef, src: *const GhosttyGridRef) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).node),
            ptr::read(core::ptr::addr_of!((*src).node)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).x),
            ptr::read(core::ptr::addr_of!((*src).x)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).y),
            ptr::read(core::ptr::addr_of!((*src).y)),
        );
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn copy_selection(dst: *mut GhosttySelection, src: *const GhosttySelection) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );

        let result = copy_grid_ref(
            core::ptr::addr_of_mut!((*dst).start),
            core::ptr::addr_of!((*src).start),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }

        let result = copy_grid_ref(
            core::ptr::addr_of_mut!((*dst).end),
            core::ptr::addr_of!((*src).end),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }

        ptr::write(
            core::ptr::addr_of_mut!((*dst).rectangle),
            ptr::read(core::ptr::addr_of!((*src).rectangle)),
        );
    }

    GHOSTTY_SUCCESS
}

