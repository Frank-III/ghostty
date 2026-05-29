//! Embedded runtime options (`apprt/embedded.zig` `App.Options` / `ghostty_runtime_config_s`).
//!
//! C callback function pointers are not wired yet; embedders will supply them when
//! the FFI layer grows beyond the bootstrap `ghostty_app_*` stubs.

use core::ffi::c_void;
use std::path::PathBuf;

/// Host-provided runtime hooks for the embedded apprt (skeleton).
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    pub userdata: *mut c_void,
    pub supports_selection_clipboard: bool,
    /// Ghostty resources directory for shell integration (`global_state.resources_dir`).
    pub resources_dir: Option<PathBuf>,
}

impl RuntimeConfig {
    pub const fn new() -> Self {
        Self {
            userdata: core::ptr::null_mut(),
            supports_selection_clipboard: false,
            resources_dir: None,
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
    }
}
