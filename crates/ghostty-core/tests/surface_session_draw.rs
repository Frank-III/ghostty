//! Font atlas / prepare_draw integration (separate process from core PTY tests).

use std::time::{Duration, Instant};

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

fn finish_session(session: &mut SurfaceSession) {
    let _ = session.shutdown();
    let deadline = Instant::now() + Duration::from_secs(3);
    let _ = session.run_until(deadline, |_| false);
}

#[test]
fn prepare_draw_integration() {
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

    finish_session(&mut session);

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

    finish_session(&mut session);
}
