use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;

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

pub(crate) unsafe fn kitty_placement_get_write(
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
