use core::ffi::{c_int, c_void};
use core::{mem, ptr};

use crate::early::*;
use crate::selection::*;
use crate::terminal::*;

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
