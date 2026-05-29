//! App integration tests with Rust-owned VT (isolated from lib unit tests).

use std::time::{Duration, Instant};

use ghostty_core::{App, AppEvent, RuntimeConfig, SurfaceEvent, SurfaceSessionOptions};
use ghostty_termio::{CommandBuilder, CommandSpec};

fn printf_spec(text: &str) -> CommandSpec {
    CommandBuilder::new()
        .path("/bin/sh")
        .arg("sh")
        .arg("-c")
        .arg(format!("printf '{text}'"))
        .build()
        .expect("spec")
}

fn cleanup_app(app: &mut App) {
    let ids: Vec<_> = app.surfaces().iter().map(|s| s.id()).collect();
    for id in ids {
        let _ = app.delete_surface(id);
    }
}

#[test]
fn create_surface_spawns_pty_session() {
    let mut app = App::with_defaults(RuntimeConfig::default());
    let id = app
        .create_surface_with_options(SurfaceSessionOptions {
            command: Some(printf_spec("ok")),
            ..Default::default()
        })
        .expect("surface");
    let surface = app.find_surface(id).unwrap();
    assert!(surface.has_session());
    assert!(surface.session().unwrap().pid() > 0);
    cleanup_app(&mut app);
}

#[test]
fn tick_pumps_child_output_into_terminal() {
    let mut app = App::with_defaults(RuntimeConfig::default());
    let id = app
        .create_surface_with_options(SurfaceSessionOptions {
            command: Some(printf_spec("app-vt")),
            ..Default::default()
        })
        .expect("surface");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        app.tick();
        if app.find_surface(id).unwrap().contains_text("app-vt") {
            cleanup_app(&mut app);
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    cleanup_app(&mut app);
    panic!("expected app-vt in terminal grid");
}

#[test]
fn tick_reports_child_exit() {
    let mut app = App::with_defaults(RuntimeConfig::default());
    let id = app
        .create_surface_with_options(SurfaceSessionOptions {
            command: Some(printf_spec("done")),
            ..Default::default()
        })
        .expect("surface");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let events = app.tick();
        if events.iter().any(|e| {
            matches!(
                e,
                AppEvent::Surface {
                    id: sid,
                    event: SurfaceEvent::ChildExited { exit_code: 0 },
                } if *sid == id
            )
        }) {
            cleanup_app(&mut app);
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    cleanup_app(&mut app);
    panic!("expected ChildExited event");
}

#[test]
fn spawn_from_config_file_command() {
    let dir = std::env::temp_dir().join(format!("ghostty-app-cfg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cfg_path = dir.join("test.ghostty");
    std::fs::write(&cfg_path, "command = printf cfg-port\n").unwrap();

    let app_config = ghostty_core::AppConfig::from_config_file(&cfg_path).expect("load");
    let mut app = App::new(app_config, RuntimeConfig::default());
    let id = app.create_surface().expect("surface");

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        app.tick();
        if app.find_surface(id).unwrap().contains_text("cfg-port") {
            cleanup_app(&mut app);
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    cleanup_app(&mut app);
    let _ = std::fs::remove_dir_all(&dir);
    panic!("expected cfg-port output from config file command");
}
