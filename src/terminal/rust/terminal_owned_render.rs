use core::ffi::{c_int, c_void};
use core::ptr;

use crate::allocator::GhosttyAllocator;
use crate::constants::{STYLE_COLOR_NONE, STYLE_COLOR_PALETTE, STYLE_COLOR_RGB};
use crate::early::*;
use crate::highlight::Pin;
use crate::mode_def::ModeTag;
use crate::page_list_types::{PageList, PageListDirection};
use crate::page_types::{Cell, Row, Wide};
use crate::point::PointTag;
use crate::screen_types::Screen;
use crate::selection_types::Selection;
use crate::style::{GhosttyColorRgb, GhosttyStyle, GhosttyStyleColor, GhosttyStyleColorValue};
use crate::style_types::{Color, Style, DEFAULT_ID};
use crate::terminal_owned::RustTerminalOwned;

#[allow(improper_ctypes)]
extern "C" {
    fn ghostty_vt_render_owned_begin(
        state: *mut c_void,
        alloc: *const GhosttyAllocator,
        rows: u16,
        cols: u16,
        screen_key: u8,
        dirty: c_int,
        cursor_visual_style: u8,
        cursor_visible: bool,
        cursor_blinking: bool,
        cursor_password_input: bool,
        cursor_active_x: u16,
        cursor_active_y: u16,
        cursor_style: *const Style,
        cursor_cell: *const Cell,
        viewport_pin: *const Pin,
        background: GhosttyColorRgb,
        foreground: GhosttyColorRgb,
        cursor_present: bool,
        cursor: GhosttyColorRgb,
        palette: *const GhosttyColorRgb,
    ) -> c_int;

    fn ghostty_vt_render_owned_row(
        state: *mut c_void,
        alloc: *const GhosttyAllocator,
        y: u16,
        pin: *const Pin,
        row: *const Row,
        cells: *const Cell,
        cols: u16,
        dirty: bool,
        selection_present: bool,
        selection_start: u16,
        selection_end: u16,
        is_cursor_row: bool,
        cursor_x: u16,
        cursor_wide_tail: bool,
    ) -> c_int;

    fn ghostty_vt_render_owned_cell_style(
        state: *mut c_void,
        y: u16,
        x: u16,
        style: *const GhosttyStyle,
    ) -> c_int;

    fn ghostty_vt_render_owned_end(state: *mut c_void, any_dirty: bool) -> c_int;
}

fn style_color_to_c(color: Color) -> GhosttyStyleColor {
    match color {
        Color::None => GhosttyStyleColor {
            tag: STYLE_COLOR_NONE,
            value: GhosttyStyleColorValue { padding: 0 },
        },
        Color::Palette(idx) => GhosttyStyleColor {
            tag: STYLE_COLOR_PALETTE,
            value: GhosttyStyleColorValue { palette: idx },
        },
        Color::Rgb(rgb) => GhosttyStyleColor {
            tag: STYLE_COLOR_RGB,
            value: GhosttyStyleColorValue {
                rgb: GhosttyColorRgb {
                    r: rgb.r,
                    g: rgb.g,
                    b: rgb.b,
                },
            },
        },
    }
}

fn style_to_ghostty_style(style: &Style) -> GhosttyStyle {
    GhosttyStyle {
        size: core::mem::size_of::<GhosttyStyle>(),
        fg_color: style_color_to_c(style.fg_color),
        bg_color: style_color_to_c(style.bg_color),
        underline_color: style_color_to_c(style.underline_color),
        bold: style.flags.bold(),
        italic: style.flags.italic(),
        faint: style.flags.faint(),
        blink: style.flags.blink(),
        inverse: style.flags.inverse(),
        invisible: style.flags.invisible(),
        strikethrough: style.flags.strikethrough(),
        overline: style.flags.overline(),
        underline: if style.flags.underline() { 1 } else { 0 },
    }
}

fn selection_row_range(
    pages: &PageList,
    _screen: &Screen,
    sel: &Selection,
    row_pin: Pin,
    _viewport_y: u16,
) -> Option<(u16, u16)> {
    let tl = pages.point_from_pin(PointTag::VIEWPORT, sel.start())?;
    let br = pages.point_from_pin(PointTag::VIEWPORT, sel.end_pin())?;
    let row_pt = pages.point_from_pin(PointTag::VIEWPORT, row_pin)?;
    if row_pt.1 < tl.1 || row_pt.1 > br.1 {
        return None;
    }
    if sel.rectangle {
        return Some((tl.0, br.0));
    }
    if row_pt.1 == tl.1 && row_pt.1 == br.1 {
        return Some((tl.0, br.0));
    }
    if row_pt.1 == tl.1 {
        return Some((tl.0, pages.cols.saturating_sub(1)));
    }
    if row_pt.1 == br.1 {
        return Some((0, br.0));
    }
    Some((0, pages.cols.saturating_sub(1)))
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_render_state_update(
    handle: *mut c_void,
    state: *mut c_void,
    alloc: *const GhosttyAllocator,
) -> c_int {
    unsafe {
        if handle.is_null() || state.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        let term = &mut owned.terminal;
        let screen = term.active();
        if screen.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let pages = (*screen).pages;
        if pages.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let pages = &*pages;

        let rows = pages.rows;
        let cols = pages.cols;
        let screen_key = term.screens.active_key as u8;

        let mut dirty: c_int = 0; // .false
        {
            let d = term.flags.dirty;
            if d.palette || d.reverse_colors || d.clear || d.preedit {
                dirty = 2; // full
            }
        }
        if (*screen).dirty.selection {
            dirty = 2;
        }

        let reverse = term.modes.get_by_tag(ModeTag {
            value: 5,
            ansi: false,
        });
        let bg = term.colors.background.get();
        let fg = term.colors.foreground.get();
        let (background, foreground) = match (bg, fg, reverse) {
            (Some(b), Some(f), true) => (f, b),
            (Some(b), Some(f), false) => (b, f),
            _ => (
                GhosttyColorRgb { r: 0, g: 0, b: 0 },
                GhosttyColorRgb {
                    r: 0xff,
                    g: 0xff,
                    b: 0xff,
                },
            ),
        };

        let cursor_rgb = term.colors.cursor.get();
        let palette = term.colors.palette.current();

        let cursor = &(*screen).cursor;
        let cursor_cell_ptr = if cursor.page_cell.is_null() {
            ptr::null()
        } else {
            cursor.page_cell
        };
        let viewport_pin = pages.get_top_left(PointTag::VIEWPORT);
        let default_cell = Cell::default();
        let cursor_cell_ref: *const Cell = if cursor_cell_ptr.is_null() {
            &default_cell
        } else {
            cursor_cell_ptr
        };

        let result = ghostty_vt_render_owned_begin(
            state,
            alloc,
            rows,
            cols,
            screen_key,
            dirty,
            cursor.cursor_style as u8,
            term.modes.get_by_tag(ModeTag {
                value: 25,
                ansi: false,
            }),
            term.modes.get_by_tag(ModeTag {
                value: 12,
                ansi: false,
            }),
            term.flags.password_input,
            cursor.x,
            cursor.y,
            ptr::from_ref(&cursor.style),
            cursor_cell_ref,
            ptr::from_ref(&viewport_pin),
            background,
            foreground,
            cursor_rgb.is_some(),
            cursor_rgb.unwrap_or(GhosttyColorRgb { r: 0, g: 0, b: 0 }),
            palette.as_ptr(),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }

        let mut row_it = pages.row_iterator(
            PageListDirection::RightDown,
            PointTag::VIEWPORT,
            0,
            0,
            None,
            None,
            None,
        );

        let mut y: u16 = 0;
        let mut any_dirty = dirty == 2;
        while let Some(row_pin) = row_it.next() {
            if y >= rows {
                break;
            }
            let (row_ptr, _) = row_pin.row_and_cell_ptr();
            let row_ref = &*row_ptr;
            let page = &(*row_pin.node).data;
            let cells_ptr = page.get_cells(row_ptr);

            let (sel_present, sel_start, sel_end) = if let Some(ref sel) = (*screen).selection {
                if let Some((start, end)) = selection_row_range(pages, &*screen, sel, row_pin, y) {
                    (true, start, end)
                } else {
                    (false, 0, 0)
                }
            } else {
                (false, 0, 0)
            };

            let row_dirty = dirty == 2 || row_ref.dirty() || page.dirty;
            if row_dirty {
                any_dirty = true;
            }

            let (is_cursor_row, cursor_wide_tail) =
                match pages.pin(PointTag::ACTIVE, cursor.x, cursor.y as u32) {
                    None => (false, false),
                    Some(active_pin) => {
                        if active_pin.node != row_pin.node || active_pin.y != row_pin.y {
                            (false, false)
                        } else {
                            let mut wide_tail = false;
                            if cursor.x > 0 {
                                let left = Pin {
                                    x: cursor.x - 1,
                                    ..active_pin
                                };
                                let (_r, cell) = left.row_and_cell_ptr();
                                if !cell.is_null() {
                                    wide_tail = (*cell).wide() == Wide::Wide;
                                }
                            }
                            (true, wide_tail)
                        }
                    }
                };

            let row_result = ghostty_vt_render_owned_row(
                state,
                alloc,
                y,
                &row_pin,
                row_ref,
                cells_ptr.as_ptr(),
                cols,
                row_dirty,
                sel_present,
                sel_start,
                sel_end,
                is_cursor_row,
                (*screen).cursor.x,
                cursor_wide_tail,
            );
            if row_result != GHOSTTY_SUCCESS {
                ghostty_vt_render_owned_end(state, any_dirty);
                return row_result;
            }

            let mut x = 0usize;
            while x < cols as usize {
                let cell = *cells_ptr.as_ptr().add(x);
                let style_id = cell.style_id();
                if style_id != DEFAULT_ID {
                    let style: Style = page.styles.get(page.memory, style_id);
                    let c_style = style_to_ghostty_style(&style);
                    let style_result =
                        ghostty_vt_render_owned_cell_style(state, y, x as u16, &c_style);
                    if style_result != GHOSTTY_SUCCESS {
                        ghostty_vt_render_owned_end(state, any_dirty);
                        return style_result;
                    }
                }
                x += 1;
            }

            y += 1;
        }

        ghostty_vt_render_owned_end(state, any_dirty)
    }
}
