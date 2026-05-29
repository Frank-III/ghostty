use core::ffi::c_int;
use core::ptr;

use crate::sgr::*;
use crate::sgr_attr::*;
use crate::sgr_constants::*;
use crate::style::*;

pub(crate) unsafe fn clear_sgr_value(result: *mut GhosttySgrAttribute) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*result).value.padding), [0u64; 8]);
    }
}

pub(crate) unsafe fn write_sgr_empty(result: *mut GhosttySgrAttribute, tag: c_int) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*result).tag), tag);
        clear_sgr_value(result);
    }
}

pub(crate) unsafe fn write_sgr_c_int(result: *mut GhosttySgrAttribute, tag: c_int, value: c_int) {
    unsafe {
        write_sgr_empty(result, tag);
        ptr::write(
            core::ptr::addr_of_mut!((*result).value).cast::<c_int>(),
            value,
        );
    }
}

pub(crate) unsafe fn write_sgr_u8(result: *mut GhosttySgrAttribute, tag: c_int, value: u8) {
    unsafe {
        write_sgr_empty(result, tag);
        ptr::write(core::ptr::addr_of_mut!((*result).value).cast::<u8>(), value);
    }
}

pub(crate) unsafe fn write_sgr_rgb(
    result: *mut GhosttySgrAttribute,
    tag: c_int,
    r: u8,
    g: u8,
    b: u8,
) {
    unsafe {
        write_sgr_empty(result, tag);
        let rgb = core::ptr::addr_of_mut!((*result).value).cast::<GhosttyColorRgb>();
        ptr::write(core::ptr::addr_of_mut!((*rgb).r), r);
        ptr::write(core::ptr::addr_of_mut!((*rgb).g), g);
        ptr::write(core::ptr::addr_of_mut!((*rgb).b), b);
    }
}

pub(crate) unsafe fn write_sgr_unknown(
    result: *mut GhosttySgrAttribute,
    full_ptr: *const u16,
    full_len: usize,
    partial_ptr: *const u16,
    partial_len: usize,
) {
    unsafe {
        write_sgr_empty(result, SGR_UNKNOWN);
        ptr::write(
            core::ptr::addr_of_mut!((*result).value.unknown),
            GhosttySgrUnknown {
                full_ptr,
                full_len,
                partial_ptr,
                partial_len,
            },
        );
    }
}
