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

    use ghostty_vt::test_support::{terminal_cell_codepoint, test_allocator};

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
    }

    impl RustOwnedTerminalSink {
        pub fn new(cols: u16, rows: u16, max_scrollback: usize) -> Option<Self> {
            let alloc = test_allocator();
            unsafe {
                let handle = ghostty_rust_terminal_create(&alloc, cols, rows, max_scrollback);
                if handle.is_null() {
                    return None;
                }
                Some(Self { handle, alloc })
            }
        }

        pub fn cell_codepoint(&self, x: u16, y: u16) -> Option<u32> {
            terminal_cell_codepoint(self.handle, x, y)
        }

        pub fn contains_text(&self, needle: &str) -> bool {
            for (i, ch) in needle.chars().enumerate() {
                if self.cell_codepoint(i as u16, 0) != Some(ch as u32) {
                    return false;
                }
            }
            !needle.is_empty()
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
