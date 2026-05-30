//! Bridge rust-owned VT effect callbacks into [`StreamHandler`].

use core::ffi::c_void;

use ghostty_vt::GhosttySizeReportSize;
use ghostty_vt::GhosttyVtEffectWrapper;

use crate::stream_handler::StreamHandler;
use crate::termio::TermioMessage;
use crate::TermioLoop;

const TERMINAL_DATA_TITLE: i32 = 12;
const DEFAULT_CELL_WIDTH_PX: u32 = 8;
const DEFAULT_CELL_HEIGHT_PX: u32 = 16;

fn osc_report_format_byte(format: ghostty_config::OscColorReportFormat) -> u8 {
    match format {
        ghostty_config::OscColorReportFormat::None => 0,
        ghostty_config::OscColorReportFormat::SixteenBit => 1,
        ghostty_config::OscColorReportFormat::EightBit => 2,
    }
}

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
    termio: *mut TermioLoop,
}

impl TermioVtBridge {
    pub fn new(terminal: *mut c_void) -> Box<Self> {
        let mut bridge = Box::new(Self {
            effects: GhosttyVtEffectWrapper::empty(),
            terminal,
            stream: core::ptr::null_mut(),
            termio: core::ptr::null_mut(),
        });
        let userdata = &mut *bridge as *mut Self as *mut c_void;
        bridge.effects.userdata = userdata;
        bridge.effects.title_changed = Some(title_changed);
        bridge.effects.bell = Some(bell);
        bridge.effects.clipboard_contents = Some(clipboard_contents);
        bridge.effects.write_pty = Some(write_pty);
        bridge.effects.report_enquiry = Some(report_enquiry);
        bridge.effects.report_xtversion = Some(report_xtversion);
        bridge.effects.report_device_attributes = Some(report_device_attributes);
        bridge.effects.query_size = Some(query_size);
        bridge.effects.report_color_scheme = Some(report_color_scheme);
        bridge.effects.color_changed = Some(color_changed);
        bridge
    }

    pub fn bind_stream(&mut self, stream: &mut StreamHandler) {
        self.stream = stream as *mut StreamHandler;
        self.effects.osc_color_report_format =
            osc_report_format_byte(stream.config().osc_color_report_format);
    }

    pub fn bind_termio(&mut self, termio: &mut TermioLoop) {
        self.termio = termio as *mut TermioLoop;
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

unsafe extern "C" fn report_enquiry(wrapper: *mut c_void) {
    let Some((bridge, stream)) = bridge_and_stream(wrapper) else {
        return;
    };
    let response = stream.config().enquiry_response.as_bytes();
    if response.is_empty() {
        return;
    }
    push_termio_write(bridge, response);
}

unsafe extern "C" fn report_xtversion(_wrapper: *mut c_void) {
    let Some(bridge) = bridge_from_wrapper(_wrapper) else {
        return;
    };
    push_termio_write(bridge, b"\x1bP>|libghostty\x1b\\");
}

unsafe extern "C" fn report_device_attributes(_wrapper: *mut c_void, req: u8) {
    let Some(bridge) = bridge_from_wrapper(_wrapper) else {
        return;
    };
    let response: &[u8] = match req {
        0 => b"\x1b[?62;22c",
        1 => b"\x1b[>0;0;0c",
        2 => b"\x1bP!|00000000\x1b\\",
        _ => return,
    };
    push_termio_write(bridge, response);
}

unsafe extern "C" fn report_color_scheme(wrapper: *mut c_void) {
    let Some((bridge, stream)) = bridge_and_stream(wrapper) else {
        return;
    };
    let response = if stream.config().color_scheme_dark {
        b"\x1b[?997;1n"
    } else {
        b"\x1b[?997;2n"
    };
    push_termio_write(bridge, response);
}

unsafe extern "C" fn query_size(wrapper: *mut c_void, out: *mut GhosttySizeReportSize) -> bool {
    let Some(bridge) = bridge_from_wrapper(wrapper) else {
        return false;
    };
    if out.is_null() || bridge.termio.is_null() {
        return false;
    }
    let winsize = unsafe { (*bridge.termio).winsize() };
    unsafe {
        (*out).rows = winsize.rows;
        (*out).columns = winsize.cols;
        (*out).cell_width = DEFAULT_CELL_WIDTH_PX;
        (*out).cell_height = DEFAULT_CELL_HEIGHT_PX;
    }
    true
}

unsafe extern "C" fn write_pty(wrapper: *mut c_void, ptr: *const u8, len: usize) {
    if wrapper.is_null() || ptr.is_null() || len == 0 {
        return;
    }
    let Some(bridge) = bridge_from_wrapper(wrapper) else {
        return;
    };
    let bytes = core::slice::from_raw_parts(ptr, len);
    push_termio_write(bridge, bytes);
}

unsafe extern "C" fn color_changed(wrapper: *mut c_void, kind: i32, r: u8, g: u8, b: u8) {
    let Some((_, stream)) = bridge_and_stream(wrapper) else {
        return;
    };
    let _ = stream.on_color_change(kind, ghostty_config::RgbColor { r, g, b });
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

fn push_termio_write(bridge: &TermioVtBridge, bytes: &[u8]) {
    if bridge.termio.is_null() || bytes.is_empty() {
        return;
    }
    let _ = unsafe { (*bridge.termio).push(TermioMessage::Write(bytes.to_vec())) };
}

unsafe fn bridge_from_wrapper(wrapper: *mut c_void) -> Option<&'static TermioVtBridge> {
    if wrapper.is_null() {
        return None;
    }
    let effects = &*(wrapper as *const GhosttyVtEffectWrapper);
    if effects.userdata.is_null() {
        return None;
    }
    Some(&*(effects.userdata as *const TermioVtBridge))
}

unsafe fn bridge_and_stream(
    wrapper: *mut c_void,
) -> Option<(&'static TermioVtBridge, &'static mut StreamHandler)> {
    let bridge = bridge_from_wrapper(wrapper)?;
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
    fn query_size_reports_winsize() {
        use crate::{CommandBuilder, TermioLoop, Winsize};
        use ghostty_config::DerivedStreamConfig;
        use ghostty_vt::GhosttySizeReportSize;

        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("sleep 10")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 120,
            rows: 40,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut termio =
            TermioLoop::spawn(&spec, winsize, DerivedStreamConfig::default()).expect("spawn");
        let mut bridge = TermioVtBridge::new(core::ptr::null_mut());
        bridge.bind_termio(&mut termio);
        let mut out = GhosttySizeReportSize {
            rows: 0,
            columns: 0,
            cell_width: 0,
            cell_height: 0,
        };
        let f = bridge.effects.query_size.expect("query_size");
        let ok = unsafe { f(bridge.effects_ptr(), &mut out) };
        assert!(ok);
        assert_eq!(out.columns, 120);
        assert_eq!(out.rows, 40);
        assert_eq!(out.cell_width, 8);
        assert_eq!(out.cell_height, 16);
    }

    #[test]
    fn bridge_registers_callbacks() {
        let mut bridge = TermioVtBridge::new(core::ptr::null_mut());
        let mut stream = StreamHandler::new((&Config::with_defaults()).into());
        bridge.bind_stream(&mut stream);
        assert!(bridge.effects.title_changed.is_some());
        assert!(bridge.effects.bell.is_some());
        assert!(bridge.effects.clipboard_contents.is_some());
        assert!(bridge.effects.write_pty.is_some());
        assert!(bridge.effects.report_enquiry.is_some());
        assert!(bridge.effects.report_xtversion.is_some());
        assert!(bridge.effects.report_device_attributes.is_some());
        assert!(bridge.effects.query_size.is_some());
        assert!(bridge.effects.report_color_scheme.is_some());
    }
}
