use core::ffi::{c_int, c_void};
use core::{mem, ptr};

use crate::cell::*;
use crate::constants::*;
use crate::render_cell_text::*;
use crate::style::*;
use crate::style_copy::*;

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_cell_get(
    has_cells: bool,
    has_cell: bool,
    data: c_int,
    has_out: bool,
) -> c_int {
    if !has_cells || !has_cell || !has_out {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match data {
        RENDER_ROW_CELL_DATA_RAW
        | RENDER_ROW_CELL_DATA_STYLE
        | RENDER_ROW_CELL_DATA_GRAPHEMES_LEN
        | RENDER_ROW_CELL_DATA_GRAPHEMES_BUF
        | RENDER_ROW_CELL_DATA_BG_COLOR
        | RENDER_ROW_CELL_DATA_FG_COLOR
        | RENDER_ROW_CELL_DATA_SELECTED => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}

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
                    ghostty_rust_render_row_cell_get_text(key, cell, extra, extra_len, out)
                }
                RENDER_ROW_CELL_DATA_BG_COLOR | RENDER_ROW_CELL_DATA_FG_COLOR => {
                    ghostty_rust_render_row_cell_get_color(
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
                RENDER_ROW_CELL_DATA_STYLE => ghostty_rust_render_row_cell_get_style(
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
                RENDER_ROW_CELL_DATA_SELECTED => ghostty_rust_render_row_cell_selected(
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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_get_color(
    data: c_int,
    cell: u64,
    fg_color: *const GhosttyStyleColor,
    bg_color: *const GhosttyStyleColor,
    cell_palette_color: GhosttyColorRgb,
    fg_palette_color: GhosttyColorRgb,
    bg_palette_color: GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_ROW_CELL_DATA_BG_COLOR => match cell_content_tag(cell) {
                CELL_CONTENT_TAG_BG_COLOR_PALETTE => {
                    write_rgb(out, &cell_palette_color);
                    RENDER_RESULT_SUCCESS
                }
                CELL_CONTENT_TAG_BG_COLOR_RGB => {
                    write_cell_rgb(out, cell_content(cell));
                    RENDER_RESULT_SUCCESS
                }
                _ => write_style_color_rgb(bg_color, &bg_palette_color, out),
            },
            RENDER_ROW_CELL_DATA_FG_COLOR => {
                write_style_color_rgb(fg_color, &fg_palette_color, out)
            }
            _ => RENDER_RESULT_INVALID_VALUE,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_get_style(
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
    out: *mut GhosttyStyle,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*out).size),
            mem::size_of::<GhosttyStyle>(),
        );
        let result = copy_style_color(core::ptr::addr_of_mut!((*out).fg_color), fg_color);
        if result != RENDER_RESULT_SUCCESS {
            return result;
        }
        let result = copy_style_color(core::ptr::addr_of_mut!((*out).bg_color), bg_color);
        if result != RENDER_RESULT_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*out).underline_color),
            underline_color,
        );
        if result != RENDER_RESULT_SUCCESS {
            return result;
        }
        ptr::write(core::ptr::addr_of_mut!((*out).bold), bold);
        ptr::write(core::ptr::addr_of_mut!((*out).italic), italic);
        ptr::write(core::ptr::addr_of_mut!((*out).faint), faint);
        ptr::write(core::ptr::addr_of_mut!((*out).blink), blink);
        ptr::write(core::ptr::addr_of_mut!((*out).inverse), inverse);
        ptr::write(core::ptr::addr_of_mut!((*out).invisible), invisible);
        ptr::write(core::ptr::addr_of_mut!((*out).strikethrough), strikethrough);
        ptr::write(core::ptr::addr_of_mut!((*out).overline), overline);
        ptr::write(core::ptr::addr_of_mut!((*out).underline), underline);
    }

    RENDER_RESULT_SUCCESS
}
