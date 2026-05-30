//! Core PTY + rust-owned VT integration for [`SurfaceSession`].

use std::time::{Duration, Instant};

use ghostty_config::Config;
use ghostty_core::{AppConfig, SurfaceEvent, SurfaceSession, SurfaceSessionOptions};
use ghostty_termio::{CommandBuilder, CommandSpec, TermioMessage};

fn echo_cat_command() -> CommandSpec {
    CommandBuilder::new()
        .path("/bin/sh")
        .arg("sh")
        .arg("-c")
        .arg("cat")
        .build()
        .expect("cat spec")
}

fn short_shell_command() -> CommandSpec {
    CommandBuilder::new()
        .path("/bin/sh")
        .arg("sh")
        .arg("-c")
        .arg("true")
        .build()
        .expect("spec")
}

fn finish_session(session: &mut SurfaceSession) {
    let _ = session.shutdown();
    let deadline = Instant::now() + Duration::from_secs(3);
    let _ = session.run_until(deadline, |_| false);
}

#[test]
fn surface_session_integration() {
    let session = SurfaceSession::spawn(
        AppConfig::default(),
        SurfaceSessionOptions {
            command: Some(short_shell_command()),
            ..Default::default()
        },
    )
    .expect("spawn");
    assert!(session.pid() > 0);
    assert_eq!(session.id().get(), 1);

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
    finish_session(&mut session);

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
    finish_session(&mut session);

    let mut cfg = Config::with_defaults();
    cfg.command = Some("printf cfg-cmd".to_string());
    let mut session = SurfaceSession::spawn(AppConfig::new(cfg), SurfaceSessionOptions::default())
        .expect("spawn");
    let deadline = Instant::now() + Duration::from_secs(3);
    session
        .run_until(deadline, |s| s.contains_text("cfg-cmd"))
        .expect("run");
    assert!(session.contains_text("cfg-cmd"));
    finish_session(&mut session);

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
    let mut saw_color = false;
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
            saw_color = true;
            finish_session(&mut session);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(saw_color, "expected ColorChanged for background");

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
    finish_session(&mut session);

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
    let mut saw_title = false;
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        session.tick().expect("tick");
        let events = session.drain_session_events();
        if events.iter().any(|e| {
            matches!(
                e,
                SurfaceEvent::TitleChanged { title } if title == "term-title"
            )
        }) {
            saw_title = true;
            finish_session(&mut session);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(saw_title, "expected TitleChanged event from SetTitle");

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
    finish_session(&mut session);

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
    finish_session(&mut session);
}
