use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;
use crate::cell::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_index_next(
    has_current: bool,
    current: u16,
    len: usize,
    out_next: *mut u16,
) -> bool {
    if out_next.is_null() {
        return false;
    }

    let next = if has_current {
        match current.checked_add(1) {
            Some(value) => value,
            None => return false,
        }
    } else {
        0
    };

    if usize::from(next) >= len {
        return false;
    }

    unsafe {
        ptr::write(out_next, next);
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_index_select(index: u16, len: usize) -> c_int {
    if usize::from(index) >= len {
        return RENDER_RESULT_INVALID_VALUE;
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_selected(
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    x: u16,
    out: *mut bool,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            out,
            selection_present && x >= selection_start && x <= selection_end,
        );
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_get_primitive(
    data: c_int,
    cols: u16,
    rows: u16,
    dirty: c_int,
    cursor_visual_style: c_int,
    cursor_visible: bool,
    cursor_blinking: bool,
    cursor_password_input: bool,
    cursor_viewport_has_value: bool,
    cursor_viewport_x: u16,
    cursor_viewport_y: u16,
    cursor_viewport_wide_tail: bool,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_DATA_COLS => ptr::write(out.cast::<u16>(), cols),
            RENDER_DATA_ROWS => ptr::write(out.cast::<u16>(), rows),
            RENDER_DATA_DIRTY => ptr::write(out.cast::<c_int>(), dirty),
            RENDER_DATA_CURSOR_VISUAL_STYLE => {
                let style = match cursor_visual_style {
                    RENDER_CURSOR_STYLE_BAR
                    | RENDER_CURSOR_STYLE_BLOCK
                    | RENDER_CURSOR_STYLE_UNDERLINE
                    | RENDER_CURSOR_STYLE_BLOCK_HOLLOW => cursor_visual_style,
                    _ => return RENDER_RESULT_INVALID_VALUE,
                };
                ptr::write(out.cast::<c_int>(), style);
            }
            RENDER_DATA_CURSOR_VISIBLE => ptr::write(out.cast::<bool>(), cursor_visible),
            RENDER_DATA_CURSOR_BLINKING => ptr::write(out.cast::<bool>(), cursor_blinking),
            RENDER_DATA_CURSOR_PASSWORD_INPUT => {
                ptr::write(out.cast::<bool>(), cursor_password_input);
            }
            RENDER_DATA_CURSOR_VIEWPORT_HAS_VALUE => {
                ptr::write(out.cast::<bool>(), cursor_viewport_has_value);
            }
            RENDER_DATA_CURSOR_VIEWPORT_X => {
                if !cursor_viewport_has_value {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                ptr::write(out.cast::<u16>(), cursor_viewport_x);
            }
            RENDER_DATA_CURSOR_VIEWPORT_Y => {
                if !cursor_viewport_has_value {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                ptr::write(out.cast::<u16>(), cursor_viewport_y);
            }
            RENDER_DATA_CURSOR_VIEWPORT_WIDE_TAIL => {
                if !cursor_viewport_has_value {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                ptr::write(out.cast::<bool>(), cursor_viewport_wide_tail);
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_get_color(
    data: c_int,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor_present: bool,
    cursor: GhosttyColorRgb,
    palette: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_DATA_COLOR_BACKGROUND => {
                write_rgb(out, core::ptr::addr_of!(background));
            }
            RENDER_DATA_COLOR_FOREGROUND => {
                write_rgb(out, core::ptr::addr_of!(foreground));
            }
            RENDER_DATA_COLOR_CURSOR => {
                if !cursor_present {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                write_rgb(out, core::ptr::addr_of!(cursor));
            }
            RENDER_DATA_COLOR_CURSOR_HAS_VALUE => {
                ptr::write(out.cast::<bool>(), cursor_present);
            }
            RENDER_DATA_COLOR_PALETTE => {
                return copy_palette(out.cast::<GhosttyColorRgb>(), palette);
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    cols: u16,
    rows: u16,
    dirty: c_int,
    cursor_visual_style: c_int,
    cursor_visible: bool,
    cursor_blinking: bool,
    cursor_password_input: bool,
    cursor_viewport_has_value: bool,
    cursor_viewport_x: u16,
    cursor_viewport_y: u16,
    cursor_viewport_wide_tail: bool,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor_present: bool,
    cursor: GhosttyColorRgb,
    palette: *const GhosttyColorRgb,
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
                RENDER_DATA_COLS
                | RENDER_DATA_ROWS
                | RENDER_DATA_DIRTY
                | RENDER_DATA_CURSOR_VISUAL_STYLE
                | RENDER_DATA_CURSOR_VISIBLE
                | RENDER_DATA_CURSOR_BLINKING
                | RENDER_DATA_CURSOR_PASSWORD_INPUT
                | RENDER_DATA_CURSOR_VIEWPORT_HAS_VALUE
                | RENDER_DATA_CURSOR_VIEWPORT_X
                | RENDER_DATA_CURSOR_VIEWPORT_Y
                | RENDER_DATA_CURSOR_VIEWPORT_WIDE_TAIL => ghostty_rust_render_state_get_primitive(
                    key,
                    cols,
                    rows,
                    dirty,
                    cursor_visual_style,
                    cursor_visible,
                    cursor_blinking,
                    cursor_password_input,
                    cursor_viewport_has_value,
                    cursor_viewport_x,
                    cursor_viewport_y,
                    cursor_viewport_wide_tail,
                    out,
                ),
                RENDER_DATA_COLOR_BACKGROUND
                | RENDER_DATA_COLOR_FOREGROUND
                | RENDER_DATA_COLOR_CURSOR
                | RENDER_DATA_COLOR_CURSOR_HAS_VALUE
                | RENDER_DATA_COLOR_PALETTE => ghostty_rust_render_state_get_color(
                    key,
                    background,
                    foreground,
                    cursor_present,
                    cursor,
                    palette,
                    out,
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

#[repr(C)]
pub struct GhosttyRenderRowSelection {
    pub(crate) size: usize,
    pub(crate) start_x: u16,
    pub(crate) end_x: u16,
}

#[repr(C)]
pub struct GhosttyRenderStateColors {
    pub(crate) size: usize,
    pub(crate) background: GhosttyColorRgb,
    pub(crate) foreground: GhosttyColorRgb,
    pub(crate) cursor: GhosttyColorRgb,
    pub(crate) cursor_has_value: bool,
    pub(crate) palette: [GhosttyColorRgb; 256],
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_set_dirty(
    value: c_int,
    out: *mut c_int,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_state_set(
    has_state: bool,
    option: c_int,
    has_value: bool,
) -> c_int {
    if !has_state || !has_value {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match option {
        RENDER_STATE_SET_DIRTY => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_colors_get(
    out_size: usize,
    out: *mut GhosttyRenderStateColors,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor_present: bool,
    cursor: GhosttyColorRgb,
    palette: *const GhosttyColorRgb,
) -> c_int {
    if out.is_null() || palette.is_null() || out_size < mem::size_of::<usize>() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        if struct_sized_field_fits::<GhosttyColorRgb>(
            out_size,
            mem::offset_of!(GhosttyRenderStateColors, background),
        ) {
            write_rgb(
                core::ptr::addr_of_mut!((*out).background).cast::<c_void>(),
                core::ptr::addr_of!(background),
            );
        }

        if struct_sized_field_fits::<GhosttyColorRgb>(
            out_size,
            mem::offset_of!(GhosttyRenderStateColors, foreground),
        ) {
            write_rgb(
                core::ptr::addr_of_mut!((*out).foreground).cast::<c_void>(),
                core::ptr::addr_of!(foreground),
            );
        }

        if cursor_present
            && struct_sized_field_fits::<GhosttyColorRgb>(
                out_size,
                mem::offset_of!(GhosttyRenderStateColors, cursor),
            )
        {
            write_rgb(
                core::ptr::addr_of_mut!((*out).cursor).cast::<c_void>(),
                core::ptr::addr_of!(cursor),
            );
        }

        if struct_sized_field_fits::<bool>(
            out_size,
            mem::offset_of!(GhosttyRenderStateColors, cursor_has_value),
        ) {
            ptr::write(
                core::ptr::addr_of_mut!((*out).cursor_has_value),
                cursor_present,
            );
        }

        let palette_offset = mem::offset_of!(GhosttyRenderStateColors, palette);
        if out_size > palette_offset {
            let mut available = out_size - palette_offset;
            let out_palette = core::ptr::addr_of_mut!((*out).palette).cast::<GhosttyColorRgb>();
            let mut i = 0usize;
            while i < 256 && available >= mem::size_of::<GhosttyColorRgb>() {
                write_rgb(out_palette.add(i).cast::<c_void>(), palette.add(i));
                available -= mem::size_of::<GhosttyColorRgb>();
                i += 1;
            }
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_dirty(dirty: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, dirty);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_data(
    data: c_int,
    raw: u64,
    dirty: bool,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    out_size: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_ROW_DATA_DIRTY => ptr::write(out.cast::<bool>(), dirty),
            RENDER_ROW_DATA_RAW => ptr::write(out.cast::<u64>(), raw),
            RENDER_ROW_DATA_SELECTION => {
                if out_size < mem::size_of::<GhosttyRenderRowSelection>() {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                if !selection_present {
                    return GHOSTTY_NO_VALUE;
                }
                let selection = out.cast::<GhosttyRenderRowSelection>();
                ptr::write(
                    core::ptr::addr_of_mut!((*selection).start_x),
                    selection_start,
                );
                ptr::write(core::ptr::addr_of_mut!((*selection).end_x), selection_end);
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_selection(
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
    out_size: usize,
    out: *mut GhosttyRenderRowSelection,
) -> c_int {
    if out.is_null() || out_size < mem::size_of::<GhosttyRenderRowSelection>() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    if !selection_present {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).start_x), selection_start);
        ptr::write(core::ptr::addr_of_mut!((*out).end_x), selection_end);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_get(
    has_iterator: bool,
    has_row: bool,
    data: c_int,
    has_out: bool,
    out_size: usize,
) -> c_int {
    if !has_iterator || !has_row || !has_out {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match data {
        RENDER_ROW_DATA_DIRTY | RENDER_ROW_DATA_RAW | RENDER_ROW_DATA_CELLS => {
            RENDER_RESULT_SUCCESS
        }
        RENDER_ROW_DATA_SELECTION => {
            if out_size < mem::size_of::<GhosttyRenderRowSelection>() {
                RENDER_RESULT_INVALID_VALUE
            } else {
                RENDER_RESULT_SUCCESS
            }
        }
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    raw: u64,
    dirty: bool,
    selection_present: bool,
    selection_start: u16,
    selection_end: u16,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let out_size = if key == RENDER_ROW_DATA_SELECTION && !out.is_null() {
            unsafe {
                let selection = out.cast::<GhosttyRenderRowSelection>();
                ptr::read(core::ptr::addr_of!((*selection).size))
            }
        } else {
            0
        };

        let result = unsafe {
            ghostty_rust_render_row_get_data(
                key,
                raw,
                dirty,
                selection_present,
                selection_start,
                selection_end,
                out_size,
                out,
            )
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
pub unsafe extern "C" fn ghostty_rust_render_row_set_dirty(value: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_set(
    has_iterator: bool,
    has_row: bool,
    option: c_int,
    has_value: bool,
) -> c_int {
    if !has_iterator || !has_row || !has_value {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match option {
        RENDER_ROW_SET_DIRTY => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_row_cell_get_text(
    data: c_int,
    cell: u64,
    extra: *const u32,
    extra_len: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_ROW_CELL_DATA_RAW => ptr::write(out.cast::<u64>(), cell),
            RENDER_ROW_CELL_DATA_GRAPHEMES_LEN => {
                let len = if cell_has_text(cell) {
                    1usize.wrapping_add(if cell_has_grapheme(cell) {
                        extra_len
                    } else {
                        0
                    })
                } else {
                    0
                };
                ptr::write(out.cast::<u32>(), len as u32);
            }
            RENDER_ROW_CELL_DATA_GRAPHEMES_BUF => {
                if !cell_has_text(cell) {
                    return RENDER_RESULT_SUCCESS;
                }

                let buf = out.cast::<u32>();
                ptr::write(buf, cell_codepoint(cell));
                if cell_has_grapheme(cell) {
                    if extra.is_null() && extra_len > 0 {
                        return RENDER_RESULT_INVALID_VALUE;
                    }

                    let mut i = 0usize;
                    while i < extra_len {
                        ptr::write(buf.add(i + 1), ptr::read(extra.add(i)));
                        i += 1;
                    }
                }
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

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
