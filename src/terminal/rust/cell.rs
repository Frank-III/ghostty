use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::style::*;

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

pub(crate) unsafe fn cell_get(cell: u64, data: c_int, out: *mut c_void) -> c_int {
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

pub(crate) fn cell_content_tag(cell: u64) -> u64 {
    cell & 0x3
}

pub(crate) fn cell_content(cell: u64) -> u64 {
    (cell >> 2) & 0x00ff_ffff
}

pub(crate) fn cell_codepoint(cell: u64) -> u32 {
    match cell_content_tag(cell) {
        CELL_CONTENT_TAG_CODEPOINT | CELL_CONTENT_TAG_CODEPOINT_GRAPHEME => {
            (cell_content(cell) & 0x001f_ffff) as u32
        }
        _ => 0,
    }
}

pub(crate) fn cell_has_text(cell: u64) -> bool {
    match cell_content_tag(cell) {
        CELL_CONTENT_TAG_CODEPOINT | CELL_CONTENT_TAG_CODEPOINT_GRAPHEME => {
            cell_codepoint(cell) != 0
        }
        _ => false,
    }
}

pub(crate) fn cell_has_grapheme(cell: u64) -> bool {
    cell_content_tag(cell) == CELL_CONTENT_TAG_CODEPOINT_GRAPHEME
}

pub(crate) fn cell_style_id(cell: u64) -> u16 {
    ((cell >> 26) & 0xffff) as u16
}

pub(crate) fn cell_bit(cell: u64, bit: u32) -> bool {
    ((cell >> bit) & 1) != 0
}

pub(crate) unsafe fn write_cell_rgb(out: *mut c_void, content: u64) {
    let rgb = out.cast::<GhosttyColorRgb>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*rgb).r), content as u8);
        ptr::write(core::ptr::addr_of_mut!((*rgb).g), (content >> 8) as u8);
        ptr::write(core::ptr::addr_of_mut!((*rgb).b), (content >> 16) as u8);
    }
}
