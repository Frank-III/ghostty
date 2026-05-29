//! Std-only helpers for cargo integration tests.

#[cfg(feature = "std")]
pub fn test_allocator() -> crate::allocator::GhosttyAllocator {
    crate::allocator::test_support_allocator()
}

/// Read a cell codepoint from a rust-owned terminal at active-screen `(x, y)`.
#[cfg(feature = "std")]
pub fn terminal_cell_codepoint(handle: *mut core::ffi::c_void, x: u16, y: u16) -> Option<u32> {
    use core::mem;
    use core::ptr;

    use crate::constants::CELL_DATA_CODEPOINT;
    use crate::early::GHOSTTY_SUCCESS;
    use crate::point::{Coordinate, PointC, PointTag};

    unsafe {
        let mut pt = mem::MaybeUninit::<PointC>::uninit();
        let pt_ptr = pt.as_mut_ptr();
        ptr::write(ptr::addr_of_mut!((*pt_ptr).tag), PointTag::ACTIVE);
        ptr::write(
            ptr::addr_of_mut!((*pt_ptr).value.active),
            Coordinate { x, y: u32::from(y) },
        );
        let pt = pt.assume_init();

        let mut grid_ref = mem::MaybeUninit::<crate::selection::GhosttyGridRef>::uninit();
        if crate::terminal_owned::ghostty_rust_terminal_owned_grid_ref(
            handle,
            &pt,
            grid_ref.as_mut_ptr(),
        ) != GHOSTTY_SUCCESS
        {
            return None;
        }
        let grid_ref = grid_ref.assume_init();

        let mut cell: u64 = 0;
        if crate::grid_ref::ghostty_rust_grid_ref_cell_from_ref(
            grid_ref.node,
            grid_ref.x,
            grid_ref.y,
            &mut cell,
        ) != GHOSTTY_SUCCESS
        {
            return None;
        }
        let mut codepoint: u32 = 0;
        if crate::cell::ghostty_rust_cell_get(
            cell,
            CELL_DATA_CODEPOINT,
            &mut codepoint as *mut u32 as *mut core::ffi::c_void,
        ) != GHOSTTY_SUCCESS
        {
            return None;
        }
        Some(codepoint)
    }
}
