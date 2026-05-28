use core::ffi::{c_int, c_void};
use core::{mem, ptr};

use crate::early::*;
use crate::highlight::Pin;
use crate::page_list_types::PageListNode;
use crate::selection::*;
use crate::selection_types::Selection;

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

pub(crate) unsafe fn copy_selection(
    dst: *mut GhosttySelection,
    src: *const GhosttySelection,
) -> c_int {
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

pub(crate) fn grid_ref_to_pin(grid: GhosttyGridRef) -> Option<Pin> {
    if grid.node.is_null() {
        return None;
    }
    Some(Pin {
        node: grid.node as *mut PageListNode,
        x: grid.x,
        y: grid.y,
        garbage: false,
    })
}

pub(crate) fn grid_ref_from_pin(pin: Pin) -> GhosttyGridRef {
    GhosttyGridRef {
        size: mem::size_of::<GhosttyGridRef>(),
        node: pin.node as *mut c_void,
        x: pin.x,
        y: pin.y,
    }
}

pub(crate) fn selection_from_ghostty(c: &GhosttySelection) -> Option<Selection> {
    let start = grid_ref_to_pin(c.start)?;
    let end = grid_ref_to_pin(c.end)?;
    Some(Selection::init(start, end, c.rectangle))
}

pub(crate) fn selection_to_ghostty(sel: &Selection) -> GhosttySelection {
    GhosttySelection {
        size: mem::size_of::<GhosttySelection>(),
        start: grid_ref_from_pin(sel.start()),
        end: grid_ref_from_pin(sel.end_pin()),
        rectangle: sel.rectangle,
    }
}
