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

/// Returns the Rust port milestone string for embedders (bootstrap ABI).
#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_port_milestone() -> *const u8 {
    b"phase6-surface-session\0".as_ptr()
}

#[cfg(all(unix, feature = "rust-vt"))]
mod surface_session_ffi {
    use core::ffi::{c_char, c_void};
    use core::ptr;
    use std::time::{Duration, Instant};

    use ghostty_core::SurfaceSession;

    /// Opaque headless surface session (`ghostty_surface_session_t` bootstrap).
    pub type GhosttySurfaceSession = c_void;

    fn session_from_ptr(ptr: *mut GhosttySurfaceSession) -> Option<&'static mut SurfaceSession> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &mut *(ptr as *mut SurfaceSession) })
    }

    /// Spawn a default headless session (config defaults + `/bin/sh`).
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_spawn_default(
    ) -> *mut GhosttySurfaceSession {
        match SurfaceSession::from_defaults() {
            Ok(session) => Box::into_raw(Box::new(session)) as *mut GhosttySurfaceSession,
            Err(_) => ptr::null_mut(),
        }
    }

    /// Release a session created by `ghostty_surface_session_spawn_default`.
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_free(session: *mut GhosttySurfaceSession) {
        if session.is_null() {
            return;
        }
        drop(Box::from_raw(session as *mut SurfaceSession));
    }

    /// Write bytes to the PTY (returns 0 on success, -1 on error).
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_write(
        session: *mut GhosttySurfaceSession,
        data: *const u8,
        len: usize,
    ) -> i32 {
        let Some(session) = session_from_ptr(session) else {
            return -1;
        };
        if data.is_null() || len == 0 {
            return 0;
        }
        let bytes = core::slice::from_raw_parts(data, len);
        match session.write(bytes) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// Pump PTY output into terminal state (returns bytes read, or -1 on error).
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_tick(
        session: *mut GhosttySurfaceSession,
    ) -> isize {
        let Some(session) = session_from_ptr(session) else {
            return -1;
        };
        match session.tick() {
            Ok(n) => n as isize,
            Err(_) => -1,
        }
    }

    /// Returns 1 if active row 0 starts with `needle` (ASCII test helper).
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_contains_text(
        session: *mut GhosttySurfaceSession,
        needle: *const c_char,
    ) -> i32 {
        let Some(session) = session_from_ptr(session) else {
            return 0;
        };
        if needle.is_null() {
            return 0;
        }
        let cstr = unsafe { core::ffi::CStr::from_ptr(needle) };
        let Ok(s) = cstr.to_str() else {
            return 0;
        };
        i32::from(session.contains_text(s))
    }

    /// Run the session loop until `needle` appears or timeout_ms elapses.
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_run_until_text(
        session: *mut GhosttySurfaceSession,
        needle: *const c_char,
        timeout_ms: u32,
    ) -> i32 {
        let Some(session) = session_from_ptr(session) else {
            return 0;
        };
        if needle.is_null() {
            return 0;
        }
        let cstr = unsafe { core::ffi::CStr::from_ptr(needle) };
        let Ok(needle_str) = cstr.to_str() else {
            return 0;
        };
        let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
        let found = session
            .run_until(deadline, |s| s.contains_text(needle_str))
            .is_ok()
            && session.contains_text(needle_str);
        i32::from(found)
    }
}

#[cfg(all(unix, feature = "rust-vt"))]
pub use surface_session_ffi::*;

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

    #[cfg(all(unix, feature = "rust-vt"))]
    mod surface_session {
        use super::*;
        use ghostty_termio::{CommandBuilder, CommandSpec};
        use ghostty_core::{AppConfig, SurfaceSession, SurfaceSessionOptions};

        #[test]
        fn ffi_spawn_tick_contains_text() {
            let spec: CommandSpec = CommandBuilder::new()
                .path("/bin/sh")
                .arg("sh")
                .arg("-c")
                .arg("printf 'ffi-vt'")
                .build()
                .expect("spec");

            let mut session = SurfaceSession::spawn(
                AppConfig::default(),
                SurfaceSessionOptions {
                    command: Some(spec),
                    ..Default::default()
                },
            )
            .expect("spawn");

            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            session
                .run_until(deadline, |s| s.contains_text("ffi-vt"))
                .expect("run");
            assert!(session.contains_text("ffi-vt"));
        }

        #[test]
        fn ffi_c_api_round_trip() {
            let spec = CommandBuilder::new()
                .path("/bin/sh")
                .arg("sh")
                .arg("-c")
                .arg("cat")
                .build()
                .expect("spec");

            let session = SurfaceSession::spawn(
                AppConfig::default(),
                SurfaceSessionOptions {
                    command: Some(spec),
                    ..Default::default()
                },
            )
            .expect("spawn");

            let raw = Box::into_raw(Box::new(session)) as *mut GhosttySurfaceSession;
            assert_eq!(
                unsafe { ghostty_surface_session_write(raw, b"ffi\n".as_ptr(), 4) },
                0
            );
            let found = unsafe {
                ghostty_surface_session_run_until_text(raw, c"ffi".as_ptr(), 3000)
            };
            assert_eq!(found, 1);
            unsafe { ghostty_surface_session_free(raw) };
        }

        #[test]
        fn ffi_spawn_default_not_null() {
            let session = unsafe { ghostty_surface_session_spawn_default() };
            assert!(!session.is_null());
            unsafe { ghostty_surface_session_free(session) };
        }
    }
}
