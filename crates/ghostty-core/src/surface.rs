//! Terminal surface skeleton (`Surface.zig`, `apprt/embedded.zig` `Surface`).
//!
//! No renderer, termio, or input loop — only identity and embedder-facing state.

use core::ffi::c_void;

use crate::SurfaceId;

/// A single terminal surface (widget) owned by an [`App`](crate::App).
#[derive(Debug)]
pub struct Surface {
    id: SurfaceId,
    title: Option<String>,
    userdata: *mut c_void,
    focused: bool,
}

impl Surface {
    pub(crate) fn new(id: SurfaceId) -> Self {
        Self {
            id,
            title: None,
            userdata: core::ptr::null_mut(),
            focused: false,
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
