//! Std-only helpers for cargo integration tests.

#[cfg(feature = "std")]
pub fn test_allocator() -> crate::allocator::GhosttyAllocator {
    crate::allocator::test_support_allocator()
}

#[cfg(feature = "std")]
fn active_grid_ref(
    handle: *mut core::ffi::c_void,
    x: u16,
    y: u16,
) -> Option<crate::selection::GhosttyGridRef> {
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
        Some(grid_ref.assume_init())
    }
}

#[cfg(feature = "std")]
fn resolve_style_color(
    terminal: &crate::terminal_types::Terminal,
    style_color: &crate::style::GhosttyStyleColor,
    default: Option<crate::style::GhosttyColorRgb>,
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
            STYLE_COLOR_NONE => default.map(|c| [c.r, c.g, c.b]),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
fn terminal_cell_colors(
    handle: *mut core::ffi::c_void,
    x: u16,
    y: u16,
) -> Option<([u8; 3], [u8; 3])> {
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
        let terminal = &owned.terminal;
        let palette0 = terminal.colors.palette.current()[0];
        let default_fg = terminal.colors.foreground.get().unwrap_or(palette0);
        let default_bg = terminal.colors.background.get().unwrap_or(palette0);
        let mut fg = resolve_style_color(terminal, &style.fg_color, Some(default_fg))?;
        let mut bg = resolve_style_color(terminal, &style.bg_color, Some(default_bg))?;
        if style.inverse {
            core::mem::swap(&mut fg, &mut bg);
        }
        Some((fg, bg))
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
    terminal_cell_colors(handle, x, y).map(|(fg, _)| fg)
}

/// Resolve the effective background RGB for a cell on the active screen.
#[cfg(feature = "std")]
pub fn terminal_cell_bg_rgb(handle: *mut core::ffi::c_void, x: u16, y: u16) -> Option<[u8; 3]> {
    terminal_cell_colors(handle, x, y).map(|(_, bg)| bg)
}

/// Resolve effective foreground and background RGB for a cell.
#[cfg(feature = "std")]
pub fn terminal_cell_colors_rgb(
    handle: *mut core::ffi::c_void,
    x: u16,
    y: u16,
) -> Option<([u8; 3], [u8; 3])> {
    terminal_cell_colors(handle, x, y)
}

/// Raw wide-cell tag (`page_types::Wide` as u8) for a grid cell.
#[cfg(feature = "std")]
pub fn terminal_cell_wide_raw(handle: *mut core::ffi::c_void, x: u16, y: u16) -> Option<u8> {
    use crate::constants::CELL_DATA_WIDE;
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
        let mut wide: i32 = 0;
        if crate::cell::ghostty_rust_cell_get(
            cell,
            CELL_DATA_WIDE,
            &mut wide as *mut i32 as *mut core::ffi::c_void,
        ) != GHOSTTY_SUCCESS
        {
            return None;
        }
        Some(wide as u8)
    }
}

/// Cursor fields for the renderer draw path (viewport-relative when visible).
#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCursorSnapshot {
    pub viewport_visible: bool,
    pub viewport_x: u16,
    pub viewport_y: u16,
    pub viewport_wide_tail: bool,
    pub visible: bool,
    pub blinking: bool,
    pub password_input: bool,
    /// Matches `constants::RENDER_CURSOR_STYLE_*` (bar=0, block=1, underline=2, hollow=3).
    pub visual_style: u8,
    pub cursor_rgb: Option<[u8; 3]>,
}

#[cfg(feature = "std")]
impl Default for TerminalCursorSnapshot {
    fn default() -> Self {
        Self {
            viewport_visible: false,
            viewport_x: 0,
            viewport_y: 0,
            viewport_wide_tail: false,
            visible: true,
            blinking: false,
            password_input: false,
            visual_style: crate::constants::RENDER_CURSOR_STYLE_BLOCK as u8,
            cursor_rgb: None,
        }
    }
}

/// Read cursor state for renderer style resolution and draw placement.
#[cfg(feature = "std")]
pub fn terminal_cursor_snapshot(handle: *mut core::ffi::c_void) -> TerminalCursorSnapshot {
    use crate::constants::{
        RENDER_CURSOR_STYLE_BAR, RENDER_CURSOR_STYLE_BLOCK, RENDER_CURSOR_STYLE_BLOCK_HOLLOW,
        RENDER_CURSOR_STYLE_UNDERLINE,
    };
    use crate::cursor_style::CursorVisualStyle;
    use crate::mode_def::ModeTag;
    use crate::page_types::Wide;
    use crate::point::PointTag;
    use crate::terminal_owned::RustTerminalOwned;

    unsafe {
        let owned = &*(handle as *const RustTerminalOwned);
        let term = &owned.terminal;
        let visible = term.modes.get_by_tag(ModeTag {
            value: 25,
            ansi: false,
        });
        let blinking = term.modes.get_by_tag(ModeTag {
            value: 12,
            ansi: false,
        });
        let password_input = term.flags.password_input;
        let cursor_rgb = term.colors.cursor.get().map(|c| [c.r, c.g, c.b]);

        let screen = term.active();
        if screen.is_null() {
            return TerminalCursorSnapshot {
                visible,
                blinking,
                password_input,
                cursor_rgb,
                ..TerminalCursorSnapshot::default()
            };
        }

        let screen = &*screen;
        let pages = screen.pages;
        if pages.is_null() {
            return TerminalCursorSnapshot {
                visible,
                blinking,
                password_input,
                cursor_rgb,
                ..TerminalCursorSnapshot::default()
            };
        }
        let pages = &*pages;
        let cursor = &screen.cursor;

        if cursor.page_pin.is_null() {
            return TerminalCursorSnapshot {
                visible,
                blinking,
                password_input,
                cursor_rgb,
                ..TerminalCursorSnapshot::default()
            };
        }

        let visual_style = match cursor.cursor_style {
            CursorVisualStyle::Bar => RENDER_CURSOR_STYLE_BAR,
            CursorVisualStyle::Block => RENDER_CURSOR_STYLE_BLOCK,
            CursorVisualStyle::Underline => RENDER_CURSOR_STYLE_UNDERLINE,
            CursorVisualStyle::BlockHollow => RENDER_CURSOR_STYLE_BLOCK_HOLLOW,
        } as u8;

        let active_pin = *cursor.page_pin;
        if active_pin.node.is_null() {
            return TerminalCursorSnapshot {
                visible,
                blinking,
                password_input,
                visual_style: RENDER_CURSOR_STYLE_BLOCK as u8,
                cursor_rgb,
                viewport_visible: false,
                ..TerminalCursorSnapshot::default()
            };
        }

        let Some((vx, vy)) = pages.point_from_pin(PointTag::VIEWPORT, active_pin) else {
            return TerminalCursorSnapshot {
                visible,
                blinking,
                password_input,
                visual_style,
                cursor_rgb,
                ..TerminalCursorSnapshot::default()
            };
        };

        let mut viewport_wide_tail = false;
        if vx > 0 {
            let left = crate::highlight::Pin {
                x: vx - 1,
                y: active_pin.y,
                node: active_pin.node,
                garbage: active_pin.garbage,
            };
            if !left.node.is_null() {
                let (_row, cell) = left.row_and_cell_ptr();
                if !cell.is_null() && (*cell).wide() == Wide::Wide {
                    viewport_wide_tail = true;
                }
            }
        }

        TerminalCursorSnapshot {
            viewport_visible: true,
            viewport_x: vx as u16,
            viewport_y: vy as u16,
            viewport_wide_tail,
            visible,
            blinking,
            password_input,
            visual_style,
            cursor_rgb,
        }
    }
}
