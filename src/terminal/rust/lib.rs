#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};
use core::mem;
use core::panic::PanicInfo;
use core::ptr;

const GHOSTTY_SUCCESS: c_int = 0;
const GHOSTTY_INVALID_VALUE: c_int = -2;
const GHOSTTY_OUT_OF_SPACE: c_int = -3;
const GHOSTTY_NO_VALUE: c_int = -4;

const OSC_COMMAND_INVALID: c_int = 0;
const OSC_DATA_CHANGE_WINDOW_TITLE_STR: c_int = 1;
const KEY_ENCODER_OPT_CURSOR_KEY_APPLICATION: c_int = 0;
const KEY_ENCODER_OPT_KEYPAD_KEY_APPLICATION: c_int = 1;
const KEY_ENCODER_OPT_IGNORE_KEYPAD_WITH_NUMLOCK: c_int = 2;
const KEY_ENCODER_OPT_ALT_ESC_PREFIX: c_int = 3;
const KEY_ENCODER_OPT_MODIFY_OTHER_KEYS_STATE_2: c_int = 4;
const KEY_ENCODER_OPT_MACOS_OPTION_AS_ALT: c_int = 6;
const KEY_ENCODER_OPT_BACKARROW_KEY_MODE: c_int = 7;
const OPTION_AS_ALT_FALSE: c_int = 0;
const OPTION_AS_ALT_TRUE: c_int = 1;
const OPTION_AS_ALT_LEFT: c_int = 2;
const OPTION_AS_ALT_RIGHT: c_int = 3;
const GHOSTTY_FOCUS_LOST: c_int = 1;
const SYS_OPT_USERDATA: c_int = 0;
const SYS_OPT_DECODE_PNG: c_int = 1;
const SYS_OPT_LOG: c_int = 2;
const TERMINAL_OPT_USERDATA: c_int = 0;
const TERMINAL_OPT_WRITE_PTY: c_int = 1;
const TERMINAL_OPT_BELL: c_int = 2;
const TERMINAL_OPT_ENQUIRY: c_int = 3;
const TERMINAL_OPT_XTVERSION: c_int = 4;
const TERMINAL_OPT_TITLE_CHANGED: c_int = 5;
const TERMINAL_OPT_SIZE: c_int = 6;
const TERMINAL_OPT_COLOR_SCHEME: c_int = 7;
const TERMINAL_OPT_DEVICE_ATTRIBUTES: c_int = 8;
const TERMINAL_OPT_TITLE: c_int = 9;
const TERMINAL_OPT_PWD: c_int = 10;
const TERMINAL_OPT_COLOR_FOREGROUND: c_int = 11;
const TERMINAL_OPT_COLOR_BACKGROUND: c_int = 12;
const TERMINAL_OPT_COLOR_CURSOR: c_int = 13;
const TERMINAL_OPT_COLOR_PALETTE: c_int = 14;
const TERMINAL_OPT_KITTY_IMAGE_STORAGE_LIMIT: c_int = 15;
const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_FILE: c_int = 16;
const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_TEMP_FILE: c_int = 17;
const TERMINAL_OPT_KITTY_IMAGE_MEDIUM_SHARED_MEM: c_int = 18;
const TERMINAL_OPT_APC_MAX_BYTES: c_int = 19;
const TERMINAL_OPT_APC_MAX_BYTES_KITTY: c_int = 20;
const TERMINAL_OPT_SELECTION: c_int = 21;
const FOCUS_GAINED: &[u8; 3] = b"\x1B[I";
const FOCUS_LOST: &[u8; 3] = b"\x1B[O";

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {}
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
pub unsafe extern "C" fn ghostty_rust_focus_encode(
    event: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let seq = if event == GHOSTTY_FOCUS_LOST {
        FOCUS_LOST
    } else {
        FOCUS_GAINED
    };

    unsafe {
        ptr::write(out_written, seq.len());
    }

    if out.is_null() || out_len < seq.len() {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        ptr::copy_nonoverlapping(seq.as_ptr(), out, seq.len());
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_rust_osc_command_type(has_command: bool, kind: c_int) -> c_int {
    if has_command {
        kind
    } else {
        OSC_COMMAND_INVALID
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_osc_command_data_string(
    data: c_int,
    has_value: bool,
    value: *const u8,
    out: *mut c_void,
) -> bool {
    if data != OSC_DATA_CHANGE_WINDOW_TITLE_STR || !has_value || value.is_null() || out.is_null() {
        return false;
    }

    unsafe {
        ptr::write(out.cast::<*const u8>(), value);
    }

    true
}

#[no_mangle]
pub extern "C" fn ghostty_rust_sys_set(option: c_int) -> c_int {
    match option {
        SYS_OPT_USERDATA | SYS_OPT_DECODE_PNG | SYS_OPT_LOG => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_bool(
    option: c_int,
    value: bool,
    out: *mut bool,
) {
    if out.is_null() {
        return;
    }

    match option {
        KEY_ENCODER_OPT_CURSOR_KEY_APPLICATION
        | KEY_ENCODER_OPT_KEYPAD_KEY_APPLICATION
        | KEY_ENCODER_OPT_IGNORE_KEYPAD_WITH_NUMLOCK
        | KEY_ENCODER_OPT_ALT_ESC_PREFIX
        | KEY_ENCODER_OPT_MODIFY_OTHER_KEYS_STATE_2
        | KEY_ENCODER_OPT_BACKARROW_KEY_MODE => unsafe {
            ptr::write(out, value);
        },
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_setopt_option_as_alt(
    option: c_int,
    value: c_int,
    out: *mut c_int,
) {
    if option != KEY_ENCODER_OPT_MACOS_OPTION_AS_ALT || out.is_null() {
        return;
    }

    match value {
        OPTION_AS_ALT_FALSE | OPTION_AS_ALT_TRUE | OPTION_AS_ALT_LEFT | OPTION_AS_ALT_RIGHT => unsafe {
            ptr::write(out, value);
        },
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_encoder_from_terminal(
    alt_esc_prefix: bool,
    cursor_key_application: bool,
    keypad_key_application: bool,
    backarrow_key_mode: bool,
    ignore_keypad_with_numlock: bool,
    modify_other_keys_state_2: bool,
    out_alt_esc_prefix: *mut bool,
    out_cursor_key_application: *mut bool,
    out_keypad_key_application: *mut bool,
    out_backarrow_key_mode: *mut bool,
    out_ignore_keypad_with_numlock: *mut bool,
    out_modify_other_keys_state_2: *mut bool,
    out_macos_option_as_alt: *mut c_int,
) {
    unsafe {
        ptr::write(out_alt_esc_prefix, alt_esc_prefix);
        ptr::write(out_cursor_key_application, cursor_key_application);
        ptr::write(out_keypad_key_application, keypad_key_application);
        ptr::write(out_backarrow_key_mode, backarrow_key_mode);
        ptr::write(out_ignore_keypad_with_numlock, ignore_keypad_with_numlock);
        ptr::write(out_modify_other_keys_state_2, modify_other_keys_state_2);
        ptr::write(out_macos_option_as_alt, OPTION_AS_ALT_FALSE);
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
    has_ref && has_page_list && !garbage
}

#[no_mangle]
pub extern "C" fn ghostty_rust_tracked_grid_ref_result(
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
    if !has_ref || !has_terminal || !same_terminal {
        return GHOSTTY_INVALID_VALUE;
    }

    GHOSTTY_SUCCESS
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPointCoordinate {
    x: u16,
    y: u32,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttySizeReportSize {
    rows: u16,
    columns: u16,
    cell_width: u32,
    cell_height: u32,
}

const SIZE_REPORT_MODE_2048: c_int = 0;
const SIZE_REPORT_CSI_14_T: c_int = 1;
const SIZE_REPORT_CSI_16_T: c_int = 2;
const SIZE_REPORT_CSI_18_T: c_int = 3;
const PASTE_START: &[u8; 6] = b"\x1b[200~";
const PASTE_END: &[u8; 6] = b"\x1b[201~";
const MODE_ANSI_MASK: u16 = 1 << 15;
const MODE_VALUE_MASK: u16 = 0x7fff;
const STYLE_COLOR_NONE: c_int = 0;
const STYLE_COLOR_PALETTE: c_int = 1;
const STYLE_COLOR_RGB: c_int = 2;
const ROW_DATA_WRAP: c_int = 1;
const ROW_DATA_WRAP_CONTINUATION: c_int = 2;
const ROW_DATA_GRAPHEME: c_int = 3;
const ROW_DATA_STYLED: c_int = 4;
const ROW_DATA_HYPERLINK: c_int = 5;
const ROW_DATA_SEMANTIC_PROMPT: c_int = 6;
const ROW_DATA_KITTY_VIRTUAL_PLACEHOLDER: c_int = 7;
const ROW_DATA_DIRTY: c_int = 8;
const CELL_CONTENT_TAG_CODEPOINT: u64 = 0;
const CELL_CONTENT_TAG_CODEPOINT_GRAPHEME: u64 = 1;
const CELL_CONTENT_TAG_BG_COLOR_PALETTE: u64 = 2;
const CELL_CONTENT_TAG_BG_COLOR_RGB: u64 = 3;
const CELL_DATA_CODEPOINT: c_int = 1;
const CELL_DATA_CONTENT_TAG: c_int = 2;
const CELL_DATA_WIDE: c_int = 3;
const CELL_DATA_HAS_TEXT: c_int = 4;
const CELL_DATA_HAS_STYLING: c_int = 5;
const CELL_DATA_STYLE_ID: c_int = 6;
const CELL_DATA_HAS_HYPERLINK: c_int = 7;
const CELL_DATA_PROTECTED: c_int = 8;
const CELL_DATA_SEMANTIC_CONTENT: c_int = 9;
const CELL_DATA_COLOR_PALETTE: c_int = 10;
const CELL_DATA_COLOR_RGB: c_int = 11;
const BUILD_INFO_SIMD: c_int = 1;
const BUILD_INFO_KITTY_GRAPHICS: c_int = 2;
const BUILD_INFO_TMUX_CONTROL_MODE: c_int = 3;
const BUILD_INFO_OPTIMIZE: c_int = 4;
const BUILD_INFO_VERSION_STRING: c_int = 5;
const BUILD_INFO_VERSION_MAJOR: c_int = 6;
const BUILD_INFO_VERSION_MINOR: c_int = 7;
const BUILD_INFO_VERSION_PATCH: c_int = 8;
const BUILD_INFO_VERSION_PRE: c_int = 9;
const BUILD_INFO_VERSION_BUILD: c_int = 10;
const TERMINAL_DATA_COLS: c_int = 1;
const TERMINAL_DATA_ROWS: c_int = 2;
const TERMINAL_DATA_CURSOR_X: c_int = 3;
const TERMINAL_DATA_CURSOR_Y: c_int = 4;
const TERMINAL_DATA_CURSOR_PENDING_WRAP: c_int = 5;
const TERMINAL_DATA_ACTIVE_SCREEN: c_int = 6;
const TERMINAL_DATA_CURSOR_VISIBLE: c_int = 7;
const TERMINAL_DATA_KITTY_KEYBOARD_FLAGS: c_int = 8;
const TERMINAL_DATA_SCROLLBAR: c_int = 9;
const TERMINAL_DATA_CURSOR_STYLE: c_int = 10;
const TERMINAL_DATA_MOUSE_TRACKING: c_int = 11;
const TERMINAL_DATA_TITLE: c_int = 12;
const TERMINAL_DATA_PWD: c_int = 13;
const TERMINAL_DATA_TOTAL_ROWS: c_int = 14;
const TERMINAL_DATA_SCROLLBACK_ROWS: c_int = 15;
const TERMINAL_DATA_WIDTH_PX: c_int = 16;
const TERMINAL_DATA_HEIGHT_PX: c_int = 17;
const TERMINAL_DATA_COLOR_FOREGROUND: c_int = 18;
const TERMINAL_DATA_COLOR_BACKGROUND: c_int = 19;
const TERMINAL_DATA_COLOR_CURSOR: c_int = 20;
const TERMINAL_DATA_COLOR_PALETTE: c_int = 21;
const TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT: c_int = 22;
const TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT: c_int = 23;
const TERMINAL_DATA_COLOR_CURSOR_DEFAULT: c_int = 24;
const TERMINAL_DATA_COLOR_PALETTE_DEFAULT: c_int = 25;
const TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT: c_int = 26;
const TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE: c_int = 27;
const TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE: c_int = 28;
const TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM: c_int = 29;
const TERMINAL_DATA_KITTY_GRAPHICS: c_int = 30;
const TERMINAL_DATA_SELECTION: c_int = 31;
const RENDER_RESULT_SUCCESS: c_int = GHOSTTY_SUCCESS;
const RENDER_RESULT_INVALID_VALUE: c_int = GHOSTTY_INVALID_VALUE;
const RENDER_DATA_COLS: c_int = 1;
const RENDER_DATA_ROWS: c_int = 2;
const RENDER_DATA_DIRTY: c_int = 3;
const RENDER_DATA_COLOR_BACKGROUND: c_int = 5;
const RENDER_DATA_COLOR_FOREGROUND: c_int = 6;
const RENDER_DATA_COLOR_CURSOR: c_int = 7;
const RENDER_DATA_COLOR_CURSOR_HAS_VALUE: c_int = 8;
const RENDER_DATA_COLOR_PALETTE: c_int = 9;
const RENDER_DATA_CURSOR_VISUAL_STYLE: c_int = 10;
const RENDER_DATA_CURSOR_VISIBLE: c_int = 11;
const RENDER_DATA_CURSOR_BLINKING: c_int = 12;
const RENDER_DATA_CURSOR_PASSWORD_INPUT: c_int = 13;
const RENDER_DATA_CURSOR_VIEWPORT_HAS_VALUE: c_int = 14;
const RENDER_DATA_CURSOR_VIEWPORT_X: c_int = 15;
const RENDER_DATA_CURSOR_VIEWPORT_Y: c_int = 16;
const RENDER_DATA_CURSOR_VIEWPORT_WIDE_TAIL: c_int = 17;
const RENDER_ROW_CELL_DATA_RAW: c_int = 1;
const RENDER_ROW_CELL_DATA_STYLE: c_int = 2;
const RENDER_ROW_CELL_DATA_GRAPHEMES_LEN: c_int = 3;
const RENDER_ROW_CELL_DATA_GRAPHEMES_BUF: c_int = 4;
const RENDER_ROW_CELL_DATA_BG_COLOR: c_int = 5;
const RENDER_ROW_CELL_DATA_FG_COLOR: c_int = 6;
const RENDER_ROW_CELL_DATA_SELECTED: c_int = 7;
const RENDER_ROW_DATA_DIRTY: c_int = 1;
const RENDER_ROW_DATA_RAW: c_int = 2;
const RENDER_ROW_DATA_CELLS: c_int = 3;
const RENDER_ROW_DATA_SELECTION: c_int = 4;
const RENDER_STATE_SET_DIRTY: c_int = 0;
const RENDER_ROW_SET_DIRTY: c_int = 0;
const RENDER_CURSOR_STYLE_BAR: c_int = 0;
const RENDER_CURSOR_STYLE_BLOCK: c_int = 1;
const RENDER_CURSOR_STYLE_UNDERLINE: c_int = 2;
const RENDER_CURSOR_STYLE_BLOCK_HOLLOW: c_int = 3;
const SELECTION_ORDER_FORWARD: c_int = 0;
const SELECTION_ORDER_REVERSE: c_int = 1;
const SELECTION_ORDER_MIRRORED_FORWARD: c_int = 2;
const SELECTION_ORDER_MIRRORED_REVERSE: c_int = 3;
const KEY_EVENT_UTF8_PTR_OFFSET: usize = 0;
const KEY_EVENT_UTF8_LEN_OFFSET: usize = 8;
const KEY_EVENT_ACTION_OFFSET: usize = 16;
const KEY_EVENT_KEY_OFFSET: usize = 20;
const KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET: usize = 24;
const KEY_EVENT_MODS_OFFSET: usize = 28;
const KEY_EVENT_CONSUMED_MODS_OFFSET: usize = 30;
const KEY_EVENT_COMPOSING_OFFSET: usize = 32;
const MOUSE_EVENT_EVENT_OFFSET: usize = 16;
const MOUSE_EVENT_ACTION_OFFSET: usize = 0;
const MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET: usize = 4;
const MOUSE_EVENT_BUTTON_TAG_OFFSET: usize = 8;
const MOUSE_EVENT_POS_OFFSET: usize = 12;
const MOUSE_EVENT_MODS_OFFSET: usize = 20;
const MOUSE_ACTION_PRESS: c_int = 0;
const MOUSE_ACTION_RELEASE: c_int = 1;
const MOUSE_ACTION_MOTION: c_int = 2;
const MOUSE_BUTTON_LEFT: c_int = 1;
const MOUSE_BUTTON_RIGHT: c_int = 2;
const MOUSE_BUTTON_MIDDLE: c_int = 3;
const MOUSE_BUTTON_FOUR: c_int = 4;
const MOUSE_BUTTON_FIVE: c_int = 5;
const MOUSE_BUTTON_SIX: c_int = 6;
const MOUSE_BUTTON_SEVEN: c_int = 7;
const MOUSE_BUTTON_EIGHT: c_int = 8;
const MOUSE_BUTTON_NINE: c_int = 9;
const MOUSE_TRACKING_NONE: c_int = 0;
const MOUSE_TRACKING_X10: c_int = 1;
const MOUSE_TRACKING_NORMAL: c_int = 2;
const MOUSE_TRACKING_BUTTON: c_int = 3;
const MOUSE_TRACKING_ANY: c_int = 4;
const MOUSE_FORMAT_X10: c_int = 0;
const MOUSE_FORMAT_UTF8: c_int = 1;
const MOUSE_FORMAT_SGR: c_int = 2;
const MOUSE_FORMAT_URXVT: c_int = 3;
const MOUSE_FORMAT_SGR_PIXELS: c_int = 4;
const MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED: c_int = 3;
const MOUSE_ENCODER_OPT_TRACK_LAST_CELL: c_int = 4;
const KITTY_PLACEMENT_LAYER_ALL: c_int = 0;
const KITTY_PLACEMENT_LAYER_BELOW_BG: c_int = 1;
const KITTY_PLACEMENT_LAYER_BELOW_TEXT: c_int = 2;
const KITTY_PLACEMENT_LAYER_ABOVE_TEXT: c_int = 3;
const KITTY_PLACEMENT_ITERATOR_OPTION_LAYER: c_int = 0;
const KITTY_IMAGE_DATA_ID: c_int = 1;
const KITTY_IMAGE_DATA_NUMBER: c_int = 2;
const KITTY_IMAGE_DATA_WIDTH: c_int = 3;
const KITTY_IMAGE_DATA_HEIGHT: c_int = 4;
const KITTY_IMAGE_DATA_FORMAT: c_int = 5;
const KITTY_IMAGE_DATA_COMPRESSION: c_int = 6;
const KITTY_IMAGE_DATA_PTR: c_int = 7;
const KITTY_IMAGE_DATA_LEN: c_int = 8;
const KITTY_PLACEMENT_DATA_IMAGE_ID: c_int = 1;
const KITTY_PLACEMENT_DATA_PLACEMENT_ID: c_int = 2;
const KITTY_PLACEMENT_DATA_IS_VIRTUAL: c_int = 3;
const KITTY_PLACEMENT_DATA_X_OFFSET: c_int = 4;
const KITTY_PLACEMENT_DATA_Y_OFFSET: c_int = 5;
const KITTY_PLACEMENT_DATA_SOURCE_X: c_int = 6;
const KITTY_PLACEMENT_DATA_SOURCE_Y: c_int = 7;
const KITTY_PLACEMENT_DATA_SOURCE_WIDTH: c_int = 8;
const KITTY_PLACEMENT_DATA_SOURCE_HEIGHT: c_int = 9;
const KITTY_PLACEMENT_DATA_COLUMNS: c_int = 10;
const KITTY_PLACEMENT_DATA_ROWS: c_int = 11;
const KITTY_PLACEMENT_DATA_Z: c_int = 12;
const MOD_SHIFT: u16 = 1 << 0;
const MOD_CTRL: u16 = 1 << 1;
const MOD_ALT: u16 = 1 << 2;
static EMPTY_UTF8: [u8; 0] = [];

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
    size: usize,
    start_x: u16,
    end_x: u16,
}

#[repr(C)]
pub struct GhosttyRenderStateColors {
    size: usize,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor: GhosttyColorRgb,
    cursor_has_value: bool,
    palette: [GhosttyColorRgb; 256],
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyMousePosition {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy)]
struct GhosttyMouseSize {
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
}

#[derive(Clone, Copy)]
struct GhosttyMouseCell {
    x: u16,
    y: u32,
}

#[derive(Clone, Copy)]
struct GhosttyMousePixels {
    x: i32,
    y: i32,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_event(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_mode(value, current, out, last_cell_present);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_format(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        mouse_encoder_setopt_mode(value, current, out, last_cell_present);
    }
}

unsafe fn mouse_encoder_setopt_mode(
    value: c_int,
    current: c_int,
    out: *mut c_int,
    last_cell_present: *mut bool,
) {
    if out.is_null() {
        return;
    }

    unsafe {
        if value != current && !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
        ptr::write(out, value);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_size(
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
    out_screen_width: *mut u32,
    out_screen_height: *mut u32,
    out_cell_width: *mut u32,
    out_cell_height: *mut u32,
    out_padding_top: *mut u32,
    out_padding_bottom: *mut u32,
    out_padding_right: *mut u32,
    out_padding_left: *mut u32,
    last_cell_present: *mut bool,
) {
    unsafe {
        if !out_screen_width.is_null() {
            ptr::write(out_screen_width, screen_width);
        }
        if !out_screen_height.is_null() {
            ptr::write(out_screen_height, screen_height);
        }
        if !out_cell_width.is_null() {
            ptr::write(out_cell_width, cell_width);
        }
        if !out_cell_height.is_null() {
            ptr::write(out_cell_height, cell_height);
        }
        if !out_padding_top.is_null() {
            ptr::write(out_padding_top, padding_top);
        }
        if !out_padding_bottom.is_null() {
            ptr::write(out_padding_bottom, padding_bottom);
        }
        if !out_padding_right.is_null() {
            ptr::write(out_padding_right, padding_right);
        }
        if !out_padding_left.is_null() {
            ptr::write(out_padding_left, padding_left);
        }
        if !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_bool(
    option: c_int,
    value: bool,
    any_button_pressed: *mut bool,
    track_last_cell: *mut bool,
    last_cell_present: *mut bool,
) {
    unsafe {
        match option {
            MOUSE_ENCODER_OPT_ANY_BUTTON_PRESSED => {
                if !any_button_pressed.is_null() {
                    ptr::write(any_button_pressed, value);
                }
            }
            MOUSE_ENCODER_OPT_TRACK_LAST_CELL => {
                if !track_last_cell.is_null() {
                    ptr::write(track_last_cell, value);
                }
                if !value && !last_cell_present.is_null() {
                    ptr::write(last_cell_present, false);
                }
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_from_terminal(
    event: c_int,
    format: c_int,
    out_event: *mut c_int,
    out_format: *mut c_int,
    last_cell_present: *mut bool,
) {
    unsafe {
        if !out_event.is_null() {
            ptr::write(out_event, event);
        }
        if !out_format.is_null() {
            ptr::write(out_format, format);
        }
        if !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_reset(last_cell_present: *mut bool) {
    if last_cell_present.is_null() {
        return;
    }

    unsafe {
        ptr::write(last_cell_present, false);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyGridRef {
    size: usize,
    node: *mut c_void,
    x: u16,
    y: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttySelection {
    size: usize,
    start: GhosttyGridRef,
    end: GhosttyGridRef,
    rectangle: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyTerminalScrollbar {
    total: u64,
    offset: u64,
    len: u64,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_write(
    has_value: bool,
    selection: *const GhosttySelection,
    out: *mut GhosttySelection,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    if !has_value {
        return GHOSTTY_NO_VALUE;
    }

    unsafe { copy_selection(out, selection) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_write_order(
    order: c_int,
    out: *mut c_int,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    match order {
        SELECTION_ORDER_FORWARD
        | SELECTION_ORDER_REVERSE
        | SELECTION_ORDER_MIRRORED_FORWARD
        | SELECTION_ORDER_MIRRORED_REVERSE => {}
        _ => return GHOSTTY_INVALID_VALUE,
    }

    unsafe {
        ptr::write(out, order);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_write_bool(value: bool, out: *mut bool) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(out, value);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_selection_equal(
    terminal: *mut c_void,
    a: *const GhosttySelection,
    b: *const GhosttySelection,
    out: *mut bool,
) -> c_int {
    if terminal.is_null() || a.is_null() || b.is_null() || out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let a_start = unsafe { ptr::read(core::ptr::addr_of!((*a).start)) };
    let a_end = unsafe { ptr::read(core::ptr::addr_of!((*a).end)) };
    let b_start = unsafe { ptr::read(core::ptr::addr_of!((*b).start)) };
    let b_end = unsafe { ptr::read(core::ptr::addr_of!((*b).end)) };
    if !grid_ref_valid(a_start)
        || !grid_ref_valid(a_end)
        || !grid_ref_valid(b_start)
        || !grid_ref_valid(b_end)
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let a_rectangle = unsafe { ptr::read(core::ptr::addr_of!((*a).rectangle)) };
    let b_rectangle = unsafe { ptr::read(core::ptr::addr_of!((*b).rectangle)) };
    unsafe {
        ptr::write(
            out,
            grid_ref_equal(a_start, b_start)
                && grid_ref_equal(a_end, b_end)
                && a_rectangle == b_rectangle,
        );
    }

    GHOSTTY_SUCCESS
}

fn grid_ref_valid(value: GhosttyGridRef) -> bool {
    !value.node.is_null()
}

fn grid_ref_equal(a: GhosttyGridRef, b: GhosttyGridRef) -> bool {
    a.node == b.node && a.x == b.x && a.y == b.y
}

unsafe fn copy_grid_ref(dst: *mut GhosttyGridRef, src: *const GhosttyGridRef) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).node),
            ptr::read(core::ptr::addr_of!((*src).node)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).x),
            ptr::read(core::ptr::addr_of!((*src).x)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).y),
            ptr::read(core::ptr::addr_of!((*src).y)),
        );
    }

    GHOSTTY_SUCCESS
}

unsafe fn copy_selection(dst: *mut GhosttySelection, src: *const GhosttySelection) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );

        let result = copy_grid_ref(
            core::ptr::addr_of_mut!((*dst).start),
            core::ptr::addr_of!((*src).start),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }

        let result = copy_grid_ref(
            core::ptr::addr_of_mut!((*dst).end),
            core::ptr::addr_of!((*src).end),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }

        ptr::write(
            core::ptr::addr_of_mut!((*dst).rectangle),
            ptr::read(core::ptr::addr_of!((*src).rectangle)),
        );
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_source_rect(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    out_x: *mut u32,
    out_y: *mut u32,
    out_width: *mut u32,
    out_height: *mut u32,
) -> c_int {
    if out_x.is_null() || out_y.is_null() || out_width.is_null() || out_height.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (x, y, width, height) = kitty_source_rect(
        image_width,
        image_height,
        source_x,
        source_y,
        source_width,
        source_height,
    );

    unsafe {
        ptr::write(out_x, x);
        ptr::write(out_y, y);
        ptr::write(out_width, width);
        ptr::write(out_height, height);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_render_info(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    x_offset: u32,
    y_offset: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    viewport_col: i32,
    viewport_row: i32,
    viewport_visible: bool,
    out_pixel_width: *mut u32,
    out_pixel_height: *mut u32,
    out_grid_cols: *mut u32,
    out_grid_rows: *mut u32,
    out_viewport_col: *mut i32,
    out_viewport_row: *mut i32,
    out_viewport_visible: *mut bool,
    out_source_x: *mut u32,
    out_source_y: *mut u32,
    out_source_width: *mut u32,
    out_source_height: *mut u32,
) -> c_int {
    if out_pixel_width.is_null()
        || out_pixel_height.is_null()
        || out_grid_cols.is_null()
        || out_grid_rows.is_null()
        || out_viewport_col.is_null()
        || out_viewport_row.is_null()
        || out_viewport_visible.is_null()
        || out_source_x.is_null()
        || out_source_y.is_null()
        || out_source_width.is_null()
        || out_source_height.is_null()
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let (pixel_width, pixel_height) = kitty_pixel_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );
    let (grid_cols, grid_rows) = kitty_grid_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        x_offset,
        y_offset,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );
    let (source_rect_x, source_rect_y, source_rect_width, source_rect_height) = kitty_source_rect(
        image_width,
        image_height,
        source_x,
        source_y,
        source_width,
        source_height,
    );

    unsafe {
        ptr::write(out_pixel_width, pixel_width);
        ptr::write(out_pixel_height, pixel_height);
        ptr::write(out_grid_cols, grid_cols);
        ptr::write(out_grid_rows, grid_rows);
        ptr::write(out_viewport_col, viewport_col);
        ptr::write(out_viewport_row, viewport_row);
        ptr::write(out_viewport_visible, viewport_visible);
        ptr::write(out_source_x, source_rect_x);
        ptr::write(out_source_y, source_rect_y);
        ptr::write(out_source_width, source_rect_width);
        ptr::write(out_source_height, source_rect_height);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_rect(
    start_node: *mut c_void,
    start_x: u16,
    start_y: u16,
    end_node: *mut c_void,
    end_y: u16,
    grid_cols_minus_one: u32,
    terminal_cols_minus_one: u16,
    out: *mut GhosttySelection,
) -> c_int {
    if start_node.is_null() || end_node.is_null() || out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let end_x = u32::from(start_x)
        .wrapping_add(grid_cols_minus_one)
        .min(u32::from(terminal_cols_minus_one)) as u16;

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*out).size),
            mem::size_of::<GhosttySelection>(),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*out).start.size),
            mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(core::ptr::addr_of_mut!((*out).start.node), start_node);
        ptr::write(core::ptr::addr_of_mut!((*out).start.x), start_x);
        ptr::write(core::ptr::addr_of_mut!((*out).start.y), start_y);
        ptr::write(
            core::ptr::addr_of_mut!((*out).end.size),
            mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(core::ptr::addr_of_mut!((*out).end.node), end_node);
        ptr::write(core::ptr::addr_of_mut!((*out).end.x), end_x);
        ptr::write(core::ptr::addr_of_mut!((*out).end.y), end_y);
        ptr::write(core::ptr::addr_of_mut!((*out).rectangle), true);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_pixel_size(
    image_width: u32,
    image_height: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    out_width: *mut u32,
    out_height: *mut u32,
) -> c_int {
    if out_width.is_null() || out_height.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (width, height) = kitty_pixel_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );

    unsafe {
        ptr::write(out_width, width);
        ptr::write(out_height, height);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_grid_size(
    image_width: u32,
    image_height: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    x_offset: u32,
    y_offset: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    out_cols: *mut u32,
    out_rows: *mut u32,
) -> c_int {
    if out_cols.is_null() || out_rows.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (cols, rows) = kitty_grid_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        x_offset,
        y_offset,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );

    unsafe {
        ptr::write(out_cols, cols);
        ptr::write(out_rows, rows);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_viewport_pos(
    pin_screen_x: i32,
    pin_screen_y: i32,
    viewport_screen_y: i32,
    grid_rows: i32,
    terminal_rows: u16,
    out_col: *mut i32,
    out_row: *mut i32,
    out_visible: *mut bool,
) -> c_int {
    if out_col.is_null() || out_row.is_null() || out_visible.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let row = pin_screen_y.wrapping_sub(viewport_screen_y);
    let terminal_rows_i32 = i32::from(terminal_rows);
    let visible = row.wrapping_add(grid_rows) > 0 && row < terminal_rows_i32;

    unsafe {
        ptr::write(out_col, pin_screen_x);
        ptr::write(out_row, row);
        ptr::write(out_visible, visible);
    }

    GHOSTTY_SUCCESS
}

fn kitty_pixel_size(
    image_width: u32,
    image_height: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
) -> (u32, u32) {
    let width = if source_width > 0 {
        source_width
    } else {
        image_width
    };
    let height = if source_height > 0 {
        source_height
    } else {
        image_height
    };

    if placement_columns == 0 && placement_rows == 0 {
        return (width, height);
    }

    let cell_width = nonzero_u32_div(terminal_width_px, u32::from(terminal_cols));
    let cell_height = nonzero_u32_div(terminal_height_px, u32::from(terminal_rows));

    if placement_columns > 0 && placement_rows > 0 {
        return (
            cell_width.wrapping_mul(placement_columns),
            cell_height.wrapping_mul(placement_rows),
        );
    }

    if placement_columns > 0 {
        let calc_width = cell_width.wrapping_mul(placement_columns);
        let aspect = (height as f64) / (width as f64);
        return (calc_width, round_f64_to_u32((calc_width as f64) * aspect));
    }

    let calc_height = cell_height.wrapping_mul(placement_rows);
    let aspect = (width as f64) / (height as f64);
    (round_f64_to_u32((calc_height as f64) * aspect), calc_height)
}

fn kitty_grid_size(
    image_width: u32,
    image_height: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    x_offset: u32,
    y_offset: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
) -> (u32, u32) {
    if placement_columns > 0 && placement_rows > 0 {
        return (placement_columns, placement_rows);
    }

    let (pixel_width, pixel_height) = kitty_pixel_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );
    let cell_width = nonzero_u32_div(terminal_width_px, u32::from(terminal_cols));
    let cell_height = nonzero_u32_div(terminal_height_px, u32::from(terminal_rows));

    (
        div_ceil_u32(pixel_width.wrapping_add(x_offset), cell_width),
        div_ceil_u32(pixel_height.wrapping_add(y_offset), cell_height),
    )
}

fn div_ceil_u32(numerator: u32, denominator: u32) -> u32 {
    let quotient = numerator.checked_div(denominator).unwrap_or(0);
    let remainder = numerator.checked_rem(denominator).unwrap_or(0);
    quotient.wrapping_add(if remainder == 0 { 0 } else { 1 })
}

fn round_f64_to_u32(value: f64) -> u32 {
    if value <= 0.0 {
        0
    } else if value >= u32::MAX as f64 {
        u32::MAX
    } else {
        (value + 0.5) as u32
    }
}

fn kitty_source_rect(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
) -> (u32, u32, u32, u32) {
    let x = source_x.min(image_width);
    let y = source_y.min(image_height);
    let width = if source_width > 0 {
        source_width
    } else {
        image_width
    }
    .min(image_width.saturating_sub(x));
    let height = if source_height > 0 {
        source_height
    } else {
        image_height
    }
    .min(image_height.saturating_sub(y));

    (x, y, width, height)
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_layer_matches(layer: c_int, z: i32) -> bool {
    let below_bg_boundary = i32::MIN / 2;
    match layer {
        KITTY_PLACEMENT_LAYER_ALL => true,
        KITTY_PLACEMENT_LAYER_BELOW_BG => z < below_bg_boundary,
        KITTY_PLACEMENT_LAYER_BELOW_TEXT => z >= below_bg_boundary && z < 0,
        KITTY_PLACEMENT_LAYER_ABOVE_TEXT => z >= 0,
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_iterator_set(
    option: c_int,
    layer: c_int,
    out_layer: *mut c_int,
) -> c_int {
    if out_layer.is_null() || option != KITTY_PLACEMENT_ITERATOR_OPTION_LAYER {
        return GHOSTTY_INVALID_VALUE;
    }

    match layer {
        KITTY_PLACEMENT_LAYER_ALL
        | KITTY_PLACEMENT_LAYER_BELOW_BG
        | KITTY_PLACEMENT_LAYER_BELOW_TEXT
        | KITTY_PLACEMENT_LAYER_ABOVE_TEXT => unsafe {
            ptr::write(out_layer, layer);
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_image_get(
    data: c_int,
    id: u32,
    number: u32,
    width: u32,
    height: u32,
    format: c_int,
    compression: c_int,
    data_ptr: *const u8,
    data_len: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        kitty_image_get_write(
            data,
            id,
            number,
            width,
            height,
            format,
            compression,
            data_ptr,
            data_len,
            out,
        )
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_image_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    id: u32,
    number: u32,
    width: u32,
    height: u32,
    format: c_int,
    compression: c_int,
    data_ptr: *const u8,
    data_len: usize,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut index = 0usize;
    while index < count {
        let key = unsafe { ptr::read(keys.add(index)) };
        let out = unsafe { ptr::read(values.add(index)) };
        let result = unsafe {
            kitty_image_get_write(
                key,
                id,
                number,
                width,
                height,
                format,
                compression,
                data_ptr,
                data_len,
                out,
            )
        };
        if result != GHOSTTY_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, index);
                }
            }
            return result;
        }

        index = index.wrapping_add(1);
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    GHOSTTY_SUCCESS
}

unsafe fn kitty_image_get_write(
    data: c_int,
    id: u32,
    number: u32,
    width: u32,
    height: u32,
    format: c_int,
    compression: c_int,
    data_ptr: *const u8,
    data_len: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match data {
            KITTY_IMAGE_DATA_ID => ptr::write(out.cast::<u32>(), id),
            KITTY_IMAGE_DATA_NUMBER => ptr::write(out.cast::<u32>(), number),
            KITTY_IMAGE_DATA_WIDTH => ptr::write(out.cast::<u32>(), width),
            KITTY_IMAGE_DATA_HEIGHT => ptr::write(out.cast::<u32>(), height),
            KITTY_IMAGE_DATA_FORMAT => ptr::write(out.cast::<c_int>(), format),
            KITTY_IMAGE_DATA_COMPRESSION => ptr::write(out.cast::<c_int>(), compression),
            KITTY_IMAGE_DATA_PTR => ptr::write(out.cast::<*const u8>(), data_ptr),
            KITTY_IMAGE_DATA_LEN => ptr::write(out.cast::<usize>(), data_len),
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_get(
    data: c_int,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        kitty_placement_get_write(
            data,
            image_id,
            placement_id,
            is_virtual,
            x_offset,
            y_offset,
            source_x,
            source_y,
            source_width,
            source_height,
            columns,
            rows,
            z,
            out,
        )
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut index = 0usize;
    while index < count {
        let key = unsafe { ptr::read(keys.add(index)) };
        let out = unsafe { ptr::read(values.add(index)) };
        let result = unsafe {
            kitty_placement_get_write(
                key,
                image_id,
                placement_id,
                is_virtual,
                x_offset,
                y_offset,
                source_x,
                source_y,
                source_width,
                source_height,
                columns,
                rows,
                z,
                out,
            )
        };
        if result != GHOSTTY_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, index);
                }
            }
            return result;
        }

        index = index.wrapping_add(1);
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    GHOSTTY_SUCCESS
}

unsafe fn kitty_placement_get_write(
    data: c_int,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match data {
            KITTY_PLACEMENT_DATA_IMAGE_ID => ptr::write(out.cast::<u32>(), image_id),
            KITTY_PLACEMENT_DATA_PLACEMENT_ID => ptr::write(out.cast::<u32>(), placement_id),
            KITTY_PLACEMENT_DATA_IS_VIRTUAL => ptr::write(out.cast::<bool>(), is_virtual),
            KITTY_PLACEMENT_DATA_X_OFFSET => ptr::write(out.cast::<u32>(), x_offset),
            KITTY_PLACEMENT_DATA_Y_OFFSET => ptr::write(out.cast::<u32>(), y_offset),
            KITTY_PLACEMENT_DATA_SOURCE_X => ptr::write(out.cast::<u32>(), source_x),
            KITTY_PLACEMENT_DATA_SOURCE_Y => ptr::write(out.cast::<u32>(), source_y),
            KITTY_PLACEMENT_DATA_SOURCE_WIDTH => ptr::write(out.cast::<u32>(), source_width),
            KITTY_PLACEMENT_DATA_SOURCE_HEIGHT => ptr::write(out.cast::<u32>(), source_height),
            KITTY_PLACEMENT_DATA_COLUMNS => ptr::write(out.cast::<u32>(), columns),
            KITTY_PLACEMENT_DATA_ROWS => ptr::write(out.cast::<u32>(), rows),
            KITTY_PLACEMENT_DATA_Z => ptr::write(out.cast::<i32>(), z),
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encode(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    pos: GhosttyMousePosition,
    tracking_mode: c_int,
    format: c_int,
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
    any_button_pressed: bool,
    track_last_cell: bool,
    last_cell_present: bool,
    last_cell_x: u16,
    last_cell_y: u32,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
    next_last_cell_present: *mut bool,
    next_last_cell_x: *mut u16,
    next_last_cell_y: *mut u32,
) -> c_int {
    let size = GhosttyMouseSize {
        screen_width,
        screen_height,
        cell_width,
        cell_height,
        padding_top,
        padding_bottom,
        padding_right,
        padding_left,
    };

    if size.cell_width == 0 || size.cell_height == 0 {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(next_last_cell_present, last_cell_present);
        ptr::write(next_last_cell_x, last_cell_x);
        ptr::write(next_last_cell_y, last_cell_y);
    }

    if !mouse_should_report(action, button_present, button, tracking_mode) {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    }

    if action != MOUSE_ACTION_RELEASE && mouse_pos_out_of_viewport(pos, size) {
        if !mouse_event_sends_motion(tracking_mode) || !any_button_pressed {
            unsafe {
                ptr::write(out_written, 0);
            }
            return GHOSTTY_SUCCESS;
        }
    }

    let cell = mouse_pos_to_cell(pos, size);
    if action == MOUSE_ACTION_MOTION
        && format != MOUSE_FORMAT_SGR_PIXELS
        && track_last_cell
        && last_cell_present
        && last_cell_x == cell.x
        && last_cell_y == cell.y
    {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    }

    if track_last_cell {
        unsafe {
            ptr::write(next_last_cell_present, true);
            ptr::write(next_last_cell_x, cell.x);
            ptr::write(next_last_cell_y, cell.y);
        }
    }

    let Some(button_code) =
        mouse_button_code(action, button_present, button, mods, tracking_mode, format)
    else {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    };

    if format == MOUSE_FORMAT_X10 && (cell.x > 222 || cell.y > 222) {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_SUCCESS;
    }

    let Some(required) = mouse_sequence_len(format, action, button_code, cell, pos, size) else {
        return GHOSTTY_INVALID_VALUE;
    };

    unsafe {
        ptr::write(out_written, required);
    }

    if required > 0 && (out.is_null() || out_len < required) {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_mouse_sequence(format, action, button_code, cell, pos, size, out);
    }

    GHOSTTY_SUCCESS
}

fn mouse_should_report(
    action: c_int,
    button_present: bool,
    button: c_int,
    tracking_mode: c_int,
) -> bool {
    match tracking_mode {
        MOUSE_TRACKING_NONE => false,
        MOUSE_TRACKING_X10 => {
            action == MOUSE_ACTION_PRESS
                && button_present
                && (button == MOUSE_BUTTON_LEFT
                    || button == MOUSE_BUTTON_MIDDLE
                    || button == MOUSE_BUTTON_RIGHT)
        }
        MOUSE_TRACKING_NORMAL => action != MOUSE_ACTION_MOTION,
        MOUSE_TRACKING_BUTTON => button_present,
        MOUSE_TRACKING_ANY => true,
        _ => false,
    }
}

fn mouse_event_sends_motion(tracking_mode: c_int) -> bool {
    tracking_mode == MOUSE_TRACKING_BUTTON || tracking_mode == MOUSE_TRACKING_ANY
}

fn mouse_pos_out_of_viewport(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> bool {
    pos.x < 0.0
        || pos.y < 0.0
        || pos.x > size.screen_width as f32
        || pos.y > size.screen_height as f32
}

fn mouse_grid_size(size: GhosttyMouseSize) -> GhosttyMouseCell {
    let terminal_width = size
        .screen_width
        .saturating_sub(size.padding_left.saturating_add(size.padding_right));
    let terminal_height = size
        .screen_height
        .saturating_sub(size.padding_top.saturating_add(size.padding_bottom));
    let columns = nonzero_u32_div(terminal_width, size.cell_width).max(1);
    let rows = nonzero_u32_div(terminal_height, size.cell_height).max(1);

    GhosttyMouseCell {
        x: columns.min(u32::from(u16::MAX)) as u16,
        y: rows,
    }
}

fn mouse_pos_to_cell(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> GhosttyMouseCell {
    let grid = mouse_grid_size(size);
    let term_x = (pos.x - size.padding_left as f32).max(0.0);
    let term_y = (pos.y - size.padding_top as f32).max(0.0);
    let col = (term_x / size.cell_width as f32) as u32;
    let row = (term_y / size.cell_height as f32) as u32;

    GhosttyMouseCell {
        x: col.min(u32::from(grid.x.saturating_sub(1))) as u16,
        y: row.min(grid.y.saturating_sub(1)),
    }
}

fn nonzero_u32_div(numerator: u32, denominator: u32) -> u32 {
    numerator.checked_div(denominator).unwrap_or(0)
}

fn mouse_pos_to_pixels(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> GhosttyMousePixels {
    GhosttyMousePixels {
        x: round_f32_to_i32(pos.x - size.padding_left as f32),
        y: round_f32_to_i32(pos.y - size.padding_top as f32),
    }
}

fn round_f32_to_i32(value: f32) -> i32 {
    if value >= 0.0 {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    }
}

fn mouse_button_code(
    action: c_int,
    button_present: bool,
    button: c_int,
    mods: u16,
    tracking_mode: c_int,
    format: c_int,
) -> Option<u8> {
    let mut acc = if !button_present {
        3u8
    } else if action == MOUSE_ACTION_RELEASE
        && format != MOUSE_FORMAT_SGR
        && format != MOUSE_FORMAT_SGR_PIXELS
    {
        3u8
    } else {
        match button {
            MOUSE_BUTTON_LEFT => 0,
            MOUSE_BUTTON_MIDDLE => 1,
            MOUSE_BUTTON_RIGHT => 2,
            MOUSE_BUTTON_FOUR => 64,
            MOUSE_BUTTON_FIVE => 65,
            MOUSE_BUTTON_SIX => 66,
            MOUSE_BUTTON_SEVEN => 67,
            MOUSE_BUTTON_EIGHT => 128,
            MOUSE_BUTTON_NINE => 129,
            _ => return None,
        }
    };

    if tracking_mode != MOUSE_TRACKING_X10 {
        if (mods & MOD_SHIFT) != 0 {
            acc = acc.wrapping_add(4);
        }
        if (mods & MOD_ALT) != 0 {
            acc = acc.wrapping_add(8);
        }
        if (mods & MOD_CTRL) != 0 {
            acc = acc.wrapping_add(16);
        }
    }

    if action == MOUSE_ACTION_MOTION {
        acc = acc.wrapping_add(32);
    }

    Some(acc)
}

fn mouse_sequence_len(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
) -> Option<usize> {
    match format {
        MOUSE_FORMAT_X10 => Some(b"\x1B[M".len() + 3),
        MOUSE_FORMAT_UTF8 => {
            Some(b"\x1B[M".len() + 1 + utf8_len(u32::from(cell.x) + 33)? + utf8_len(cell.y + 33)?)
        }
        MOUSE_FORMAT_SGR => Some(
            b"\x1B[<".len()
                + decimal_len(u64::from(button_code))
                + 1
                + decimal_len(u64::from(cell.x) + 1)
                + 1
                + decimal_len(u64::from(cell.y) + 1)
                + mouse_action_suffix_len(action),
        ),
        MOUSE_FORMAT_URXVT => Some(
            b"\x1B[".len()
                + decimal_len(u64::from(button_code) + 32)
                + 1
                + decimal_len(u64::from(cell.x) + 1)
                + 1
                + decimal_len(u64::from(cell.y) + 1)
                + 1,
        ),
        MOUSE_FORMAT_SGR_PIXELS => {
            let pixels = mouse_pos_to_pixels(pos, size);
            Some(
                b"\x1B[<".len()
                    + decimal_len(u64::from(button_code))
                    + 1
                    + signed_decimal_len(pixels.x)
                    + 1
                    + signed_decimal_len(pixels.y)
                    + mouse_action_suffix_len(action),
            )
        }
        _ => None,
    }
}

fn mouse_action_suffix_len(_: c_int) -> usize {
    1
}

unsafe fn write_mouse_sequence(
    format: c_int,
    action: c_int,
    button_code: u8,
    cell: GhosttyMouseCell,
    pos: GhosttyMousePosition,
    size: GhosttyMouseSize,
    out: *mut u8,
) {
    let mut offset = 0usize;
    match format {
        MOUSE_FORMAT_X10 => unsafe {
            write_bytes(out, &mut offset, b"\x1B[M");
            write_byte(out, &mut offset, button_code.wrapping_add(32));
            write_byte(out, &mut offset, (cell.x as u8).wrapping_add(33));
            write_byte(out, &mut offset, (cell.y as u8).wrapping_add(33));
        },
        MOUSE_FORMAT_UTF8 => unsafe {
            write_bytes(out, &mut offset, b"\x1B[M");
            write_byte(out, &mut offset, button_code.wrapping_add(32));
            write_utf8(out, &mut offset, u32::from(cell.x) + 33);
            write_utf8(out, &mut offset, cell.y + 33);
        },
        MOUSE_FORMAT_SGR => unsafe {
            write_bytes(out, &mut offset, b"\x1B[<");
            write_decimal(out, &mut offset, u64::from(button_code));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.x) + 1);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.y) + 1);
            write_mouse_action_suffix(out, &mut offset, action);
        },
        MOUSE_FORMAT_URXVT => unsafe {
            write_bytes(out, &mut offset, b"\x1B[");
            write_decimal(out, &mut offset, u64::from(button_code) + 32);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.x) + 1);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(cell.y) + 1);
            write_bytes(out, &mut offset, b"M");
        },
        MOUSE_FORMAT_SGR_PIXELS => unsafe {
            let pixels = mouse_pos_to_pixels(pos, size);
            write_bytes(out, &mut offset, b"\x1B[<");
            write_decimal(out, &mut offset, u64::from(button_code));
            write_bytes(out, &mut offset, b";");
            write_signed_decimal(out, &mut offset, pixels.x);
            write_bytes(out, &mut offset, b";");
            write_signed_decimal(out, &mut offset, pixels.y);
            write_mouse_action_suffix(out, &mut offset, action);
        },
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_size_report_encode(
    style: c_int,
    size: GhosttySizeReportSize,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let Some(required) = size_report_len(style, size) else {
        unsafe {
            ptr::write(out_written, 0);
        }
        return GHOSTTY_INVALID_VALUE;
    };

    unsafe {
        ptr::write(out_written, required);
    }

    if out.is_null() || out_len < required {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_size_report(style, size, out);
    }

    GHOSTTY_SUCCESS
}

fn width_pixels(size: GhosttySizeReportSize) -> u64 {
    u64::from(size.columns) * u64::from(size.cell_width)
}

fn height_pixels(size: GhosttySizeReportSize) -> u64 {
    u64::from(size.rows) * u64::from(size.cell_height)
}

fn size_report_len(style: c_int, size: GhosttySizeReportSize) -> Option<usize> {
    let rows = u64::from(size.rows);
    let columns = u64::from(size.columns);
    let height = height_pixels(size);
    let width = width_pixels(size);

    match style {
        SIZE_REPORT_MODE_2048 => Some(
            b"\x1B[48;".len()
                + decimal_len(rows)
                + 1
                + decimal_len(columns)
                + 1
                + decimal_len(height)
                + 1
                + decimal_len(width)
                + 1,
        ),
        SIZE_REPORT_CSI_14_T => {
            Some(b"\x1b[4;".len() + decimal_len(height) + 1 + decimal_len(width) + 1)
        }
        SIZE_REPORT_CSI_16_T => Some(
            b"\x1b[6;".len()
                + decimal_len(u64::from(size.cell_height))
                + 1
                + decimal_len(u64::from(size.cell_width))
                + 1,
        ),
        SIZE_REPORT_CSI_18_T => {
            Some(b"\x1b[8;".len() + decimal_len(rows) + 1 + decimal_len(columns) + 1)
        }
        _ => None,
    }
}

fn decimal_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 10 {
        value /= 10;
        len += 1;
    }
    len
}

fn signed_decimal_len(value: i32) -> usize {
    if value < 0 {
        1 + decimal_len((-i64::from(value)) as u64)
    } else {
        decimal_len(value as u64)
    }
}

fn utf8_len(codepoint: u32) -> Option<usize> {
    match codepoint {
        0x0000..=0x007f => Some(1),
        0x0080..=0x07ff => Some(2),
        0x0800..=0xffff => Some(3),
        0x1_0000..=0x10_ffff => Some(4),
        _ => None,
    }
}

unsafe fn write_size_report(style: c_int, size: GhosttySizeReportSize, out: *mut u8) {
    let mut offset = 0usize;
    let rows = u64::from(size.rows);
    let columns = u64::from(size.columns);

    match style {
        SIZE_REPORT_MODE_2048 => unsafe {
            write_bytes(out, &mut offset, b"\x1B[48;");
            write_decimal(out, &mut offset, rows);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, columns);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, height_pixels(size));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, width_pixels(size));
            write_bytes(out, &mut offset, b"t");
        },
        SIZE_REPORT_CSI_14_T => unsafe {
            write_bytes(out, &mut offset, b"\x1b[4;");
            write_decimal(out, &mut offset, height_pixels(size));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, width_pixels(size));
            write_bytes(out, &mut offset, b"t");
        },
        SIZE_REPORT_CSI_16_T => unsafe {
            write_bytes(out, &mut offset, b"\x1b[6;");
            write_decimal(out, &mut offset, u64::from(size.cell_height));
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, u64::from(size.cell_width));
            write_bytes(out, &mut offset, b"t");
        },
        SIZE_REPORT_CSI_18_T => unsafe {
            write_bytes(out, &mut offset, b"\x1b[8;");
            write_decimal(out, &mut offset, rows);
            write_bytes(out, &mut offset, b";");
            write_decimal(out, &mut offset, columns);
            write_bytes(out, &mut offset, b"t");
        },
        _ => {}
    }
}

unsafe fn write_bytes(out: *mut u8, offset: &mut usize, bytes: &[u8]) {
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = unsafe { ptr::read(bytes.as_ptr().add(i)) };
        unsafe {
            ptr::write(out.add(*offset + i), byte);
        }
        i += 1;
    }
    *offset += bytes.len();
}

unsafe fn write_byte(out: *mut u8, offset: &mut usize, byte: u8) {
    unsafe {
        ptr::write(out.add(*offset), byte);
    }
    *offset += 1;
}

unsafe fn write_decimal(out: *mut u8, offset: &mut usize, mut value: u64) {
    let mut reversed = [0u8; 20];
    let mut len = 0usize;

    loop {
        let digit = (value % 10) as u8;
        unsafe {
            ptr::write(reversed.as_mut_ptr().add(len), b'0' + digit);
        }
        len += 1;
        value /= 10;

        if value == 0 {
            break;
        }
    }

    while len > 0 {
        len -= 1;
        let byte = unsafe { ptr::read(reversed.as_ptr().add(len)) };
        unsafe {
            ptr::write(out.add(*offset), byte);
        }
        *offset += 1;
    }
}

unsafe fn write_signed_decimal(out: *mut u8, offset: &mut usize, value: i32) {
    if value < 0 {
        unsafe {
            write_byte(out, offset, b'-');
            write_decimal(out, offset, (-i64::from(value)) as u64);
        }
    } else {
        unsafe {
            write_decimal(out, offset, value as u64);
        }
    }
}

unsafe fn write_utf8(out: *mut u8, offset: &mut usize, codepoint: u32) {
    if codepoint <= 0x7f {
        unsafe {
            write_byte(out, offset, codepoint as u8);
        }
    } else if codepoint <= 0x7ff {
        unsafe {
            write_byte(out, offset, 0xc0 | ((codepoint >> 6) as u8));
            write_byte(out, offset, 0x80 | ((codepoint & 0x3f) as u8));
        }
    } else if codepoint <= 0xffff {
        unsafe {
            write_byte(out, offset, 0xe0 | ((codepoint >> 12) as u8));
            write_byte(out, offset, 0x80 | (((codepoint >> 6) & 0x3f) as u8));
            write_byte(out, offset, 0x80 | ((codepoint & 0x3f) as u8));
        }
    } else {
        unsafe {
            write_byte(out, offset, 0xf0 | ((codepoint >> 18) as u8));
            write_byte(out, offset, 0x80 | (((codepoint >> 12) & 0x3f) as u8));
            write_byte(out, offset, 0x80 | (((codepoint >> 6) & 0x3f) as u8));
            write_byte(out, offset, 0x80 | ((codepoint & 0x3f) as u8));
        }
    }
}

unsafe fn write_mouse_action_suffix(out: *mut u8, offset: &mut usize, action: c_int) {
    unsafe {
        write_byte(
            out,
            offset,
            if action == MOUSE_ACTION_RELEASE {
                b'm'
            } else {
                b'M'
            },
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_paste_is_safe(data: *const u8, len: usize) -> bool {
    if data.is_null() {
        return true;
    }

    let mut offset = 0usize;
    while offset < len {
        let byte = unsafe { ptr::read(data.add(offset)) };
        if byte == b'\n' || unsafe { matches_bytes_at(data, len, offset, PASTE_END) } {
            return false;
        }

        offset += 1;
    }

    true
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_paste_encode(
    data: *mut u8,
    data_len: usize,
    bracketed: bool,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    let actual_data_len = if data.is_null() { 0 } else { data_len };

    if !data.is_null() {
        let mut offset = 0usize;
        while offset < actual_data_len {
            let byte = unsafe { ptr::read(data.add(offset)) };
            if paste_strip_byte(byte) || (!bracketed && byte == b'\n') {
                unsafe {
                    ptr::write(
                        data.add(offset),
                        if !bracketed && byte == b'\n' {
                            b'\r'
                        } else {
                            b' '
                        },
                    );
                }
            }
            offset += 1;
        }
    }

    let prefix_len = if bracketed { PASTE_START.len() } else { 0 };
    let suffix_len = if bracketed { PASTE_END.len() } else { 0 };
    let total = prefix_len + actual_data_len + suffix_len;

    unsafe {
        ptr::write(out_written, total);
    }

    if out_len < total || (total > 0 && out.is_null()) {
        return GHOSTTY_OUT_OF_SPACE;
    }

    let mut out_offset = 0usize;
    if bracketed {
        unsafe {
            write_bytes(out, &mut out_offset, PASTE_START);
        }
    }
    if actual_data_len > 0 {
        unsafe {
            copy_data_bytes(out, &mut out_offset, data, actual_data_len);
        }
    }
    if bracketed {
        unsafe {
            write_bytes(out, &mut out_offset, PASTE_END);
        }
    }

    GHOSTTY_SUCCESS
}

fn paste_strip_byte(byte: u8) -> bool {
    matches!(
        byte,
        0x00 | 0x08
            | 0x05
            | 0x04
            | 0x1B
            | 0x7F
            | 0x03
            | 0x1C
            | 0x15
            | 0x1A
            | 0x11
            | 0x13
            | 0x17
            | 0x16
            | 0x12
            | 0x0F
    )
}

unsafe fn matches_bytes_at(data: *const u8, len: usize, offset: usize, bytes: &[u8]) -> bool {
    if len - offset < bytes.len() {
        return false;
    }

    let mut i = 0usize;
    while i < bytes.len() {
        let actual = unsafe { ptr::read(data.add(offset + i)) };
        let expected = unsafe { ptr::read(bytes.as_ptr().add(i)) };
        if actual != expected {
            return false;
        }
        i += 1;
    }

    true
}

unsafe fn copy_data_bytes(out: *mut u8, offset: &mut usize, data: *const u8, len: usize) {
    let mut i = 0usize;
    while i < len {
        let byte = unsafe { ptr::read(data.add(i)) };
        unsafe {
            ptr::write(out.add(*offset + i), byte);
        }
        i += 1;
    }
    *offset += len;
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mode_report_encode(
    tag: u16,
    state: c_int,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    if !(0..=4).contains(&state) {
        return GHOSTTY_INVALID_VALUE;
    }

    let value = u64::from(tag & MODE_VALUE_MASK);
    let ansi = (tag & MODE_ANSI_MASK) != 0;
    let state_value = state as u64;
    let total = mode_report_len(value, ansi, state_value);

    unsafe {
        ptr::write(out_written, total);
    }

    if out.is_null() || out_len < total {
        return GHOSTTY_OUT_OF_SPACE;
    }

    unsafe {
        write_mode_report(out, value, ansi, state_value);
    }

    GHOSTTY_SUCCESS
}

fn mode_report_len(value: u64, ansi: bool, state: u64) -> usize {
    b"\x1B[".len()
        + if ansi { 0 } else { 1 }
        + decimal_len(value)
        + 1
        + decimal_len(state)
        + b"$y".len()
}

unsafe fn write_mode_report(out: *mut u8, value: u64, ansi: bool, state: u64) {
    let mut offset = 0usize;
    unsafe {
        write_bytes(out, &mut offset, b"\x1B[");
        if !ansi {
            write_bytes(out, &mut offset, b"?");
        }
        write_decimal(out, &mut offset, value);
        write_bytes(out, &mut offset, b";");
        write_decimal(out, &mut offset, state);
        write_bytes(out, &mut offset, b"$y");
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyString {
    ptr: *const u8,
    len: usize,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_build_info(data: c_int, out: *mut c_void) -> c_int {
    match data {
        BUILD_INFO_SIMD => unsafe { write_out(out, env_flag(env!("GHOSTTY_VT_SIMD"))) },
        BUILD_INFO_KITTY_GRAPHICS => unsafe {
            write_out(out, env_flag(env!("GHOSTTY_VT_KITTY_GRAPHICS")))
        },
        BUILD_INFO_TMUX_CONTROL_MODE => unsafe {
            write_out(out, env_flag(env!("GHOSTTY_VT_TMUX_CONTROL_MODE")))
        },
        BUILD_INFO_OPTIMIZE => unsafe {
            write_out(out, optimize_value(env!("GHOSTTY_VT_OPTIMIZE")))
        },
        BUILD_INFO_VERSION_STRING => unsafe {
            write_string(out, env!("GHOSTTY_VT_VERSION_STRING").as_bytes())
        },
        BUILD_INFO_VERSION_MAJOR => unsafe {
            write_out(out, env_usize(env!("GHOSTTY_VT_VERSION_MAJOR").as_bytes()))
        },
        BUILD_INFO_VERSION_MINOR => unsafe {
            write_out(out, env_usize(env!("GHOSTTY_VT_VERSION_MINOR").as_bytes()))
        },
        BUILD_INFO_VERSION_PATCH => unsafe {
            write_out(out, env_usize(env!("GHOSTTY_VT_VERSION_PATCH").as_bytes()))
        },
        BUILD_INFO_VERSION_PRE => unsafe {
            write_string(out, env!("GHOSTTY_VT_VERSION_PRE").as_bytes())
        },
        BUILD_INFO_VERSION_BUILD => unsafe {
            write_string(out, env!("GHOSTTY_VT_VERSION_BUILD").as_bytes())
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

fn env_flag(value: &str) -> bool {
    value.as_bytes() == b"1"
}

fn optimize_value(value: &str) -> c_int {
    match value.as_bytes() {
        b"debug" => 0,
        b"release_safe" => 1,
        b"release_small" => 2,
        b"release_fast" => 3,
        _ => 0,
    }
}

fn env_usize(bytes: &[u8]) -> usize {
    let mut value = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = unsafe { ptr::read(bytes.as_ptr().add(i)) };
        value = value
            .saturating_mul(10)
            .saturating_add(usize::from(byte.saturating_sub(b'0')));
        i += 1;
    }
    value
}

fn struct_sized_field_fits<T>(size: usize, offset: usize) -> bool {
    size >= offset.saturating_add(mem::size_of::<T>())
}

unsafe fn write_string(out: *mut c_void, bytes: &'static [u8]) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), bytes.as_ptr());
        ptr::write(core::ptr::addr_of_mut!((*string).len), bytes.len());
    }
}

unsafe fn write_borrowed_string(out: *mut c_void, ptr: *const u8, len: usize) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), ptr);
        ptr::write(core::ptr::addr_of_mut!((*string).len), len);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_action(event: *mut c_void, action: c_int) {
    unsafe {
        ptr::write(
            key_event_field::<c_int>(event, KEY_EVENT_ACTION_OFFSET),
            action,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_action(event: *mut c_void) -> c_int {
    unsafe { ptr::read(key_event_field::<c_int>(event, KEY_EVENT_ACTION_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_key(event: *mut c_void, key: c_int) {
    unsafe {
        ptr::write(key_event_field::<c_int>(event, KEY_EVENT_KEY_OFFSET), key);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_key(event: *mut c_void) -> c_int {
    unsafe { ptr::read(key_event_field::<c_int>(event, KEY_EVENT_KEY_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_mods(event: *mut c_void, mods: u16) {
    unsafe {
        ptr::write(key_event_field::<u16>(event, KEY_EVENT_MODS_OFFSET), mods);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_mods(event: *mut c_void) -> u16 {
    unsafe { ptr::read(key_event_field::<u16>(event, KEY_EVENT_MODS_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_consumed_mods(event: *mut c_void, mods: u16) {
    unsafe {
        ptr::write(
            key_event_field::<u16>(event, KEY_EVENT_CONSUMED_MODS_OFFSET),
            mods,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_consumed_mods(event: *mut c_void) -> u16 {
    unsafe {
        ptr::read(key_event_field::<u16>(
            event,
            KEY_EVENT_CONSUMED_MODS_OFFSET,
        ))
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_composing(event: *mut c_void, composing: bool) {
    unsafe {
        ptr::write(
            key_event_field::<bool>(event, KEY_EVENT_COMPOSING_OFFSET),
            composing,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_composing(event: *mut c_void) -> bool {
    unsafe { ptr::read(key_event_field::<bool>(event, KEY_EVENT_COMPOSING_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_utf8(
    event: *mut c_void,
    utf8: *const u8,
    len: usize,
) {
    let ptr = if utf8.is_null() {
        EMPTY_UTF8.as_ptr()
    } else {
        utf8
    };
    unsafe {
        ptr::write(
            key_event_field::<*const u8>(event, KEY_EVENT_UTF8_PTR_OFFSET),
            ptr,
        );
        ptr::write(
            key_event_field::<usize>(event, KEY_EVENT_UTF8_LEN_OFFSET),
            len,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_utf8(
    event: *mut c_void,
    len: *mut usize,
) -> *const u8 {
    let utf8_len = unsafe { ptr::read(key_event_field::<usize>(event, KEY_EVENT_UTF8_LEN_OFFSET)) };
    if !len.is_null() {
        unsafe {
            ptr::write(len, utf8_len);
        }
    }

    if utf8_len == 0 {
        ptr::null()
    } else {
        unsafe {
            ptr::read(key_event_field::<*const u8>(
                event,
                KEY_EVENT_UTF8_PTR_OFFSET,
            ))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_set_unshifted_codepoint(
    event: *mut c_void,
    codepoint: u32,
) {
    unsafe {
        ptr::write(
            key_event_field::<u32>(event, KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET),
            codepoint & 0x001f_ffff,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_key_event_get_unshifted_codepoint(event: *mut c_void) -> u32 {
    unsafe {
        ptr::read(key_event_field::<u32>(
            event,
            KEY_EVENT_UNSHIFTED_CODEPOINT_OFFSET,
        ))
    }
}

unsafe fn key_event_field<T>(event: *mut c_void, offset: usize) -> *mut T {
    unsafe { event.cast::<u8>().add(offset).cast::<T>() }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_action(event: *mut c_void, action: c_int) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_ACTION_OFFSET),
            action,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_action(event: *mut c_void) -> c_int {
    unsafe { ptr::read(mouse_event_field::<c_int>(event, MOUSE_EVENT_ACTION_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_button(event: *mut c_void, button: c_int) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET),
            button,
        );
        ptr::write(
            mouse_event_field::<u32>(event, MOUSE_EVENT_BUTTON_TAG_OFFSET),
            1,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_clear_button(event: *mut c_void) {
    unsafe {
        ptr::write(
            mouse_event_field::<c_int>(event, MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET),
            0,
        );
        ptr::write(
            mouse_event_field::<u32>(event, MOUSE_EVENT_BUTTON_TAG_OFFSET),
            0,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_button(
    event: *mut c_void,
    out: *mut c_int,
) -> bool {
    let tag = unsafe {
        ptr::read(mouse_event_field::<u32>(
            event,
            MOUSE_EVENT_BUTTON_TAG_OFFSET,
        ))
    };
    if tag == 0 {
        return false;
    }

    if !out.is_null() {
        unsafe {
            ptr::write(
                out,
                ptr::read(mouse_event_field::<c_int>(
                    event,
                    MOUSE_EVENT_BUTTON_PAYLOAD_OFFSET,
                )),
            );
        }
    }

    true
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_mods(event: *mut c_void, mods: u16) {
    unsafe {
        ptr::write(
            mouse_event_field::<u16>(event, MOUSE_EVENT_MODS_OFFSET),
            mods,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_mods(event: *mut c_void) -> u16 {
    unsafe { ptr::read(mouse_event_field::<u16>(event, MOUSE_EVENT_MODS_OFFSET)) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_set_position(
    event: *mut c_void,
    pos: GhosttyMousePosition,
) {
    unsafe {
        ptr::write(
            mouse_event_field::<GhosttyMousePosition>(event, MOUSE_EVENT_POS_OFFSET),
            pos,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_event_get_position(
    event: *mut c_void,
) -> GhosttyMousePosition {
    unsafe {
        ptr::read(mouse_event_field::<GhosttyMousePosition>(
            event,
            MOUSE_EVENT_POS_OFFSET,
        ))
    }
}

unsafe fn mouse_event_field<T>(event: *mut c_void, offset: usize) -> *mut T {
    unsafe {
        event
            .cast::<u8>()
            .add(MOUSE_EVENT_EVENT_OFFSET + offset)
            .cast::<T>()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_cell_get(cell: u64, data: c_int, out: *mut c_void) -> c_int {
    unsafe { cell_get(cell, data, out) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_cell_get_multi(
    cell: u64,
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe { cell_get(cell, key, out) };
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

unsafe fn cell_get(cell: u64, data: c_int, out: *mut c_void) -> c_int {
    match data {
        CELL_DATA_CODEPOINT => unsafe { write_out(out, cell_codepoint(cell)) },
        CELL_DATA_CONTENT_TAG => unsafe { write_out(out, cell_content_tag(cell) as c_int) },
        CELL_DATA_WIDE => unsafe { write_out(out, ((cell >> 42) & 0x3) as c_int) },
        CELL_DATA_HAS_TEXT => unsafe { write_out(out, cell_has_text(cell)) },
        CELL_DATA_HAS_STYLING => unsafe { write_out(out, cell_style_id(cell) != 0) },
        CELL_DATA_STYLE_ID => unsafe { write_out(out, cell_style_id(cell)) },
        CELL_DATA_HAS_HYPERLINK => unsafe { write_out(out, cell_bit(cell, 45)) },
        CELL_DATA_PROTECTED => unsafe { write_out(out, cell_bit(cell, 44)) },
        CELL_DATA_SEMANTIC_CONTENT => unsafe { write_out(out, ((cell >> 46) & 0x3) as c_int) },
        CELL_DATA_COLOR_PALETTE => unsafe { write_out(out, cell_content(cell) as u8) },
        CELL_DATA_COLOR_RGB => unsafe { write_cell_rgb(out, cell_content(cell)) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

fn cell_content_tag(cell: u64) -> u64 {
    cell & 0x3
}

fn cell_content(cell: u64) -> u64 {
    (cell >> 2) & 0x00ff_ffff
}

fn cell_codepoint(cell: u64) -> u32 {
    match cell_content_tag(cell) {
        CELL_CONTENT_TAG_CODEPOINT | CELL_CONTENT_TAG_CODEPOINT_GRAPHEME => {
            (cell_content(cell) & 0x001f_ffff) as u32
        }
        _ => 0,
    }
}

fn cell_has_text(cell: u64) -> bool {
    match cell_content_tag(cell) {
        CELL_CONTENT_TAG_CODEPOINT | CELL_CONTENT_TAG_CODEPOINT_GRAPHEME => {
            cell_codepoint(cell) != 0
        }
        _ => false,
    }
}

fn cell_has_grapheme(cell: u64) -> bool {
    cell_content_tag(cell) == CELL_CONTENT_TAG_CODEPOINT_GRAPHEME
}

fn cell_style_id(cell: u64) -> u16 {
    ((cell >> 26) & 0xffff) as u16
}

fn cell_bit(cell: u64, bit: u32) -> bool {
    ((cell >> bit) & 1) != 0
}

unsafe fn write_cell_rgb(out: *mut c_void, content: u64) {
    let rgb = out.cast::<GhosttyColorRgb>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*rgb).r), content as u8);
        ptr::write(core::ptr::addr_of_mut!((*rgb).g), (content >> 8) as u8);
        ptr::write(core::ptr::addr_of_mut!((*rgb).b), (content >> 16) as u8);
    }
}

unsafe fn write_style_color_rgb(
    color: *const GhosttyStyleColor,
    palette_color: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    if color.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match ptr::read(core::ptr::addr_of!((*color).tag)) {
            STYLE_COLOR_NONE => GHOSTTY_INVALID_VALUE,
            STYLE_COLOR_PALETTE => {
                write_rgb(out, palette_color);
                GHOSTTY_SUCCESS
            }
            STYLE_COLOR_RGB => {
                let rgb = core::ptr::addr_of!((*color).value.rgb);
                write_rgb(out, rgb);
                GHOSTTY_SUCCESS
            }
            _ => GHOSTTY_INVALID_VALUE,
        }
    }
}

unsafe fn copy_style_color(dst: *mut GhosttyStyleColor, src: *const GhosttyStyleColor) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        let tag = ptr::read(core::ptr::addr_of!((*src).tag));
        ptr::write(core::ptr::addr_of_mut!((*dst).tag), tag);
        ptr::write(core::ptr::addr_of_mut!((*dst).value.padding), 0);

        match tag {
            STYLE_COLOR_NONE => {}
            STYLE_COLOR_PALETTE => {
                ptr::write(
                    core::ptr::addr_of_mut!((*dst).value.palette),
                    ptr::read(core::ptr::addr_of!((*src).value.palette)),
                );
            }
            STYLE_COLOR_RGB => {
                write_rgb(
                    core::ptr::addr_of_mut!((*dst).value.rgb).cast::<c_void>(),
                    core::ptr::addr_of!((*src).value.rgb),
                );
            }
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}

unsafe fn copy_style(dst: *mut GhosttyStyle, src: *const GhosttyStyle) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).fg_color),
            core::ptr::addr_of!((*src).fg_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).bg_color),
            core::ptr::addr_of!((*src).bg_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).underline_color),
            core::ptr::addr_of!((*src).underline_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        ptr::write(
            core::ptr::addr_of_mut!((*dst).bold),
            ptr::read(core::ptr::addr_of!((*src).bold)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).italic),
            ptr::read(core::ptr::addr_of!((*src).italic)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).faint),
            ptr::read(core::ptr::addr_of!((*src).faint)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).blink),
            ptr::read(core::ptr::addr_of!((*src).blink)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).inverse),
            ptr::read(core::ptr::addr_of!((*src).inverse)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).invisible),
            ptr::read(core::ptr::addr_of!((*src).invisible)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).strikethrough),
            ptr::read(core::ptr::addr_of!((*src).strikethrough)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).overline),
            ptr::read(core::ptr::addr_of!((*src).overline)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).underline),
            ptr::read(core::ptr::addr_of!((*src).underline)),
        );
    }

    GHOSTTY_SUCCESS
}

unsafe fn write_scrollbar(out: *mut GhosttyTerminalScrollbar, total: u64, offset: u64, len: u64) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).total), total);
        ptr::write(core::ptr::addr_of_mut!((*out).offset), offset);
        ptr::write(core::ptr::addr_of_mut!((*out).len), len);
    }
}

unsafe fn write_rgb_value(out: *mut GhosttyColorRgb, r: u8, g: u8, b: u8) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).r), r);
        ptr::write(core::ptr::addr_of_mut!((*out).g), g);
        ptr::write(core::ptr::addr_of_mut!((*out).b), b);
    }
}

unsafe fn copy_palette(dst: *mut GhosttyColorRgb, src: *const GhosttyColorRgb) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < 256 {
        unsafe {
            let src_rgb = src.add(i);
            write_rgb_value(
                dst.add(i),
                ptr::read(core::ptr::addr_of!((*src_rgb).r)),
                ptr::read(core::ptr::addr_of!((*src_rgb).g)),
                ptr::read(core::ptr::addr_of!((*src_rgb).b)),
            );
        }
        i += 1;
    }

    GHOSTTY_SUCCESS
}

unsafe fn write_rgb(out: *mut c_void, src: *const GhosttyColorRgb) {
    let rgb = out.cast::<GhosttyColorRgb>();
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).r),
            ptr::read(core::ptr::addr_of!((*src).r)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).g),
            ptr::read(core::ptr::addr_of!((*src).g)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).b),
            ptr::read(core::ptr::addr_of!((*src).b)),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_row_get(row: u64, data: c_int, out: *mut c_void) -> c_int {
    unsafe { row_get(row, data, out) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_row_get_multi(
    row: u64,
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < count {
        let key = unsafe { ptr::read(keys.add(i)) };
        let out = unsafe { ptr::read(values.add(i)) };
        let result = unsafe { row_get(row, key, out) };
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

unsafe fn row_get(row: u64, data: c_int, out: *mut c_void) -> c_int {
    match data {
        ROW_DATA_WRAP => unsafe { write_out(out, row_bit(row, 32)) },
        ROW_DATA_WRAP_CONTINUATION => unsafe { write_out(out, row_bit(row, 33)) },
        ROW_DATA_GRAPHEME => unsafe { write_out(out, row_bit(row, 34)) },
        ROW_DATA_STYLED => unsafe { write_out(out, row_bit(row, 35)) },
        ROW_DATA_HYPERLINK => unsafe { write_out(out, row_bit(row, 36)) },
        ROW_DATA_SEMANTIC_PROMPT => unsafe { write_out(out, ((row >> 37) & 0x3) as c_int) },
        ROW_DATA_KITTY_VIRTUAL_PLACEHOLDER => unsafe { write_out(out, row_bit(row, 39)) },
        ROW_DATA_DIRTY => unsafe { write_out(out, row_bit(row, 40)) },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

fn row_bit(row: u64, bit: u32) -> bool {
    ((row >> bit) & 1) != 0
}

unsafe fn write_out<T>(out: *mut c_void, value: T) {
    unsafe {
        ptr::write(out.cast::<T>(), value);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyStyleColor {
    tag: c_int,
    value: GhosttyStyleColorValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union GhosttyStyleColorValue {
    palette: u8,
    rgb: GhosttyColorRgb,
    padding: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyColorRgb {
    r: u8,
    g: u8,
    b: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyStyle {
    size: usize,
    fg_color: GhosttyStyleColor,
    bg_color: GhosttyStyleColor,
    underline_color: GhosttyStyleColor,
    bold: bool,
    italic: bool,
    faint: bool,
    blink: bool,
    inverse: bool,
    invisible: bool,
    strikethrough: bool,
    overline: bool,
    underline: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttySgrUnknown {
    full_ptr: *const u16,
    full_len: usize,
    partial_ptr: *const u16,
    partial_len: usize,
}

#[repr(C)]
pub union GhosttySgrAttributeValue {
    unknown: GhosttySgrUnknown,
    padding: [u64; 8],
}

#[repr(C)]
pub struct GhosttySgrAttribute {
    tag: c_int,
    value: GhosttySgrAttributeValue,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_parser_reset(idx: *mut usize) {
    if idx.is_null() {
        return;
    }

    unsafe {
        ptr::write(idx, 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_params_sep_mask(seps: *const u8, len: usize) -> u32 {
    if seps.is_null() {
        return 0;
    }

    let mut mask = 0u32;
    let mut i = 0usize;
    while i < len && i < 24 {
        let sep = unsafe { ptr::read(seps.add(i)) };
        if sep == b':' {
            mask |= 1u32 << i;
        }
        i += 1;
    }
    mask
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_unknown_full(
    unknown: GhosttySgrUnknown,
    ptr_out: *mut *const u16,
) -> usize {
    if !ptr_out.is_null() {
        unsafe {
            ptr::write(ptr_out, unknown.full_ptr);
        }
    }

    unknown.full_len
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_unknown_partial(
    unknown: GhosttySgrUnknown,
    ptr_out: *mut *const u16,
) -> usize {
    if !ptr_out.is_null() {
        unsafe {
            ptr::write(ptr_out, unknown.partial_ptr);
        }
    }

    unknown.partial_len
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_attribute_tag(attr: GhosttySgrAttribute) -> c_int {
    attr.tag
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_attribute_value(
    attr: *mut GhosttySgrAttribute,
) -> *mut GhosttySgrAttributeValue {
    unsafe { core::ptr::addr_of_mut!((*attr).value) }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_style_default(result: *mut GhosttyStyle) {
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*result).size),
            mem::size_of::<GhosttyStyle>(),
        );
        write_style_color_none(core::ptr::addr_of_mut!((*result).fg_color));
        write_style_color_none(core::ptr::addr_of_mut!((*result).bg_color));
        write_style_color_none(core::ptr::addr_of_mut!((*result).underline_color));
        ptr::write(core::ptr::addr_of_mut!((*result).bold), false);
        ptr::write(core::ptr::addr_of_mut!((*result).italic), false);
        ptr::write(core::ptr::addr_of_mut!((*result).faint), false);
        ptr::write(core::ptr::addr_of_mut!((*result).blink), false);
        ptr::write(core::ptr::addr_of_mut!((*result).inverse), false);
        ptr::write(core::ptr::addr_of_mut!((*result).invisible), false);
        ptr::write(core::ptr::addr_of_mut!((*result).strikethrough), false);
        ptr::write(core::ptr::addr_of_mut!((*result).overline), false);
        ptr::write(core::ptr::addr_of_mut!((*result).underline), 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_style_is_default(style: *const GhosttyStyle) -> bool {
    unsafe {
        ptr::read(core::ptr::addr_of!((*style).size)) == mem::size_of::<GhosttyStyle>()
            && ptr::read(core::ptr::addr_of!((*style).fg_color.tag)) == STYLE_COLOR_NONE
            && ptr::read(core::ptr::addr_of!((*style).bg_color.tag)) == STYLE_COLOR_NONE
            && ptr::read(core::ptr::addr_of!((*style).underline_color.tag)) == STYLE_COLOR_NONE
            && !ptr::read(core::ptr::addr_of!((*style).bold))
            && !ptr::read(core::ptr::addr_of!((*style).italic))
            && !ptr::read(core::ptr::addr_of!((*style).faint))
            && !ptr::read(core::ptr::addr_of!((*style).blink))
            && !ptr::read(core::ptr::addr_of!((*style).inverse))
            && !ptr::read(core::ptr::addr_of!((*style).invisible))
            && !ptr::read(core::ptr::addr_of!((*style).strikethrough))
            && !ptr::read(core::ptr::addr_of!((*style).overline))
            && ptr::read(core::ptr::addr_of!((*style).underline)) == 0
    }
}

unsafe fn write_style_color_none(color: *mut GhosttyStyleColor) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*color).tag), STYLE_COLOR_NONE);
        ptr::write(core::ptr::addr_of_mut!((*color).value.padding), 0);
    }
}
