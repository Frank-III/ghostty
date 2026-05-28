use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::{STYLE_COLOR_NONE, STYLE_COLOR_PALETTE, STYLE_COLOR_RGB};
use crate::early::*;
use crate::hyperlink::HyperlinkPageEntry;
use crate::page_list_types::PageListNode;
use crate::page_types::{Cell, Row};
use crate::style::{GhosttyStyle, GhosttyStyleColor, GhosttyStyleColorValue};
use crate::style_default::write_style_default;
use crate::style_types::{Color, Style};

unsafe fn cell_at_grid_ref(
    node: *mut c_void,
    x: u16,
    y: u16,
) -> Option<(*const crate::page_core::Page, *mut Row, *mut Cell)> {
    if node.is_null() {
        return None;
    }

    unsafe {
        let node = node as *mut PageListNode;
        let page = &(*node).data;
        if y >= page.size.rows || x >= page.size.cols {
            return None;
        }
        let row = page.get_row(y as usize);
        let cell = page.row_cells_ptr(row).add(x as usize);
        Some((page as *const _, row, cell))
    }
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
                rgb: crate::style::GhosttyColorRgb {
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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_cell_from_ref(
    node: *mut c_void,
    x: u16,
    y: u16,
    out_cell: *mut u64,
) -> c_int {
    unsafe {
        let Some((_page, _row, cell)) = cell_at_grid_ref(node, x, y) else {
            return GHOSTTY_INVALID_VALUE;
        };
        if !out_cell.is_null() {
            ptr::write(out_cell, (*cell).cval());
        }
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_row_from_ref(
    node: *mut c_void,
    x: u16,
    y: u16,
    out_row: *mut u64,
) -> c_int {
    unsafe {
        let Some((_page, row, _cell)) = cell_at_grid_ref(node, x, y) else {
            return GHOSTTY_INVALID_VALUE;
        };
        if !out_row.is_null() {
            ptr::write(out_row, (*row).cval());
        }
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_graphemes_from_ref(
    node: *mut c_void,
    x: u16,
    y: u16,
    out_buf: *mut u32,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        if out_len.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let Some((page, _row, cell)) = cell_at_grid_ref(node, x, y) else {
            return GHOSTTY_INVALID_VALUE;
        };
        if !(*cell).has_text() {
            ptr::write(out_len, 0);
            return GHOSTTY_SUCCESS;
        }

        let extra = (*page).lookup_grapheme(cell as *const Cell);
        let extra_len = extra.map_or(0, |(_, len)| len);
        let total = 1 + extra_len;
        ptr::write(out_len, total);
        if out_buf.is_null() || buf_len < total {
            return GHOSTTY_OUT_OF_SPACE;
        }

        ptr::write(out_buf, (*cell).codepoint());
        if let Some((extra_ptr, len)) = extra {
            let mut i = 0usize;
            while i < len {
                ptr::write(out_buf.add(i + 1), ptr::read(extra_ptr.add(i)));
                i += 1;
            }
        }
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_hyperlink_uri_from_ref(
    node: *mut c_void,
    x: u16,
    y: u16,
    out_buf: *mut u8,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        if out_len.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let Some((page, _row, cell)) = cell_at_grid_ref(node, x, y) else {
            return GHOSTTY_INVALID_VALUE;
        };
        if !(*cell).hyperlink() {
            return grid_ref_hyperlink_uri_impl(false, ptr::null(), 0, out_buf, buf_len, out_len);
        }
        let Some(id) = (*page).lookup_hyperlink(cell as *const Cell) else {
            return grid_ref_hyperlink_uri_impl(false, ptr::null(), 0, out_buf, buf_len, out_len);
        };
        let entry: HyperlinkPageEntry = (*page).hyperlink_set.get((*page).memory, id);
        let uri = entry.uri_slice((*page).memory as *const u8);
        grid_ref_hyperlink_uri_impl(true, uri.as_ptr(), uri.len(), out_buf, buf_len, out_len)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_style_from_ref(
    node: *mut c_void,
    x: u16,
    y: u16,
    out_style: *mut GhosttyStyle,
) -> c_int {
    unsafe {
        let Some((page, _row, cell)) = cell_at_grid_ref(node, x, y) else {
            return GHOSTTY_INVALID_VALUE;
        };
        if out_style.is_null() {
            return GHOSTTY_SUCCESS;
        }
        let style_id = (*cell).style_id();
        if style_id == crate::style_types::DEFAULT_ID {
            write_style_default(out_style);
        } else {
            let style: Style = (*page).styles.get((*page).memory, style_id);
            ptr::write(out_style, style_to_ghostty_style(&style));
        }
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_hyperlink_uri(
    has_uri: bool,
    uri: *const u8,
    uri_len: usize,
    out_buf: *mut u8,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe { grid_ref_hyperlink_uri_impl(has_uri, uri, uri_len, out_buf, buf_len, out_len) }
}

pub(crate) unsafe fn grid_ref_hyperlink_uri_impl(
    has_uri: bool,
    uri: *const u8,
    uri_len: usize,
    out_buf: *mut u8,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        ptr::write(out_len, if has_uri { uri_len } else { 0 });
    }

    if !has_uri {
        return GHOSTTY_SUCCESS;
    }

    if out_buf.is_null() || buf_len < uri_len {
        return GHOSTTY_OUT_OF_SPACE;
    }

    let mut i = 0usize;
    while i < uri_len {
        unsafe {
            ptr::write(out_buf.add(i), ptr::read(uri.add(i)));
        }
        i += 1;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_grid_ref_graphemes(
    has_text: bool,
    codepoint: u32,
    out_buf: *mut u32,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe { grid_ref_graphemes_impl(has_text, codepoint, out_buf, buf_len, out_len) }
}

pub(crate) unsafe fn grid_ref_graphemes_impl(
    has_text: bool,
    codepoint: u32,
    out_buf: *mut u32,
    buf_len: usize,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        ptr::write(out_len, if has_text { 1 } else { 0 });
    }

    if !has_text {
        return GHOSTTY_SUCCESS;
    }

    if out_buf.is_null() || buf_len < 1 {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        ptr::write(out_buf, codepoint);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_has_value(
    has_ref: bool,
    has_page_list: bool,
    garbage: bool,
) -> bool {
    tracked_grid_ref_has_value_impl(has_ref, has_page_list, garbage)
}

pub(crate) fn tracked_grid_ref_has_value_impl(
    has_ref: bool,
    has_page_list: bool,
    garbage: bool,
) -> bool {
    has_ref && has_page_list && !garbage
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_result(
    has_ref: bool,
    has_page_list: bool,
    garbage: bool,
    has_point: bool,
) -> c_int {
    tracked_grid_ref_result_impl(has_ref, has_page_list, garbage, has_point)
}

pub(crate) fn tracked_grid_ref_result_impl(
    has_ref: bool,
    has_page_list: bool,
    garbage: bool,
    has_point: bool,
) -> c_int {
    if !has_ref {
        return GHOSTTY_INVALID_VALUE;
    }

    if !has_page_list || garbage || !has_point {
        return GHOSTTY_NO_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_set_input(
    has_ref: bool,
    has_terminal: bool,
    same_terminal: bool,
) -> c_int {
    tracked_grid_ref_set_input_impl(has_ref, has_terminal, same_terminal)
}

pub(crate) fn tracked_grid_ref_set_input_impl(
    has_ref: bool,
    has_terminal: bool,
    same_terminal: bool,
) -> c_int {
    if !has_ref || !has_terminal || !same_terminal {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}
