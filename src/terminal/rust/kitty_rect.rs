use core::ffi::{c_int, c_void};
use core::{mem, ptr};

use crate::early::*;
use crate::selection::*;
use crate::terminal::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_rect(
    start_node: *mut c_void,
    start_x: u16,
    start_y: u16,
    end_node: *mut c_void,
    end_y: u16,
    grid_cols_minus_one: u32,
    terminal_cols_minus_one: u16,
    out: *mut GhosttySelection,
) -> c_int {
    unsafe {
        kitty_rect_impl(
            start_node,
            start_x,
            start_y,
            end_node,
            end_y,
            grid_cols_minus_one,
            terminal_cols_minus_one,
            out,
        )
    }
}

pub(crate) unsafe fn kitty_rect_impl(
    start_node: *mut c_void,
    start_x: u16,
    start_y: u16,
    end_node: *mut c_void,
    end_y: u16,
    grid_cols_minus_one: u32,
    terminal_cols_minus_one: u16,
    out: *mut GhosttySelection,
) -> c_int {
    if start_node.is_null() || end_node.is_null() || out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let end_x = u32::from(start_x)
        .wrapping_add(grid_cols_minus_one)
        .min(u32::from(terminal_cols_minus_one)) as u16;

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*out).size),
            mem::size_of::<GhosttySelection>(),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*out).start.size),
            mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(core::ptr::addr_of_mut!((*out).start.node), start_node);
        ptr::write(core::ptr::addr_of_mut!((*out).start.x), start_x);
        ptr::write(core::ptr::addr_of_mut!((*out).start.y), start_y);
        ptr::write(
            core::ptr::addr_of_mut!((*out).end.size),
            mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(core::ptr::addr_of_mut!((*out).end.node), end_node);
        ptr::write(core::ptr::addr_of_mut!((*out).end.x), end_x);
        ptr::write(core::ptr::addr_of_mut!((*out).end.y), end_y);
        ptr::write(core::ptr::addr_of_mut!((*out).rectangle), true);
    }

    GHOSTTY_SUCCESS
}
