//! Integration tests for production [`TermioLoop`] (background thread).

use std::time::{Duration, Instant};

use ghostty_config::DerivedStreamConfig;
use ghostty_termio::{CommandBuilder, CommandSpec, TermioLoop, TermioMessage, TermioSink, Winsize};

struct VecSink(Vec<u8>);

impl TermioSink for VecSink {
    fn write_terminal(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    fn resize_terminal(&mut self, _cols: u16, _rows: u16) {}
}

fn cat_spec() -> CommandSpec {
    CommandBuilder::new()
        .path("/bin/sh")
        .arg("sh")
        .arg("-c")
        .arg("cat")
        .build()
        .expect("spec")
}

#[test]
fn loop_write_round_trip() {
    let winsize = Winsize {
        cols: 80,
        rows: 24,
        x_pixels: 0,
        y_pixels: 0,
    };
    let mut loop_ = TermioLoop::spawn(&cat_spec(), winsize, DerivedStreamConfig::default()).expect("spawn");
    loop_
        .push(TermioMessage::Write(b"loop-ping\n".to_vec()))
        .expect("write");

    let mut sink = VecSink(Vec::new());
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut saw = false;
    while Instant::now() < deadline {
        loop_.tick(&mut sink).expect("tick");
        if sink.0.windows(9).any(|w| w == b"loop-ping") {
            saw = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(saw);
    loop_.shutdown().expect("shutdown");
}

#[test]
fn loop_resize_and_shutdown() {
    let winsize = Winsize {
        cols: 80,
        rows: 24,
        x_pixels: 0,
        y_pixels: 0,
    };
    let mut loop_ = TermioLoop::spawn(&cat_spec(), winsize, DerivedStreamConfig::default()).expect("spawn");
    loop_
        .push(TermioMessage::Resize {
            cols: 100,
            rows: 40,
        })
        .expect("resize");
    let mut sink = VecSink(Vec::new());
    loop_.tick(&mut sink).expect("tick");
    assert_eq!(loop_.winsize().cols, 100);
    loop_.shutdown().expect("shutdown");
}
