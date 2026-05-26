use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;

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

pub(crate) unsafe fn kitty_image_get_write(
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
