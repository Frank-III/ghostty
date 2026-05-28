//! C ABI exports for the Rust `libghostty` bootstrap (`include/ghostty.h` subset).
//!
//! Shipping symbols remain Zig-owned; these entry points are for the Rust port and
//! generate `ghostty_rust.h` via cbindgen in `build.rs`.

use core::ffi::c_void;
use core::ptr;

use ghostty_core::{App, AppConfig, RuntimeConfig};

/// Opaque app pointer (`ghostty_app_t`).
pub type GhosttyApp = c_void;

/// `ghostty_runtime_config_s` — callback fields deferred; layout matches header subset.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GhosttyRuntimeConfig {
    pub userdata: *mut c_void,
    pub supports_selection_clipboard: bool,
    // Padding so cbindgen emits a stable struct before callbacks are added.
    _reserved: [u8; 7],
}

impl Default for GhosttyRuntimeConfig {
    fn default() -> Self {
        Self {
            userdata: ptr::null_mut(),
            supports_selection_clipboard: false,
            _reserved: [0; 7],
        }
    }
}

fn runtime_from_c(cfg: *const GhosttyRuntimeConfig) -> RuntimeConfig {
    if cfg.is_null() {
        return RuntimeConfig::default();
    }
    // SAFETY: caller provides a valid `GhosttyRuntimeConfig` for the duration of the call.
    let cfg = unsafe { &*cfg };
    RuntimeConfig {
        userdata: cfg.userdata,
        supports_selection_clipboard: cfg.supports_selection_clipboard,
    }
}

fn app_from_ptr(ptr: *mut GhosttyApp) -> Option<&'static mut App> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: pointer came from `ghostty_app_new`.
    Some(unsafe { &mut *(ptr as *mut App) })
}

/// Create an application instance (`ghostty_app_new` bootstrap).
///
/// `config` is ignored for now (Zig `ghostty_config_t`); defaults are used.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_new(
    runtime: *const GhosttyRuntimeConfig,
    _config: *mut c_void,
) -> *mut GhosttyApp {
    let app = App::new(AppConfig::default(), runtime_from_c(runtime));
    Box::into_raw(Box::new(app)) as *mut GhosttyApp
}

/// Release an application instance (`ghostty_app_free` bootstrap).
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_free(app: *mut GhosttyApp) {
    if app.is_null() {
        return;
    }
    drop(Box::from_raw(app as *mut App));
}

/// Tick the app loop (`ghostty_app_tick` bootstrap) — drains Rust pending events only.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_tick(app: *mut GhosttyApp) {
    if let Some(app) = app_from_ptr(app) {
        let _ = app.tick();
    }
}

/// Return embedder userdata from runtime config (`ghostty_app_userdata` bootstrap).
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_userdata(app: *mut GhosttyApp) -> *mut c_void {
    app_from_ptr(app)
        .map(|a| a.runtime().userdata)
        .unwrap_or(ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_new_free_round_trip() {
        let cfg = GhosttyRuntimeConfig::default();
        let app = unsafe { ghostty_app_new(&cfg, ptr::null_mut()) };
        assert!(!app.is_null());
        unsafe { ghostty_app_tick(app) };
        assert!(unsafe { ghostty_app_userdata(app) }.is_null());
        unsafe { ghostty_app_free(app) };
    }

    #[test]
    fn null_app_tick_is_noop() {
        unsafe { ghostty_app_tick(ptr::null_mut()) };
    }
}
