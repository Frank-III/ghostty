//! Terminal surface (`Surface.zig`, `apprt/embedded.zig` `Surface`).
//!
//! With feature `rust-vt`, each surface owns a headless [`SurfaceSession`] (config +
//! termio + Rust VT). Renderer and input remain unported.

use core::ffi::c_void;

use crate::SurfaceId;

#[cfg(all(unix, feature = "rust-vt"))]
use ghostty_foundation::FoundationResult;

#[cfg(all(unix, feature = "rust-vt"))]
use crate::surface_session::SurfaceSession;

/// A single terminal surface (widget) owned by an [`App`](crate::App).
#[derive(Debug)]
pub struct Surface {
    id: SurfaceId,
    title: Option<String>,
    userdata: *mut c_void,
    focused: bool,
    #[cfg(all(unix, feature = "rust-vt"))]
    session: Option<SurfaceSession>,
}

impl Surface {
    pub(crate) fn new(id: SurfaceId) -> Self {
        Self {
            id,
            title: None,
            userdata: core::ptr::null_mut(),
            focused: false,
            #[cfg(all(unix, feature = "rust-vt"))]
            session: None,
        }
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub(crate) fn with_session(id: SurfaceId, session: SurfaceSession) -> Self {
        debug_assert_eq!(session.id(), id);
        Self {
            id,
            title: None,
            userdata: core::ptr::null_mut(),
            focused: false,
            session: Some(session),
        }
    }

    pub fn id(&self) -> SurfaceId {
        self.id
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    pub fn userdata(&self) -> *mut c_void {
        self.userdata
    }

    pub fn set_userdata(&mut self, userdata: *mut c_void) {
        self.userdata = userdata;
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn session(&self) -> Option<&SurfaceSession> {
        self.session.as_ref()
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn session_mut(&mut self) -> Option<&mut SurfaceSession> {
        self.session.as_mut()
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn write(&mut self, bytes: &[u8]) -> FoundationResult<()> {
        match self.session.as_mut() {
            Some(session) => session.write(bytes),
            None => Err(ghostty_foundation::FoundationError::Unsupported),
        }
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn resize(&mut self, cols: u16, rows: u16) -> FoundationResult<()> {
        match self.session.as_mut() {
            Some(session) => session.resize(cols, rows),
            None => Err(ghostty_foundation::FoundationError::Unsupported),
        }
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn contains_text(&self, needle: &str) -> bool {
        self.session
            .as_ref()
            .map(|s| s.contains_text(needle))
            .unwrap_or(false)
    }

    /// Pump termio + PTY for this surface; returns PTY bytes read.
    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn tick(&mut self) -> FoundationResult<usize> {
        match self.session.as_mut() {
            Some(session) => session.tick(),
            None => Ok(0),
        }
    }

    /// Non-blocking child exit check (emits at most once per session).
    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn poll_child_exit(&mut self) -> Option<u32> {
        self.session.as_mut()?.poll_child_exit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_round_trip() {
        let id = SurfaceId::from_raw(1).unwrap();
        let mut surface = Surface::new(id);
        surface.set_title("hello");
        assert_eq!(surface.title(), Some("hello"));
    }
}
