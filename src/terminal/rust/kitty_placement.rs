use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::kitty_placement_get::*;

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
    unsafe {
        kitty_placement_get_multi_impl(
            count,
            keys,
            values,
            out_written,
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
        )
    }
}

pub(crate) unsafe fn kitty_placement_get_multi_impl(
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
