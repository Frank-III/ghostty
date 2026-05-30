//! Integration tests for headless SurfaceSession (isolated process from lib unit tests).

use std::time::{Duration, Instant};

use ghostty_config::Config;
use ghostty_core::{AppConfig, SurfaceSession, SurfaceSessionOptions};
use ghostty_termio::{CommandBuilder, CommandSpec};

fn echo_cat_command() -> CommandSpec {
    CommandBuilder::new()
        .path("/bin/sh")
        .arg("sh")
        .arg("-c")
        .arg("cat")
        .build()
        .expect("cat spec")
}

#[test]
fn spawn_default_shell() {
    let session = SurfaceSession::from_defaults().expect("spawn");
    assert!(session.pid() > 0);
    assert_eq!(session.id().get(), 1);
}

#[test]
fn write_echo_round_trip() {
    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session.write(b"ping\n").expect("write");
    let deadline = Instant::now() + Duration::from_secs(3);
    session
        .run_until(deadline, |s| s.contains_text("ping"))
        .expect("run");
    assert!(session.contains_text("ping"));
}

#[test]
fn child_output_reaches_terminal() {
    let spec = CommandBuilder::new()
        .path("/bin/sh")
        .arg("sh")
        .arg("-c")
        .arg("printf 'core-vt'")
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

    let deadline = Instant::now() + Duration::from_secs(3);
    session
        .run_until(deadline, |s| s.contains_text("core-vt"))
        .expect("run");
    assert!(session.contains_text("core-vt"));
}

#[test]
fn config_command_used() {
    let mut cfg = Config::with_defaults();
    cfg.command = Some("printf cfg-cmd".to_string());
    let mut session = SurfaceSession::spawn(AppConfig::new(cfg), SurfaceSessionOptions::default())
        .expect("spawn");

    let deadline = Instant::now() + Duration::from_secs(3);
    session
        .run_until(deadline, |s| s.contains_text("cfg-cmd"))
        .expect("run");
    assert!(session.contains_text("cfg-cmd"));
}

#[test]
fn resize_updates_terminal() {
    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session.resize(120, 40).expect("resize");
    assert_eq!(session.winsize().cols, 120);
    assert_eq!(session.winsize().rows, 40);
    session.write(b"x").expect("write");
}

#[test]
fn termio_set_title_emits_session_event() {
    use ghostty_termio::TermioMessage;

    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session
        .push_termio(TermioMessage::SetTitle("term-title".into()))
        .expect("push");

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        session.tick().expect("tick");
        let events = session.drain_session_events();
        if events.iter().any(|e| {
            matches!(
                e,
                ghostty_core::SurfaceEvent::TitleChanged { title } if title == "term-title"
            )
        }) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("expected TitleChanged event from SetTitle");
}

#[test]
fn cell_snapshot_captures_output() {
    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session.write(b"Z\n").expect("write");
    let deadline = Instant::now() + Duration::from_secs(3);
    session
        .run_until(deadline, |s| s.contains_text("Z"))
        .expect("run");

    let snap = session.cell_snapshot();
    assert!(snap
        .codepoints
        .iter()
        .flatten()
        .any(|cp| *cp == b'Z' as u32));
    assert!(session.damage().is_dirty());
}

#[test]
fn prepare_draw_uploads_font_atlas() {
    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");
    session.write(b"A").expect("write");
    let deadline = Instant::now() + Duration::from_secs(3);
    session
        .run_until(deadline, |s| s.contains_text("A"))
        .expect("run");
    assert_eq!(session.atlas_upload_generation(), 0);
    let prep = session.prepare_draw();
    assert!(prep.grid.columns > 0);
    assert!(session.atlas_upload_generation() > 0);
    assert!(session.glyph_cache_len() > 0);
    assert!(!session.last_frame_prep().unwrap().text_cells.is_empty());
    let _ = session.shutdown();
}

#[test]
fn osc_background_change_emits_surface_event() {
    use ghostty_core::SurfaceEvent;

    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session
        .write_vt_input(b"\x1b]11;rgb:aa/bb/cc\x07")
        .expect("write");
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        session.tick().expect("tick");
        let events = session.drain_session_events();
        if events.iter().any(|e| {
            matches!(
                e,
                SurfaceEvent::ColorChanged { kind: -2, color }
                    if color.r == 0xaa && color.g == 0xbb && color.b == 0xcc
            )
        }) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("expected ColorChanged for background");
}

#[test]
fn cell_snapshot_captures_sgr_foreground() {
    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session.write_vt_input(b"\x1b[31mR").expect("write");
    let deadline = Instant::now() + Duration::from_secs(1);
    session
        .run_until(deadline, |s| s.cell_codepoint(0, 0) == Some(b'R' as u32))
        .expect("run");

    let rgb = session.cell_fg_rgb(0, 0).expect("foreground");
    assert_eq!(rgb, [0xcc, 0x66, 0x66]);

    let snap = session.cell_snapshot();
    let idx = snap
        .codepoints
        .iter()
        .position(|cp| *cp == Some(b'R' as u32))
        .expect("R cell");
    let fg = snap.foregrounds[idx].expect("snapshot foreground");
    assert_eq!(fg.r, 0xcc);
    assert_eq!(fg.g, 0x66);
    assert_eq!(fg.b, 0x66);
}

#[test]
fn prepare_draw_includes_background_cells() {
    let mut session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(echo_cat_command()),
            ..Default::default()
        },
    )
    .expect("spawn");

    session.write_vt_input(b"\x1b[42mG").expect("write");
    let prep = session.prepare_draw();
    assert!(!prep.bg_cells.is_empty());
    assert_eq!(prep.bg_cells[0].color[0..3], [0xb5, 0xbd, 0x68]);
}
