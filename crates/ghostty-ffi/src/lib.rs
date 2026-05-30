//! C ABI exports for the Rust `libghostty` bootstrap (`include/ghostty.h` subset).
//!
//! Shipping symbols remain Zig-owned; these entry points are for the Rust port and
//! generate `ghostty_rust.h` via cbindgen in `build.rs`.

use core::ffi::{c_char, c_void, CStr};
use core::ptr;

use ghostty_core::{
    App, AppConfig, AppEvent, RuntimeAction, RuntimeActionCb, RuntimeClipboard,
    RuntimeClipboardContent, RuntimeClipboardRequest, RuntimeCloseSurfaceCb, RuntimeConfig,
    RuntimeConfirmReadClipboardCb, RuntimeReadClipboardCb, RuntimeTarget, RuntimeTargetTag,
    RuntimeTargetU, RuntimeWakeupCb, RuntimeWriteClipboardCb, SurfaceEvent,
};

/// Opaque app pointer (`ghostty_app_t`).
pub type GhosttyApp = c_void;

pub type GhosttyClipboard = RuntimeClipboard;
pub type GhosttyClipboardContent = RuntimeClipboardContent;
pub type GhosttyClipboardRequest = RuntimeClipboardRequest;
pub type GhosttyTargetTag = RuntimeTargetTag;
pub type GhosttyTargetU = RuntimeTargetU;
pub type GhosttyTarget = RuntimeTarget;
pub type GhosttyAction = RuntimeAction;
pub type GhosttyRuntimeWakeupCb = RuntimeWakeupCb;
pub type GhosttyRuntimeReadClipboardCb = RuntimeReadClipboardCb;
pub type GhosttyRuntimeConfirmReadClipboardCb = RuntimeConfirmReadClipboardCb;
pub type GhosttyRuntimeWriteClipboardCb = RuntimeWriteClipboardCb;
pub type GhosttyRuntimeCloseSurfaceCb = RuntimeCloseSurfaceCb;
pub type GhosttyRuntimeActionCb = RuntimeActionCb;

/// `ghostty_runtime_config_s` — layout matches `include/ghostty.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GhosttyRuntimeConfig {
    pub userdata: *mut c_void,
    pub supports_selection_clipboard: bool,
    pub wakeup_cb: GhosttyRuntimeWakeupCb,
    pub action_cb: GhosttyRuntimeActionCb,
    pub read_clipboard_cb: GhosttyRuntimeReadClipboardCb,
    pub confirm_read_clipboard_cb: GhosttyRuntimeConfirmReadClipboardCb,
    pub write_clipboard_cb: GhosttyRuntimeWriteClipboardCb,
    pub close_surface_cb: GhosttyRuntimeCloseSurfaceCb,
}

impl Default for GhosttyRuntimeConfig {
    fn default() -> Self {
        Self {
            userdata: ptr::null_mut(),
            supports_selection_clipboard: false,
            wakeup_cb: None,
            action_cb: None,
            read_clipboard_cb: None,
            confirm_read_clipboard_cb: None,
            write_clipboard_cb: None,
            close_surface_cb: None,
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
        resources_dir: std::env::var_os("GHOSTTY_RESOURCES_DIR").map(std::path::PathBuf::from),
        wakeup_cb: cfg.wakeup_cb,
        action_cb: cfg.action_cb,
        read_clipboard_cb: cfg.read_clipboard_cb,
        confirm_read_clipboard_cb: cfg.confirm_read_clipboard_cb,
        write_clipboard_cb: cfg.write_clipboard_cb,
        close_surface_cb: cfg.close_surface_cb,
    }
}

fn app_from_ptr(ptr: *mut GhosttyApp) -> Option<&'static mut App> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: pointer came from `ghostty_app_new`.
    Some(unsafe { &mut *(ptr as *mut App) })
}

fn app_config_from_c(config_path: *const c_char) -> AppConfig {
    if config_path.is_null() {
        return AppConfig::default();
    }
    let Ok(path) = unsafe { CStr::from_ptr(config_path) }.to_str() else {
        return AppConfig::default();
    };
    AppConfig::from_config_file(std::path::Path::new(path)).unwrap_or_default()
}

/// Create an application instance (`ghostty_app_new` bootstrap).
///
/// When `config_path` is non-null, it must be a UTF-8 path to a config file
/// (tilde expansion supported). Null uses built-in defaults.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_new(
    runtime: *const GhosttyRuntimeConfig,
    config_path: *const c_char,
) -> *mut GhosttyApp {
    let app = App::new(app_config_from_c(config_path), runtime_from_c(runtime));
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

/// Set the Ghostty resources directory before creating surfaces.
///
/// `path` must be UTF-8. Reloads theme overlay when a theme is configured.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_set_resources_dir(
    app: *mut GhosttyApp,
    path: *const c_char,
) -> bool {
    let Some(app) = app_from_ptr(app) else {
        return false;
    };
    if path.is_null() {
        return false;
    }
    let Ok(path) = CStr::from_ptr(path).to_str() else {
        return false;
    };
    app.set_resources_dir(std::path::PathBuf::from(path));
    true
}

/// Prepare a draw frame for a surface; returns populated cell count or `usize::MAX` on error.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_surface_prepare_draw(
    app: *mut GhosttyApp,
    id: GhosttySurfaceId,
) -> usize {
    let Some(app) = app_from_ptr(app) else {
        return usize::MAX;
    };
    let Some(sid) = ghostty_core::SurfaceId::from_raw(id.raw) else {
        return usize::MAX;
    };
    let Some(surface) = app.find_surface_mut(sid) else {
        return usize::MAX;
    };
    surface.prepare_draw().unwrap_or(usize::MAX)
}

/// Mark the last prepared draw frame as presented.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_surface_finish_draw(
    app: *mut GhosttyApp,
    id: GhosttySurfaceId,
) {
    let Some(app) = app_from_ptr(app) else {
        return;
    };
    let Some(sid) = ghostty_core::SurfaceId::from_raw(id.raw) else {
        return;
    };
    if let Some(surface) = app.find_surface_mut(sid) {
        surface.finish_draw();
    }
}

/// Returns the Rust port milestone string for embedders (bootstrap ABI).
#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_port_milestone() -> *const u8 {
    b"phase7-cargo-primary\0".as_ptr()
}

/// Surface id for C embedders (`ghostty_surface_id_t`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhosttySurfaceId {
    pub raw: u64,
}

/// Event tag for polled app events (`ghostty_app_event_tag_e` subset).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhosttyAppEventTag {
    None = 0,
    Quit = 1,
    CloseSurface = 2,
    SurfaceChildExited = 3,
    SurfaceFocusChanged = 4,
    SurfaceTitleChanged = 5,
    SurfaceRedrawRequested = 6,
}

/// Polled app event payload (`ghostty_app_event_s` subset).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GhosttyAppEvent {
    pub tag: GhosttyAppEventTag,
    pub surface_id: GhosttySurfaceId,
    pub exit_code: u32,
    pub focused: bool,
}

impl Default for GhosttyAppEvent {
    fn default() -> Self {
        Self {
            tag: GhosttyAppEventTag::None,
            surface_id: GhosttySurfaceId { raw: 0 },
            exit_code: 0,
            focused: false,
        }
    }
}

/// Create a surface on the app (`ghostty_app_create_surface` bootstrap).
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_create_surface(app: *mut GhosttyApp) -> GhosttySurfaceId {
    let Some(app) = app_from_ptr(app) else {
        return GhosttySurfaceId { raw: 0 };
    };
    app.create_surface()
        .map(|id| GhosttySurfaceId { raw: id.get() })
        .unwrap_or(GhosttySurfaceId { raw: 0 })
}

/// Delete a surface by id (`ghostty_app_delete_surface` bootstrap).
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_delete_surface(
    app: *mut GhosttyApp,
    id: GhosttySurfaceId,
) -> bool {
    let Some(app) = app_from_ptr(app) else {
        return false;
    };
    ghostty_core::SurfaceId::from_raw(id.raw)
        .map(|sid| app.delete_surface(sid))
        .unwrap_or(false)
}

/// Focus a surface (`ghostty_app_focus_surface` bootstrap).
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_focus_surface(
    app: *mut GhosttyApp,
    id: GhosttySurfaceId,
) -> bool {
    let Some(app) = app_from_ptr(app) else {
        return false;
    };
    ghostty_core::SurfaceId::from_raw(id.raw)
        .map(|sid| app.focus_surface(sid))
        .unwrap_or(false)
}

/// Number of live surfaces.
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_surface_count(app: *mut GhosttyApp) -> usize {
    app_from_ptr(app).map(|a| a.surface_count()).unwrap_or(0)
}

/// Write bytes to a surface PTY (returns 0 on success, -1 on error).
#[cfg(all(unix, feature = "rust-vt"))]
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_surface_write(
    app: *mut GhosttyApp,
    id: GhosttySurfaceId,
    data: *const u8,
    len: usize,
) -> i32 {
    let Some(app) = app_from_ptr(app) else {
        return -1;
    };
    if data.is_null() || len == 0 {
        return 0;
    }
    let sid = match ghostty_core::SurfaceId::from_raw(id.raw) {
        Some(s) => s,
        None => return -1,
    };
    let bytes = core::slice::from_raw_parts(data, len);
    if app.write_surface(sid, bytes) {
        0
    } else {
        -1
    }
}

/// Resize a surface grid (returns 1 on success, 0 on failure).
#[cfg(all(unix, feature = "rust-vt"))]
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_surface_resize(
    app: *mut GhosttyApp,
    id: GhosttySurfaceId,
    cols: u16,
    rows: u16,
) -> i32 {
    let Some(app) = app_from_ptr(app) else {
        return 0;
    };
    let sid = match ghostty_core::SurfaceId::from_raw(id.raw) {
        Some(s) => s,
        None => return 0,
    };
    i32::from(app.resize_surface(sid, cols, rows))
}

/// Drain one pending event from the last tick into `out` (returns false when empty).
#[no_mangle]
pub unsafe extern "C" fn ghostty_app_poll_event(
    app: *mut GhosttyApp,
    out: *mut GhosttyAppEvent,
) -> bool {
    if out.is_null() {
        return false;
    }
    let Some(app) = app_from_ptr(app) else {
        return false;
    };
    let Some(event) = app.take_polled_event() else {
        return false;
    };
    unsafe {
        *out = app_event_to_c(&event);
    }
    true
}

fn app_event_to_c(event: &AppEvent) -> GhosttyAppEvent {
    match event {
        AppEvent::Quit => GhosttyAppEvent {
            tag: GhosttyAppEventTag::Quit,
            ..Default::default()
        },
        AppEvent::CloseSurface { id } => GhosttyAppEvent {
            tag: GhosttyAppEventTag::CloseSurface,
            surface_id: GhosttySurfaceId { raw: id.get() },
            ..Default::default()
        },
        AppEvent::Surface {
            id,
            event: SurfaceEvent::ChildExited { exit_code },
        } => GhosttyAppEvent {
            tag: GhosttyAppEventTag::SurfaceChildExited,
            surface_id: GhosttySurfaceId { raw: id.get() },
            exit_code: *exit_code,
            ..Default::default()
        },
        AppEvent::Surface {
            id,
            event: SurfaceEvent::FocusChanged { focused },
        } => GhosttyAppEvent {
            tag: GhosttyAppEventTag::SurfaceFocusChanged,
            surface_id: GhosttySurfaceId { raw: id.get() },
            focused: *focused,
            ..Default::default()
        },
        AppEvent::Surface {
            id,
            event: SurfaceEvent::TitleChanged { .. },
        } => GhosttyAppEvent {
            tag: GhosttyAppEventTag::SurfaceTitleChanged,
            surface_id: GhosttySurfaceId { raw: id.get() },
            ..Default::default()
        },
        AppEvent::Surface {
            id,
            event: SurfaceEvent::RedrawRequested,
        } => GhosttyAppEvent {
            tag: GhosttyAppEventTag::SurfaceRedrawRequested,
            surface_id: GhosttySurfaceId { raw: id.get() },
            ..Default::default()
        },
        _ => GhosttyAppEvent::default(),
    }
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
    pub unsafe extern "C" fn ghostty_surface_session_spawn_default() -> *mut GhosttySurfaceSession {
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

    /// Resize the PTY grid (returns 0 on success, -1 on error).
    #[no_mangle]
    pub unsafe extern "C" fn ghostty_surface_session_resize(
        session: *mut GhosttySurfaceSession,
        cols: u16,
        rows: u16,
    ) -> i32 {
        let Some(session) = session_from_ptr(session) else {
            return -1;
        };
        match session.resize(cols, rows) {
            Ok(()) => 0,
            Err(_) => -1,
        }
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
        let app = unsafe { ghostty_app_new(&cfg, ptr::null()) };
        assert!(!app.is_null());
        unsafe { ghostty_app_tick(app) };
        assert!(unsafe { ghostty_app_userdata(app) }.is_null());
        unsafe { ghostty_app_free(app) };
    }

    #[test]
    fn null_app_tick_is_noop() {
        unsafe { ghostty_app_tick(ptr::null_mut()) };
    }

    #[test]
    fn app_create_delete_surface() {
        let cfg = GhosttyRuntimeConfig::default();
        let app = unsafe { ghostty_app_new(&cfg, ptr::null()) };
        assert!(!app.is_null());
        let id = unsafe { ghostty_app_create_surface(app) };
        assert_ne!(id.raw, 0);
        assert_eq!(unsafe { ghostty_app_surface_count(app) }, 1);
        assert!(unsafe { ghostty_app_delete_surface(app, id) });
        assert_eq!(unsafe { ghostty_app_surface_count(app) }, 0);
        unsafe { ghostty_app_free(app) };
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    #[test]
    fn surface_prepare_finish_draw() {
        let cfg = GhosttyRuntimeConfig::default();
        let app = unsafe { ghostty_app_new(&cfg, ptr::null()) };
        let id = unsafe { ghostty_app_create_surface(app) };
        unsafe { ghostty_app_tick(app) };
        let count = unsafe { ghostty_app_surface_prepare_draw(app, id) };
        assert_ne!(count, usize::MAX);
        unsafe { ghostty_app_surface_finish_draw(app, id) };
        unsafe { ghostty_app_delete_surface(app, id) };
        unsafe { ghostty_app_free(app) };
    }

    #[test]
    fn app_poll_close_surface_event() {
        let cfg = GhosttyRuntimeConfig::default();
        let app = unsafe { ghostty_app_new(&cfg, ptr::null()) };
        let id = unsafe { ghostty_app_create_surface(app) };
        assert!(unsafe { ghostty_app_delete_surface(app, id) });
        unsafe { ghostty_app_tick(app) };
        let mut event = GhosttyAppEvent::default();
        assert!(unsafe { ghostty_app_poll_event(app, &mut event) });
        assert_eq!(event.tag, GhosttyAppEventTag::CloseSurface);
        assert_eq!(event.surface_id.raw, id.raw);
        unsafe { ghostty_app_free(app) };
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    mod surface_session {
        use super::*;
        use ghostty_core::{AppConfig, SurfaceSession, SurfaceSessionOptions};
        use ghostty_termio::{CommandBuilder, CommandSpec};

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
            let found =
                unsafe { ghostty_surface_session_run_until_text(raw, c"ffi".as_ptr(), 3000) };
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
