//! Lang-item stubs for `panic=abort` no_std links when the VT stream is enabled.
//!
//! See `docs/PORTING.md` (Build Contract). These abort without formatting.

use core::convert::Infallible;

#[export_name = "_RNvNtCsl93z6Lp4RQ9_4core9panicking18panic_bounds_check"]
pub extern "C" fn ghostty_panic_bounds_check() -> Infallible {
    loop {}
}

#[export_name = "_RNvNtNtCsl93z6Lp4RQ9_4core5slice5index16slice_index_fail"]
pub extern "C" fn ghostty_slice_index_fail() -> Infallible {
    loop {}
}

#[export_name = "_RNvNtNtCsl93z6Lp4RQ9_4core3str8converts9from_utf8"]
pub extern "C" fn ghostty_str_from_utf8() -> Infallible {
    loop {}
}
