use core::ffi::c_int;

use crate::early::*;

pub(crate) const SYS_OPT_USERDATA: c_int = 0;
pub(crate) const SYS_OPT_DECODE_PNG: c_int = 1;
pub(crate) const SYS_OPT_LOG: c_int = 2;

#[no_mangle]
pub extern "C" fn ghostty_rust_sys_set(option: c_int) -> c_int {
    sys_set_impl(option)
}

pub(crate) fn sys_set_impl(option: c_int) -> c_int {
    match option {
        SYS_OPT_USERDATA | SYS_OPT_DECODE_PNG | SYS_OPT_LOG => GHOSTTY_SUCCESS,
        _ => GHOSTTY_INVALID_VALUE,
    }
}
