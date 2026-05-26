use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::render_cell_style::*;
use crate::render_cell_text::*;
use crate::style::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    cell: u64,
    extra: *const u32,
    extra_len: usize,
    fg_color: *const GhosttyStyleColor,
    bg_color: *const GhosttyStyleColor,
    underline_color: *const GhosttyStyleColor,
    bold: bool,
    italic: bool,
    faint: bool,
    blink: bool,
    inverse: bool,
    invisible: bool,
    strikethrough: bool,
    overline: bool,
    underline: c_int,
    cell_palette_color: GhosttyColorRgb,
    fg_palette_color: GhosttyColorRgb,
    bg_palette_color: GhosttyColorRgb,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    x: u16,
) -> c_int {
    unsafe {
        render_row_cell_get_multi_impl(
            count,
            keys,
            values,
            out_written,
            cell,
            extra,
            extra_len,
            fg_color,
            bg_color,
            underline_color,
            bold,
            italic,
            faint,
            blink,
            inverse,
            invisible,
            strikethrough,
            overline,
            underline,
            cell_palette_color,
            fg_palette_color,
            bg_palette_color,
            selection_present,
            selection_start,
            selection_end,
            x,
        )
    }
}

pub(crate) unsafe fn render_row_cell_get_multi_impl(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    cell: u64,
    extra: *const u32,
    extra_len: usize,
    fg_color: *const GhosttyStyleColor,
    bg_color: *const GhosttyStyleColor,
    underline_color: *const GhosttyStyleColor,
    bold: bool,
    italic: bool,
    faint: bool,
    blink: bool,
    inverse: bool,
    invisible: bool,
    strikethrough: bool,
    overline: bool,
    underline: c_int,
    cell_palette_color: GhosttyColorRgb,
    fg_palette_color: GhosttyColorRgb,
    bg_palette_color: GhosttyColorRgb,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    x: u16,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe {
            match key {
                RENDER_ROW_CELL_DATA_RAW
                | RENDER_ROW_CELL_DATA_GRAPHEMES_LEN
                | RENDER_ROW_CELL_DATA_GRAPHEMES_BUF => {
                    render_row_cell_get_text_impl(key, cell, extra, extra_len, out)
                }
                RENDER_ROW_CELL_DATA_BG_COLOR | RENDER_ROW_CELL_DATA_FG_COLOR => {
                    render_row_cell_get_color_impl(
                        key,
                        cell,
                        fg_color,
                        bg_color,
                        cell_palette_color,
                        fg_palette_color,
                        bg_palette_color,
                        out,
                    )
                }
                RENDER_ROW_CELL_DATA_STYLE => render_row_cell_get_style_impl(
                    fg_color,
                    bg_color,
                    underline_color,
                    bold,
                    italic,
                    faint,
                    blink,
                    inverse,
                    invisible,
                    strikethrough,
                    overline,
                    underline,
                    out.cast::<GhosttyStyle>(),
                ),
                RENDER_ROW_CELL_DATA_SELECTED => render_row_cell_selected_impl(
                    selection_present,
                    selection_start,
                    selection_end,
                    x,
                    out.cast::<bool>(),
                ),
                _ => RENDER_RESULT_INVALID_VALUE,
            }
        };

        if result != RENDER_RESULT_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, i);
                }
            }
            return result;
        }

        i += 1;
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    RENDER_RESULT_SUCCESS
}
