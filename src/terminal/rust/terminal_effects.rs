//! Invoke C terminal wrapper effect callbacks from the rust-owned VT path.

use core::ffi::c_void;

#[cfg(ghostty_vt_terminal_owned)]
extern "C" {
    fn ghostty_terminal_wrapper_write_pty(wrapper: *mut c_void, ptr: *const u8, len: usize);
    fn ghostty_terminal_wrapper_bell(wrapper: *mut c_void);
    fn ghostty_terminal_wrapper_title_changed(wrapper: *mut c_void);
    fn ghostty_terminal_wrapper_report_enquiry(wrapper: *mut c_void);
    fn ghostty_terminal_wrapper_report_xtversion(wrapper: *mut c_void);
    fn ghostty_terminal_wrapper_report_device_attributes(wrapper: *mut c_void, req: u8);
    fn ghostty_terminal_wrapper_query_size(
        wrapper: *mut c_void,
        out: *mut crate::constants::GhosttySizeReportSize,
    ) -> bool;
    fn ghostty_terminal_wrapper_report_color_scheme(wrapper: *mut c_void);
    fn ghostty_terminal_wrapper_clipboard_contents(
        wrapper: *mut c_void,
        kind: u8,
        ptr: *const u8,
        len: usize,
    );
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn write_pty(wrapper: *mut c_void, data: &[u8]) {
    if wrapper.is_null() || data.is_empty() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_write_pty(wrapper, data.as_ptr(), data.len());
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn bell(wrapper: *mut c_void) {
    if wrapper.is_null() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_bell(wrapper);
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn title_changed(wrapper: *mut c_void) {
    if wrapper.is_null() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_title_changed(wrapper);
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn report_enquiry(wrapper: *mut c_void) {
    if wrapper.is_null() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_report_enquiry(wrapper);
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn report_xtversion(wrapper: *mut c_void) {
    if wrapper.is_null() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_report_xtversion(wrapper);
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn report_device_attributes(wrapper: *mut c_void, req: u8) {
    if wrapper.is_null() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_report_device_attributes(wrapper, req);
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn query_size(
    wrapper: *mut c_void,
    out: &mut crate::constants::GhosttySizeReportSize,
) -> bool {
    if wrapper.is_null() {
        return false;
    }
    unsafe { ghostty_terminal_wrapper_query_size(wrapper, out) }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn report_color_scheme(wrapper: *mut c_void) {
    if wrapper.is_null() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_report_color_scheme(wrapper);
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) unsafe fn clipboard_contents(wrapper: *mut c_void, kind: u8, data: &[u8]) {
    if wrapper.is_null() || data.is_empty() {
        return;
    }
    unsafe {
        ghostty_terminal_wrapper_clipboard_contents(wrapper, kind, data.as_ptr(), data.len());
    }
}
