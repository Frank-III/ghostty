use core::ffi::c_void;
use std::ptr::NonNull;

use crate::harness::TermioSink;

pub type VtWriteFn = unsafe extern "C" fn(*mut c_void, *const u8, usize);
pub type VtResizeFn = unsafe extern "C" fn(*mut c_void, u16, u16);

pub struct VtSink {
    handle: NonNull<c_void>,
    write_fn: VtWriteFn,
    resize_fn: VtResizeFn,
}

unsafe impl Send for VtSink {}

impl VtSink {
    pub unsafe fn new(
        handle: *mut c_void,
        write_fn: VtWriteFn,
        resize_fn: VtResizeFn,
    ) -> Option<Self> {
        Some(Self {
            handle: NonNull::new(handle)?,
            write_fn,
            resize_fn,
        })
    }

    pub fn handle(&self) -> *mut c_void {
        self.handle.as_ptr()
    }
}

impl TermioSink for VtSink {
    fn write_terminal(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        unsafe { (self.write_fn)(self.handle.as_ptr(), bytes.as_ptr(), bytes.len()) };
    }

    fn resize_terminal(&mut self, cols: u16, rows: u16) {
        unsafe { (self.resize_fn)(self.handle.as_ptr(), cols, rows) };
    }
}

/// Rust-owned VT wired through `ghostty-vt` (requires dev-dependency + pool stub).
#[cfg(feature = "rust-vt")]
pub mod rust_owned {
    use core::ffi::c_void;

    use ghostty_vt::test_support::{terminal_cell_codepoint, terminal_cell_fg_rgb, test_allocator};

    use super::super::harness::TermioSink;

    extern "C" {
        fn ghostty_rust_terminal_create(
            alloc: *const ghostty_vt::GhosttyAllocator,
            cols: u16,
            rows: u16,
            max_scrollback: usize,
        ) -> *mut c_void;
        fn ghostty_rust_terminal_destroy(
            alloc: *const ghostty_vt::GhosttyAllocator,
            handle: *mut c_void,
        );
        fn ghostty_rust_terminal_write(handle: *mut c_void, ptr: *const u8, len: usize);
        fn ghostty_rust_terminal_owned_set_wrapper(handle: *mut c_void, wrapper: *mut c_void);
        fn ghostty_rust_terminal_owned_resize(
            handle: *mut c_void,
            alloc: *const ghostty_vt::GhosttyAllocator,
            cols: u16,
            rows: u16,
            cell_width_px: u32,
            cell_height_px: u32,
            out_width_px: *mut u32,
            out_height_px: *mut u32,
        ) -> i32;
    }

    pub struct RustOwnedTerminalSink {
        handle: *mut c_void,
        alloc: ghostty_vt::GhosttyAllocator,
        #[cfg(feature = "rust-vt")]
        bridge: Option<Box<crate::vt_effects::TermioVtBridge>>,
    }

    impl RustOwnedTerminalSink {
        pub fn new(cols: u16, rows: u16, max_scrollback: usize) -> Option<Self> {
            let alloc = test_allocator();
            unsafe {
                let handle = ghostty_rust_terminal_create(&alloc, cols, rows, max_scrollback);
                if handle.is_null() {
                    return None;
                }
                Some(Self {
                    handle,
                    alloc,
                    bridge: None,
                })
            }
        }

        /// Attach stream-handler side effects and PTY write-back for VT responses.
        pub fn bind_session(&mut self, termio: &mut crate::TermioLoop) {
            termio.bind_rust_terminal(self);
        }

        pub(crate) fn attach_vt_bridge(
            &mut self,
            stream: &mut crate::StreamHandler,
            termio: &mut crate::TermioLoop,
        ) {
            if self.bridge.is_none() {
                let mut bridge = crate::vt_effects::TermioVtBridge::new(self.handle);
                unsafe {
                    ghostty_rust_terminal_owned_set_wrapper(self.handle, bridge.effects_ptr());
                }
                self.bridge = Some(bridge);
            }
            let bridge = self.bridge.as_mut().expect("bridge");
            bridge.bind_stream(stream);
            bridge.bind_termio(termio);
        }

        /// Attach stream-handler side effects to VT parser callbacks for this sink.
        pub fn bind_stream_handler(&mut self, stream: &mut crate::StreamHandler) {
            if self.bridge.is_none() {
                let mut bridge = crate::vt_effects::TermioVtBridge::new(self.handle);
                unsafe {
                    ghostty_rust_terminal_owned_set_wrapper(self.handle, bridge.effects_ptr());
                }
                self.bridge = Some(bridge);
            }
            self.bridge
                .as_mut()
                .expect("bridge")
                .bind_stream(stream);
        }

        pub fn cell_codepoint(&self, x: u16, y: u16) -> Option<u32> {
            terminal_cell_codepoint(self.handle, x, y)
        }

        pub fn cell_fg_rgb(&self, x: u16, y: u16) -> Option<[u8; 3]> {
            terminal_cell_fg_rgb(self.handle, x, y)
        }

        pub fn contains_text(&self, needle: &str) -> bool {
            self.contains_text_at(0, needle)
        }

        pub fn contains_text_at(&self, row: u16, needle: &str) -> bool {
            for (i, ch) in needle.chars().enumerate() {
                if self.cell_codepoint(i as u16, row) != Some(ch as u32) {
                    return false;
                }
            }
            !needle.is_empty()
        }

        /// True if `needle` appears as consecutive cells on any screen row.
        pub fn contains_text_anywhere(&self, needle: &str) -> bool {
            if needle.is_empty() {
                return false;
            }
            let needle: Vec<u32> = needle.chars().map(|c| c as u32).collect();
            let width = 80u16;
            let height = 24u16;
            for y in 0..height {
                for x in 0..width.saturating_sub(needle.len() as u16) {
                    let mut matched = true;
                    for (i, &cp) in needle.iter().enumerate() {
                        if self.cell_codepoint(x + i as u16, y) != Some(cp) {
                            matched = false;
                            break;
                        }
                    }
                    if matched {
                        return true;
                    }
                }
            }
            false
        }
    }

    impl Drop for RustOwnedTerminalSink {
        fn drop(&mut self) {
            unsafe {
                ghostty_rust_terminal_destroy(&self.alloc, self.handle);
            }
        }
    }

    impl TermioSink for RustOwnedTerminalSink {
        fn write_terminal(&mut self, bytes: &[u8]) {
            if bytes.is_empty() {
                return;
            }
            unsafe {
                ghostty_rust_terminal_write(self.handle, bytes.as_ptr(), bytes.len());
            }
        }

        fn resize_terminal(&mut self, cols: u16, rows: u16) {
            let mut width_px: u32 = 0;
            let mut height_px: u32 = 0;
            unsafe {
                let _ = ghostty_rust_terminal_owned_resize(
                    self.handle,
                    &self.alloc,
                    cols,
                    rows,
                    10,
                    20,
                    &mut width_px,
                    &mut height_px,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Captured {
        bytes: Vec<u8>,
        cols: u16,
        rows: u16,
    }

    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static LAST_CAPTURE: std::sync::LazyLock<Arc<Mutex<Captured>>> =
        std::sync::LazyLock::new(|| Arc::new(Mutex::new(Captured::default())));

    unsafe extern "C" fn stub_write(_: *mut c_void, ptr: *const u8, len: usize) {
        CALLS.fetch_add(1, Ordering::SeqCst);
        let slice = std::slice::from_raw_parts(ptr, len);
        LAST_CAPTURE.lock().unwrap().bytes.extend_from_slice(slice);
    }

    unsafe extern "C" fn stub_resize(_: *mut c_void, cols: u16, rows: u16) {
        CALLS.fetch_add(1, Ordering::SeqCst);
        let mut c = LAST_CAPTURE.lock().unwrap();
        c.cols = cols;
        c.rows = rows;
    }

    #[test]
    fn vt_sink_forwards_write_and_resize() {
        CALLS.store(0, Ordering::SeqCst);
        *LAST_CAPTURE.lock().unwrap() = Captured::default();

        let stub: *mut c_void = 0x1 as *mut c_void;
        let mut sink = unsafe { VtSink::new(stub, stub_write, stub_resize) }.unwrap();

        assert_eq!(sink.handle(), stub);

        sink.write_terminal(b"hello ");
        sink.write_terminal(b"world\x1b[H");
        sink.resize_terminal(132, 43);

        let c = LAST_CAPTURE.lock().unwrap();
        assert_eq!(c.bytes, b"hello world\x1b[H");
        assert_eq!(c.cols, 132);
        assert_eq!(c.rows, 43);
        assert_eq!(CALLS.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn vt_sink_rejects_null_handle() {
        let r = unsafe { VtSink::new(core::ptr::null_mut(), stub_write, stub_resize) };
        assert!(r.is_none());
    }
}

#[cfg(all(unix, test, feature = "rust-vt"))]
mod rust_vt_tests {
    use std::time::{Duration, Instant};

    use super::rust_owned::RustOwnedTerminalSink;
    use crate::command::CommandBuilder;
    use crate::harness::{TermioHarness, TermioSink};
    use crate::winsize::Winsize;

    #[test]
    fn direct_write_updates_grid() {
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        sink.write_terminal(b"vt-e2e");
        assert!(sink.contains_text("vt-e2e"));
    }

    #[test]
    fn osc_title_reaches_stream_handler() {
        use crate::StreamHandler;
        use ghostty_config::Config;

        let mut stream = StreamHandler::new((&Config::with_defaults()).into());
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        sink.bind_stream_handler(&mut stream);
        sink.write_terminal(b"\x1b]0;direct-title\x07");
        assert!(matches!(
            stream.surface_mailbox().pop(),
            Some(crate::SurfaceMessage::SetTitle(title)) if title == "direct-title"
        ));
    }

    #[test]
    fn enquiry_response_reaches_terminal() {
        use std::time::{Duration, Instant};

        use crate::{CommandBuilder, TermioLoop, Winsize};
        use ghostty_config::DerivedStreamConfig;

        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("cat")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut config = DerivedStreamConfig::default();
        config.enquiry_response = "ENQ-OK".to_string();
        let mut termio = TermioLoop::spawn(&spec, winsize, config).expect("spawn");
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        sink.bind_session(&mut termio);
        sink.write_terminal(&[0x05]);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            termio.tick(&mut sink).expect("tick");
            if sink.contains_text("ENQ-OK") {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("enquiry response not received");
    }

    #[test]
    fn osc4_query_response_reaches_pty() {
        use std::time::{Duration, Instant};

        use crate::{CommandBuilder, TermioLoop, Winsize};
        use ghostty_config::DerivedStreamConfig;

        // hexdump makes PTY write-back visible; raw `cat` would re-parse echoed OSC as
        // control sequences instead of printing literal "rgb:" on the grid.
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("hexdump -C")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut termio =
            TermioLoop::spawn(&spec, winsize, DerivedStreamConfig::default()).expect("spawn");
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        sink.bind_session(&mut termio);
        sink.write_terminal(b"\x1b]4;0;?\x07");
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            termio.tick(&mut sink).expect("tick");
            if sink.contains_text_anywhere("rgb:") {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("OSC 4 query response not received");
    }

    #[test]
    fn osc_background_set_emits_color_change() {
        use crate::{CommandBuilder, TermioLoop, Winsize};
        use ghostty_config::DerivedStreamConfig;

        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("cat")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut termio =
            TermioLoop::spawn(&spec, winsize, DerivedStreamConfig::default()).expect("spawn");
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        sink.bind_session(&mut termio);
        sink.write_terminal(b"\x1b]11;rgb:aa/bb/cc\x07");
        termio.tick(&mut sink).expect("tick");
        let msg = termio.drain_surface_mailbox();
        assert!(msg.iter().any(|m| matches!(
            m,
            crate::SurfaceMessage::ColorChange { kind: -2, color }
                if color.r == 0xaa && color.g == 0xbb && color.b == 0xcc
        )));
    }

    #[test]
    fn sgr_foreground_resolves_cell_rgb() {
        use crate::{CommandBuilder, TermioLoop, Winsize};
        use ghostty_config::DerivedStreamConfig;

        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("cat")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut termio =
            TermioLoop::spawn(&spec, winsize, DerivedStreamConfig::default()).expect("spawn");
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        sink.bind_session(&mut termio);
        sink.write_terminal(b"\x1b[31mR");
        assert_eq!(sink.cell_fg_rgb(0, 0), Some([0xcc, 0x66, 0x66]));
    }

    #[test]
    fn pty_output_reaches_rust_terminal() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("printf 'termio-vt'")
            .build()
            .expect("command spec");

        let winsize = Winsize {
            rows: 24,
            cols: 80,
            x_pixels: 0,
            y_pixels: 0,
        };

        let mut harness = TermioHarness::spawn(&spec, winsize).expect("spawn");
        let mut sink = RustOwnedTerminalSink::new(80, 24, 10_000).expect("terminal");
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            harness.drain_mailbox(&mut sink).expect("drain");
            let _ = harness.pump_pty(&mut sink);
            if sink.contains_text("termio-vt") {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(sink.contains_text("termio-vt"));
    }
}
