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
