//! Bridge rust-owned VT effect callbacks into [`StreamHandler`].

use core::ffi::c_void;

use ghostty_vt::GhosttyVtEffectWrapper;

use crate::stream_handler::StreamHandler;

const TERMINAL_DATA_TITLE: i32 = 12;

#[repr(C)]
struct TitleOut {
    ptr: *const u8,
    len: usize,
}

/// Owns the C effect wrapper registered on a rust-owned terminal handle.
pub struct TermioVtBridge {
    pub effects: GhosttyVtEffectWrapper,
    terminal: *mut c_void,
    stream: *mut StreamHandler,
}

impl TermioVtBridge {
    pub fn new(terminal: *mut c_void) -> Box<Self> {
        let mut bridge = Box::new(Self {
            effects: GhosttyVtEffectWrapper::empty(),
            terminal,
            stream: core::ptr::null_mut(),
        });
        let userdata = &mut *bridge as *mut Self as *mut c_void;
        bridge.effects.userdata = userdata;
        bridge.effects.title_changed = Some(title_changed);
        bridge.effects.bell = Some(bell);
        bridge.effects.clipboard_contents = Some(clipboard_contents);
        bridge
    }

    pub fn bind_stream(&mut self, stream: &mut StreamHandler) {
        self.stream = stream as *mut StreamHandler;
    }

    pub fn effects_ptr(&mut self) -> *mut c_void {
        &mut self.effects as *mut GhosttyVtEffectWrapper as *mut c_void
    }
}

unsafe extern "C" fn title_changed(wrapper: *mut c_void) {
    let Some((bridge, stream)) = bridge_and_stream(wrapper) else {
        return;
    };
    let Some(title) = terminal_title(bridge.terminal) else {
        return;
    };
    let _ = stream.on_set_title(title);
}

unsafe extern "C" fn bell(wrapper: *mut c_void) {
    let Some((_, stream)) = bridge_and_stream(wrapper) else {
        return;
    };
    let _ = stream.on_bell();
}

unsafe extern "C" fn clipboard_contents(
    wrapper: *mut c_void,
    kind: u8,
    data: *const u8,
    len: usize,
) {
    let Some((_, stream)) = bridge_and_stream(wrapper) else {
        return;
    };
    if data.is_null() {
        return;
    }
    let bytes = core::slice::from_raw_parts(data, len);
    let _ = stream.on_clipboard_contents(kind, bytes);
}

unsafe fn bridge_and_stream(
    wrapper: *mut c_void,
) -> Option<(&'static TermioVtBridge, &'static mut StreamHandler)> {
    if wrapper.is_null() {
        return None;
    }
    let effects = &*(wrapper as *const GhosttyVtEffectWrapper);
    if effects.userdata.is_null() {
        return None;
    }
    let bridge = &*(effects.userdata as *const TermioVtBridge);
    if bridge.stream.is_null() {
        return None;
    }
    Some((bridge, &mut *bridge.stream))
}

fn terminal_title(handle: *mut c_void) -> Option<String> {
    extern "C" {
        fn ghostty_rust_terminal_owned_get_string(
            handle: *mut c_void,
            data: i32,
            out: *mut c_void,
        ) -> i32;
    }
    let mut out = TitleOut {
        ptr: core::ptr::null(),
        len: 0,
    };
    let rc = unsafe {
        ghostty_rust_terminal_owned_get_string(
            handle,
            TERMINAL_DATA_TITLE,
            &mut out as *mut TitleOut as *mut c_void,
        )
    };
    if rc != 0 || out.ptr.is_null() {
        return None;
    }
    let slice = unsafe { core::slice::from_raw_parts(out.ptr, out.len) };
    std::str::from_utf8(slice).ok().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghostty_config::Config;

    #[test]
    fn bridge_registers_callbacks() {
        let mut bridge = TermioVtBridge::new(core::ptr::null_mut());
        let mut stream = StreamHandler::new((&Config::with_defaults()).into());
        bridge.bind_stream(&mut stream);
        assert!(bridge.effects.title_changed.is_some());
        assert!(bridge.effects.bell.is_some());
        assert!(bridge.effects.clipboard_contents.is_some());
    }
}
