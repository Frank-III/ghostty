use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;

pub(crate) const TERMINAL_OPT_USERDATA: c_int = 0;
pub(crate) const TERMINAL_OPT_WRITE_PTY: c_int = 1;
pub(crate) const TERMINAL_OPT_BELL: c_int = 2;
pub(crate) const TERMINAL_OPT_ENQUIRY: c_int = 3;
pub(crate) const TERMINAL_OPT_XTVERSION: c_int = 4;
pub(crate) const TERMINAL_OPT_TITLE_CHANGED: c_int = 5;
pub(crate) const TERMINAL_OPT_SIZE: c_int = 6;
pub(crate) const TERMINAL_OPT_COLOR_SCHEME: c_int = 7;
pub(crate) const TERMINAL_OPT_DEVICE_ATTRIBUTES: c_int = 8;
pub(crate) const TERMINAL_OPT_TITLE: c_int = 9;
pub(crate) const TERMINAL_OPT_PWD: c_int = 10;
pub(crate) const TERMINAL_OPT_COLOR_FOREGROUND: c_int = 11;
pub(crate) const TERMINAL_OPT_COLOR_BACKGROUND: c_int = 12;
pub(crate) const TERMINAL_OPT_COLOR_CURSOR: c_int = 13;
pub(crate) const TERMINAL_OPT_COLOR_PALETTE: c_int = 14;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_STORAGE_LIMIT: c_int = 15;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_FILE: c_int = 16;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_TEMP_FILE: c_int = 17;
pub(crate) const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_SHARED_MEM: c_int = 18;
pub(crate) const TERMINAL_OPT_APC_MAX_BYTES: c_int = 19;
pub(crate) const TERMINAL_OPT_APC_MAX_BYTES_KITTY: c_int = 20;
pub(crate) const TERMINAL_OPT_SELECTION: c_int = 21;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPointCoordinate {
    pub(crate) x: u16,
    pub(crate) y: u32,
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_new(cols: u16, rows: u16) -> c_int {
    if cols == 0 || rows == 0 {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_reset(has_terminal: bool) -> bool {
    has_terminal
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_get(
    has_terminal: bool,
    data: c_int,
    has_out: bool,
) -> c_int {
    if !has_terminal || !has_out {
        return GHOSTTY_INVALID_VALUE;
    }

    match data {
        TERMINAL_DATA_COLS
        | TERMINAL_DATA_ROWS
        | TERMINAL_DATA_CURSOR_X
        | TERMINAL_DATA_CURSOR_Y
        | TERMINAL_DATA_CURSOR_PENDING_WRAP
        | TERMINAL_DATA_ACTIVE_SCREEN
        | TERMINAL_DATA_CURSOR_VISIBLE
        | TERMINAL_DATA_KITTY_KEYBOARD_FLAGS
        | TERMINAL_DATA_SCROLLBAR
        | TERMINAL_DATA_CURSOR_STYLE
        | TERMINAL_DATA_MOUSE_TRACKING
        | TERMINAL_DATA_TITLE
        | TERMINAL_DATA_PWD
        | TERMINAL_DATA_TOTAL_ROWS
        | TERMINAL_DATA_SCROLLBACK_ROWS
        | TERMINAL_DATA_WIDTH_PX
        | TERMINAL_DATA_HEIGHT_PX
        | TERMINAL_DATA_COLOR_FOREGROUND
        | TERMINAL_DATA_COLOR_BACKGROUND
        | TERMINAL_DATA_COLOR_CURSOR
        | TERMINAL_DATA_COLOR_PALETTE
        | TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_CURSOR_DEFAULT
        | TERMINAL_DATA_COLOR_PALETTE_DEFAULT
        | TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE
        | TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM
        | TERMINAL_DATA_KITTY_GRAPHICS
        | TERMINAL_DATA_SELECTION => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scalar(
    data: c_int,
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_pending_wrap: bool,
    active_screen: c_int,
    cursor_visible: bool,
    kitty_keyboard_flags: u8,
    mouse_tracking: bool,
    total_rows: usize,
    scrollback_rows: usize,
    width_px: u32,
    height_px: u32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    match data {
        TERMINAL_DATA_COLS => unsafe { write_out(out, cols) },
        TERMINAL_DATA_ROWS => unsafe { write_out(out, rows) },
        TERMINAL_DATA_CURSOR_X => unsafe { write_out(out, cursor_x) },
        TERMINAL_DATA_CURSOR_Y => unsafe { write_out(out, cursor_y) },
        TERMINAL_DATA_CURSOR_PENDING_WRAP => unsafe { write_out(out, cursor_pending_wrap) },
        TERMINAL_DATA_ACTIVE_SCREEN => unsafe { write_out(out, active_screen) },
        TERMINAL_DATA_CURSOR_VISIBLE => unsafe { write_out(out, cursor_visible) },
        TERMINAL_DATA_KITTY_KEYBOARD_FLAGS => unsafe { write_out(out, kitty_keyboard_flags) },
        TERMINAL_DATA_MOUSE_TRACKING => unsafe { write_out(out, mouse_tracking) },
        TERMINAL_DATA_TOTAL_ROWS => unsafe { write_out(out, total_rows) },
        TERMINAL_DATA_SCROLLBACK_ROWS => unsafe { write_out(out, scrollback_rows) },
        TERMINAL_DATA_WIDTH_PX => unsafe { write_out(out, width_px) },
        TERMINAL_DATA_HEIGHT_PX => unsafe { write_out(out, height_px) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scalar_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_pending_wrap: bool,
    active_screen: c_int,
    cursor_visible: bool,
    kitty_keyboard_flags: u8,
    mouse_tracking: bool,
    total_rows: usize,
    scrollback_rows: usize,
    width_px: u32,
    height_px: u32,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe {
            ghostty_rust_terminal_get_scalar(
                key,
                cols,
                rows,
                cursor_x,
                cursor_y,
                cursor_pending_wrap,
                active_screen,
                cursor_visible,
                kitty_keyboard_flags,
                mouse_tracking,
                total_rows,
                scrollback_rows,
                width_px,
                height_px,
                out,
            )
        };

        if result != GHOSTTY_SUCCESS {
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

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_string(
    data: c_int,
    ptr: *const u8,
    len: usize,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_TITLE | TERMINAL_DATA_PWD => unsafe { write_borrowed_string(out, ptr, len) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_style(
    data: c_int,
    style: *const GhosttyStyle,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_CURSOR_STYLE => unsafe { copy_style(out.cast::<GhosttyStyle>(), style) },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_scrollbar(
    data: c_int,
    total: u64,
    offset: u64,
    len: u64,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SCROLLBAR => unsafe {
            write_scrollbar(out.cast::<GhosttyTerminalScrollbar>(), total, offset, len)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_kitty_image(
    data: c_int,
    enabled: bool,
    storage_limit: u64,
    medium_file: bool,
    medium_temp_file: bool,
    medium_shared_mem: bool,
    out: *mut c_void,
) -> c_int {
    if !enabled {
        return GHOSTTY_NO_VALUE;
    }

    match data {
        TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT => unsafe { write_out(out, storage_limit) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE => unsafe { write_out(out, medium_file) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE => unsafe { write_out(out, medium_temp_file) },
        TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM => unsafe { write_out(out, medium_shared_mem) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_color(
    data: c_int,
    has_value: bool,
    r: u8,
    g: u8,
    b: u8,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_COLOR_FOREGROUND
        | TERMINAL_DATA_COLOR_BACKGROUND
        | TERMINAL_DATA_COLOR_CURSOR
        | TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT
        | TERMINAL_DATA_COLOR_CURSOR_DEFAULT => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        write_rgb_value(out.cast::<GhosttyColorRgb>(), r, g, b);
    }
    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_palette(
    data: c_int,
    palette: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_COLOR_PALETTE | TERMINAL_DATA_COLOR_PALETTE_DEFAULT => unsafe {
            copy_palette(out.cast::<GhosttyColorRgb>(), palette)
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_pointer(
    data: c_int,
    has_value: bool,
    value: *mut c_void,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_KITTY_GRAPHICS => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe {
        ptr::write(out.cast::<*mut c_void>(), value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_get_selection(
    data: c_int,
    has_value: bool,
    selection: *const GhosttySelection,
    out: *mut c_void,
) -> c_int {
    match data {
        TERMINAL_DATA_SELECTION => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe { copy_selection(out.cast::<GhosttySelection>(), selection) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_point_from_grid_ref(
    has_point: bool,
    coord: GhosttyPointCoordinate,
    out: *mut GhosttyPointCoordinate,
) -> c_int {
    if !has_point {
        return GHOSTTY_NO_VALUE;
    }

    if !out.is_null() {
        unsafe {
            ptr::write(out, coord);
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_point_from_grid_ref_input(
    has_terminal: bool,
    has_ref: bool,
) -> c_int {
    if !has_terminal || !has_ref {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_mode_get(
    has_terminal: bool,
    has_mode: bool,
    value: bool,
    out: *mut bool,
) -> c_int {
    if !has_terminal || !has_mode {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_mode_set(has_terminal: bool, has_mode: bool) -> c_int {
    if !has_terminal || !has_mode {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_resize(
    has_terminal: bool,
    cols: u16,
    rows: u16,
    cell_width_px: u32,
    cell_height_px: u32,
    out_width_px: *mut u32,
    out_height_px: *mut u32,
) -> c_int {
    if !has_terminal || cols == 0 || rows == 0 || out_width_px.is_null() || out_height_px.is_null()
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let width = (u64::from(cols) * u64::from(cell_width_px)).min(u64::from(u32::MAX)) as u32;
    let height = (u64::from(rows) * u64::from(cell_height_px)).min(u64::from(u32::MAX)) as u32;

    unsafe {
        ptr::write(out_width_px, width);
        ptr::write(out_height_px, height);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_terminal_set(has_terminal: bool, option: c_int) -> c_int {
    if !has_terminal {
        return GHOSTTY_INVALID_VALUE;
    }

    match option {
        TERMINAL_OPT_USERDATA
        | TERMINAL_OPT_WRITE_PTY
        | TERMINAL_OPT_BELL
        | TERMINAL_OPT_ENQUIRY
        | TERMINAL_OPT_XTVERSION
        | TERMINAL_OPT_TITLE_CHANGED
        | TERMINAL_OPT_SIZE
        | TERMINAL_OPT_COLOR_SCHEME
        | TERMINAL_OPT_DEVICE_ATTRIBUTES
        | TERMINAL_OPT_TITLE
        | TERMINAL_OPT_PWD
        | TERMINAL_OPT_COLOR_FOREGROUND
        | TERMINAL_OPT_COLOR_BACKGROUND
        | TERMINAL_OPT_COLOR_CURSOR
        | TERMINAL_OPT_COLOR_PALETTE
        | TERMINAL_OPT_KITTY_IMAGE_STORAGE_LIMIT
        | TERMINAL_OPT_KITTY_IMAGE_MEDIUM_FILE
        | TERMINAL_OPT_KITTY_IMAGE_MEDIUM_TEMP_FILE
        | TERMINAL_OPT_KITTY_IMAGE_MEDIUM_SHARED_MEM
        | TERMINAL_OPT_APC_MAX_BYTES
        | TERMINAL_OPT_APC_MAX_BYTES_KITTY
        | TERMINAL_OPT_SELECTION => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_string(
    value: *const GhosttyString,
    out_ptr: *mut *const u8,
    out_len: *mut usize,
) -> c_int {
    if out_ptr.is_null() || out_len.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (ptr, len) = if value.is_null() {
        (EMPTY_UTF8.as_ptr(), 0)
    } else {
        unsafe {
            (
                ptr::read(core::ptr::addr_of!((*value).ptr)),
                ptr::read(core::ptr::addr_of!((*value).len)),
            )
        }
    };

    unsafe {
        ptr::write(out_ptr, ptr);
        ptr::write(out_len, len);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_rgb(
    value: *const GhosttyColorRgb,
    out_has_value: *mut bool,
    out_rgb: *mut GhosttyColorRgb,
) -> c_int {
    if out_has_value.is_null() || out_rgb.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    let rgb = unsafe { ptr::read(value) };
    unsafe {
        ptr::write(out_has_value, true);
        write_rgb_value(out_rgb, rgb.r, rgb.g, rgb.b);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_palette(
    value: *const GhosttyColorRgb,
    out_has_value: *mut bool,
    out_palette: *mut *const GhosttyColorRgb,
) -> c_int {
    if out_has_value.is_null() || out_palette.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    unsafe {
        ptr::write(out_has_value, true);
        ptr::write(out_palette, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_u64_zero(
    value: *const u64,
    out_value: *mut u64,
) -> c_int {
    if out_value.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let decoded = if value.is_null() {
        0
    } else {
        unsafe { ptr::read(value) }
    };

    unsafe {
        ptr::write(out_value, decoded);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_bool_optional(
    value: *const bool,
    out_has_value: *mut bool,
    out_value: *mut bool,
) -> c_int {
    if out_has_value.is_null() || out_value.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    let decoded = unsafe { ptr::read(value) };
    unsafe {
        ptr::write(out_has_value, true);
        ptr::write(out_value, decoded);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_set_usize_optional(
    value: *const usize,
    out_has_value: *mut bool,
    out_value: *mut usize,
) -> c_int {
    if out_has_value.is_null() || out_value.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if value.is_null() {
        unsafe {
            ptr::write(out_has_value, false);
        }
        return GHOSTTY_SUCCESS;
    }

    let decoded = unsafe { ptr::read(value) };
    unsafe {
        ptr::write(out_has_value, true);
        ptr::write(out_value, decoded);
    }

    GHOSTTY_SUCCESS
}
