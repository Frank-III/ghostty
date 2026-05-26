use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::simple::*;
use crate::style::*;

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
