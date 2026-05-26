use core::ffi::c_int;
use core::ptr;

use crate::constants::*;
use crate::early::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_layer_matches(layer: c_int, z: i32) -> bool {
    kitty_placement_layer_matches_impl(layer, z)
}

pub(crate) fn kitty_placement_layer_matches_impl(layer: c_int, z: i32) -> bool {
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
    unsafe { kitty_placement_iterator_set_impl(option, layer, out_layer) }
}

pub(crate) unsafe fn kitty_placement_iterator_set_impl(
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
