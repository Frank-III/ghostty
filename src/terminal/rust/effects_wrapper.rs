//! C `TerminalWrapper` effect vtable for rust-owned terminals without Zig `terminal.c`.

use core::ffi::c_void;

pub type VtEffectFn = Option<unsafe extern "C" fn(*mut c_void)>;
pub type VtDeviceAttributesFn = Option<unsafe extern "C" fn(*mut c_void, u8)>;
pub type VtClipboardFn = Option<unsafe extern "C" fn(*mut c_void, u8, *const u8, usize)>;
pub type VtQuerySizeFn = Option<
    unsafe extern "C" fn(*mut c_void, *mut crate::constants::GhosttySizeReportSize) -> bool,
>;
pub type VtWritePtyFn = Option<unsafe extern "C" fn(*mut c_void, *const u8, usize)>;

/// Effect callbacks registered on a rust-owned terminal via [`set_wrapper`](crate::terminal_owned::RustTerminalOwned::set_wrapper).
#[repr(C)]
pub struct GhosttyVtEffectWrapper {
    pub userdata: *mut c_void,
    pub write_pty: VtWritePtyFn,
    pub bell: VtEffectFn,
    pub title_changed: VtEffectFn,
    pub report_enquiry: VtEffectFn,
    pub report_xtversion: VtEffectFn,
    pub report_device_attributes: VtDeviceAttributesFn,
    pub query_size: VtQuerySizeFn,
    pub report_color_scheme: VtEffectFn,
    pub clipboard_contents: VtClipboardFn,
}

impl GhosttyVtEffectWrapper {
    pub const fn empty() -> Self {
        Self {
            userdata: core::ptr::null_mut(),
            write_pty: None,
            bell: None,
            title_changed: None,
            report_enquiry: None,
            report_xtversion: None,
            report_device_attributes: None,
            query_size: None,
            report_color_scheme: None,
            clipboard_contents: None,
        }
    }

    pub unsafe fn dispatch_write_pty(wrapper: *mut c_void, data: &[u8]) {
        unsafe {
            if wrapper.is_null() || data.is_empty() {
                return;
            }
            let w = &*(wrapper as *const Self);
            if let Some(f) = w.write_pty {
                f(wrapper, data.as_ptr(), data.len());
            }
        }
    }

    pub unsafe fn dispatch_bell(wrapper: *mut c_void) {
        unsafe {
            dispatch_effect(wrapper, |w| w.bell);
        }
    }

    pub unsafe fn dispatch_title_changed(wrapper: *mut c_void) {
        unsafe {
            dispatch_effect(wrapper, |w| w.title_changed);
        }
    }

    pub unsafe fn dispatch_report_enquiry(wrapper: *mut c_void) {
        unsafe {
            dispatch_effect(wrapper, |w| w.report_enquiry);
        }
    }

    pub unsafe fn dispatch_report_xtversion(wrapper: *mut c_void) {
        unsafe {
            dispatch_effect(wrapper, |w| w.report_xtversion);
        }
    }

    pub unsafe fn dispatch_report_device_attributes(wrapper: *mut c_void, req: u8) {
        unsafe {
            if wrapper.is_null() {
                return;
            }
            let w = &*(wrapper as *const Self);
            if let Some(f) = w.report_device_attributes {
                f(wrapper, req);
            }
        }
    }

    pub unsafe fn dispatch_query_size(
        wrapper: *mut c_void,
        out: &mut crate::constants::GhosttySizeReportSize,
    ) -> bool {
        unsafe {
            if wrapper.is_null() {
                return false;
            }
            let w = &*(wrapper as *const Self);
            if let Some(f) = w.query_size {
                f(wrapper, out)
            } else {
                false
            }
        }
    }

    pub unsafe fn dispatch_report_color_scheme(wrapper: *mut c_void) {
        unsafe {
            dispatch_effect(wrapper, |w| w.report_color_scheme);
        }
    }

    pub unsafe fn dispatch_clipboard_contents(
        wrapper: *mut c_void,
        kind: u8,
        data: &[u8],
    ) {
        unsafe {
            if wrapper.is_null() {
                return;
            }
            let w = &*(wrapper as *const Self);
            if let Some(f) = w.clipboard_contents {
                f(wrapper, kind, data.as_ptr(), data.len());
            }
        }
    }
}

unsafe fn dispatch_effect(
    wrapper: *mut c_void,
    pick: impl FnOnce(&GhosttyVtEffectWrapper) -> VtEffectFn,
) {
    unsafe {
        if wrapper.is_null() {
            return;
        }
        let w = &*(wrapper as *const GhosttyVtEffectWrapper);
        if let Some(f) = pick(w) {
            f(wrapper);
        }
    }
}
