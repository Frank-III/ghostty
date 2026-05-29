//! Embedded runtime options (`apprt/embedded.zig` `App.Options` / `ghostty_runtime_config_s`).

use core::ffi::{c_char, c_void};
use std::path::PathBuf;

/// `ghostty_clipboard_e`
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeClipboard {
    Standard = 0,
    Selection = 1,
}

/// `ghostty_clipboard_content_s`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RuntimeClipboardContent {
    pub mime: *const c_char,
    pub data: *const c_char,
}

/// `ghostty_clipboard_request_e`
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeClipboardRequest {
    Paste = 0,
    Osc52Read = 1,
    Osc52Write = 2,
}

/// `ghostty_target_tag_e`
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTargetTag {
    App = 0,
    Surface = 1,
}

/// `ghostty_target_u`
#[repr(C)]
#[derive(Clone, Copy)]
pub union RuntimeTargetU {
    pub surface: *mut c_void,
}

/// `ghostty_target_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeTarget {
    pub tag: RuntimeTargetTag,
    pub target: RuntimeTargetU,
}

/// Opaque action payload (`ghostty_action_s`); full union deferred.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeAction {
    pub tag: u32,
    _payload: [u8; 256],
}

pub type RuntimeWakeupCb = Option<unsafe extern "C" fn(*mut c_void)>;
pub type RuntimeReadClipboardCb =
    Option<unsafe extern "C" fn(*mut c_void, RuntimeClipboard, *mut c_void) -> bool>;
pub type RuntimeConfirmReadClipboardCb = Option<
    unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void, RuntimeClipboardRequest),
>;
pub type RuntimeWriteClipboardCb = Option<
    unsafe extern "C" fn(
        *mut c_void,
        RuntimeClipboard,
        *const RuntimeClipboardContent,
        usize,
        bool,
    ),
>;
pub type RuntimeCloseSurfaceCb = Option<unsafe extern "C" fn(*mut c_void, bool)>;
pub type RuntimeActionCb =
    Option<unsafe extern "C" fn(*mut c_void, RuntimeTarget, RuntimeAction) -> bool>;

/// Host-provided runtime hooks for the embedded apprt.
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    pub userdata: *mut c_void,
    pub supports_selection_clipboard: bool,
    /// Ghostty resources directory for shell integration (`global_state.resources_dir`).
    pub resources_dir: Option<PathBuf>,
    pub wakeup_cb: RuntimeWakeupCb,
    pub action_cb: RuntimeActionCb,
    pub read_clipboard_cb: RuntimeReadClipboardCb,
    pub confirm_read_clipboard_cb: RuntimeConfirmReadClipboardCb,
    pub write_clipboard_cb: RuntimeWriteClipboardCb,
    pub close_surface_cb: RuntimeCloseSurfaceCb,
}

impl RuntimeConfig {
    pub const fn new() -> Self {
        Self {
            userdata: core::ptr::null_mut(),
            supports_selection_clipboard: false,
            resources_dir: None,
            wakeup_cb: None,
            action_cb: None,
            read_clipboard_cb: None,
            confirm_read_clipboard_cb: None,
            write_clipboard_cb: None,
            close_surface_cb: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let rt = RuntimeConfig::default();
        assert!(rt.userdata.is_null());
        assert!(!rt.supports_selection_clipboard);
        assert!(rt.wakeup_cb.is_none());
    }
}
