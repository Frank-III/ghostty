use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::event_cell_style::*;

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

pub(crate) fn width_pixels(size: GhosttySizeReportSize) -> u64 {
    u64::from(size.columns) * u64::from(size.cell_width)
}

pub(crate) fn height_pixels(size: GhosttySizeReportSize) -> u64 {
    u64::from(size.rows) * u64::from(size.cell_height)
}

pub(crate) fn size_report_len(style: c_int, size: GhosttySizeReportSize) -> Option<usize> {
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

pub(crate) fn decimal_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 10 {
        value /= 10;
        len += 1;
    }
    len
}

pub(crate) fn signed_decimal_len(value: i32) -> usize {
    if value < 0 {
        1 + decimal_len((-i64::from(value)) as u64)
    } else {
        decimal_len(value as u64)
    }
}

pub(crate) fn utf8_len(codepoint: u32) -> Option<usize> {
    match codepoint {
        0x0000..=0x007f => Some(1),
        0x0080..=0x07ff => Some(2),
        0x0800..=0xffff => Some(3),
        0x1_0000..=0x10_ffff => Some(4),
        _ => None,
    }
}

pub(crate) unsafe fn write_size_report(style: c_int, size: GhosttySizeReportSize, out: *mut u8) {
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

pub(crate) unsafe fn write_bytes(out: *mut u8, offset: &mut usize, bytes: &[u8]) {
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

pub(crate) unsafe fn write_byte(out: *mut u8, offset: &mut usize, byte: u8) {
    unsafe {
        ptr::write(out.add(*offset), byte);
    }
    *offset += 1;
}

pub(crate) unsafe fn write_decimal(out: *mut u8, offset: &mut usize, mut value: u64) {
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

pub(crate) unsafe fn write_signed_decimal(out: *mut u8, offset: &mut usize, value: i32) {
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

pub(crate) unsafe fn write_utf8(out: *mut u8, offset: &mut usize, codepoint: u32) {
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

pub(crate) unsafe fn write_mouse_action_suffix(out: *mut u8, offset: &mut usize, action: c_int) {
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

pub(crate) fn paste_strip_byte(byte: u8) -> bool {
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

pub(crate) unsafe fn matches_bytes_at(data: *const u8, len: usize, offset: usize, bytes: &[u8]) -> bool {
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

pub(crate) unsafe fn copy_data_bytes(out: *mut u8, offset: &mut usize, data: *const u8, len: usize) {
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

pub(crate) fn mode_report_len(value: u64, ansi: bool, state: u64) -> usize {
    b"\x1B[".len()
        + if ansi { 0 } else { 1 }
        + decimal_len(value)
        + 1
        + decimal_len(state)
        + b"$y".len()
}

pub(crate) unsafe fn write_mode_report(out: *mut u8, value: u64, ansi: bool, state: u64) {
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
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,
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

pub(crate) fn env_flag(value: &str) -> bool {
    value.as_bytes() == b"1"
}

pub(crate) fn optimize_value(value: &str) -> c_int {
    match value.as_bytes() {
        b"debug" => 0,
        b"release_safe" => 1,
        b"release_small" => 2,
        b"release_fast" => 3,
        _ => 0,
    }
}

pub(crate) fn env_usize(bytes: &[u8]) -> usize {
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

pub(crate) fn struct_sized_field_fits<T>(size: usize, offset: usize) -> bool {
    size >= offset.saturating_add(mem::size_of::<T>())
}

pub(crate) unsafe fn write_string(out: *mut c_void, bytes: &'static [u8]) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), bytes.as_ptr());
        ptr::write(core::ptr::addr_of_mut!((*string).len), bytes.len());
    }
}

pub(crate) unsafe fn write_borrowed_string(out: *mut c_void, ptr: *const u8, len: usize) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), ptr);
        ptr::write(core::ptr::addr_of_mut!((*string).len), len);
    }
}
