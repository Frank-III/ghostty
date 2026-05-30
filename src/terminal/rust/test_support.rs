//! Std-only helpers for cargo integration tests.

#[cfg(feature = "std")]
pub fn test_allocator() -> crate::allocator::GhosttyAllocator {
    crate::allocator::test_support_allocator()
}

#[cfg(feature = "std")]
fn active_grid_ref(handle: *mut core::ffi::c_void, x: u16, y: u16) -> Option<crate::selection::GhosttyGridRef> {
    use core::mem;
    use core::ptr;

    use crate::early::GHOSTTY_SUCCESS;
    use crate::point::{Coordinate, PointC, PointTag};

    unsafe {
        let mut pt = mem::MaybeUninit::<PointC>::uninit();
        let pt_ptr = pt.as_mut_ptr();
        ptr::write(ptr::addr_of_mut!((*pt_ptr).tag), PointTag::ACTIVE);
        ptr::write(
            ptr::addr_of_mut!((*pt_ptr).value.active),
            Coordinate {
                x,
                y: u32::from(y),
            },
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
        Some(grid_ref.assume_init())
    }
}

#[cfg(feature = "std")]
fn resolve_style_color(
    terminal: &crate::terminal_types::Terminal,
    style_color: &crate::style::GhosttyStyleColor,
) -> Option<[u8; 3]> {
    use crate::constants::{STYLE_COLOR_NONE, STYLE_COLOR_PALETTE, STYLE_COLOR_RGB};

    unsafe {
        match style_color.tag {
            STYLE_COLOR_RGB => Some([
                style_color.value.rgb.r,
                style_color.value.rgb.g,
                style_color.value.rgb.b,
            ]),
            STYLE_COLOR_PALETTE => {
                let idx = style_color.value.palette as usize;
                let c = terminal.colors.palette.current()[idx];
                Some([c.r, c.g, c.b])
            }
            STYLE_COLOR_NONE => terminal
                .colors
                .foreground
                .get()
                .map(|c| [c.r, c.g, c.b]),
            _ => None,
        }
    }
}

/// Read a cell codepoint from a rust-owned terminal at active-screen `(x, y)`.
#[cfg(feature = "std")]
pub fn terminal_cell_codepoint(handle: *mut core::ffi::c_void, x: u16, y: u16) -> Option<u32> {
    use core::mem;

    use crate::constants::CELL_DATA_CODEPOINT;
    use crate::early::GHOSTTY_SUCCESS;

    unsafe {
        let grid_ref = active_grid_ref(handle, x, y)?;
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

/// Resolve the effective foreground RGB for a cell on the active screen.
#[cfg(feature = "std")]
pub fn terminal_cell_fg_rgb(handle: *mut core::ffi::c_void, x: u16, y: u16) -> Option<[u8; 3]> {
    use core::mem;

    use crate::early::GHOSTTY_SUCCESS;
    use crate::style::GhosttyStyle;
    use crate::terminal_owned::RustTerminalOwned;

    unsafe {
        let grid_ref = active_grid_ref(handle, x, y)?;
        let mut style = mem::MaybeUninit::<GhosttyStyle>::uninit();
        if crate::grid_ref::ghostty_rust_grid_ref_style_from_ref(
            grid_ref.node,
            grid_ref.x,
            grid_ref.y,
            style.as_mut_ptr(),
        ) != GHOSTTY_SUCCESS
        {
            return None;
        }
        let style = style.assume_init();
        let owned = &*(handle as *const RustTerminalOwned);
        resolve_style_color(&owned.terminal, &style.fg_color)
    }
}
