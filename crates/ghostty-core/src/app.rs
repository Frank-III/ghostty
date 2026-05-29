//! Application core (`App.zig`, `apprt/embedded.zig` `App`).
//!
//! With feature `rust-vt`, `create_surface` spawns a real [`SurfaceSession`] per surface.
//! `tick` drains pending app events and pumps all surface sessions.

use crate::{AppConfig, AppEvent, RuntimeConfig, Surface, SurfaceEvent, SurfaceId};

#[cfg(all(unix, feature = "rust-vt"))]
use crate::surface_session::{SurfaceSession, SurfaceSessionOptions};

/// Primary GUI application state.
#[derive(Debug)]
pub struct App {
    config: AppConfig,
    runtime: RuntimeConfig,
    surfaces: Vec<Surface>,
    next_surface_id: u64,
    focused: bool,
    focused_surface: Option<SurfaceId>,
    /// Mirrors `App.first` — true until the first surface is created.
    first: bool,
    pending_events: Vec<AppEvent>,
}

impl App {
    pub fn new(config: AppConfig, runtime: RuntimeConfig) -> Self {
        Self {
            config,
            runtime,
            surfaces: Vec::new(),
            next_surface_id: 1,
            focused: true,
            focused_surface: None,
            first: true,
            pending_events: Vec::new(),
        }
    }

    pub fn with_defaults(runtime: RuntimeConfig) -> Self {
        Self::new(AppConfig::default(), runtime)
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn runtime(&self) -> &RuntimeConfig {
        &self.runtime
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_first_surface(&self) -> bool {
        self.first
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn surfaces_mut(&mut self) -> &mut [Surface] {
        &mut self.surfaces
    }

    pub fn find_surface(&self, id: SurfaceId) -> Option<&Surface> {
        self.surfaces.iter().find(|s| s.id() == id)
    }

    pub fn find_surface_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces.iter_mut().find(|s| s.id() == id)
    }

    fn allocate_surface_id(&mut self) -> Option<SurfaceId> {
        let id = SurfaceId::from_raw(self.next_surface_id)?;
        self.next_surface_id = self.next_surface_id.saturating_add(1);
        if self.next_surface_id == 0 {
            self.next_surface_id = 1;
        }
        Some(id)
    }

    fn register_surface(&mut self, id: SurfaceId, surface: Surface) {
        self.surfaces.push(surface);
        if self.focused_surface.is_none() {
            self.focused_surface = Some(id);
            if let Some(s) = self.find_surface_mut(id) {
                s.set_focused(true);
            }
        }
        self.first = false;
    }

    /// Allocate a surface ID and register a surface.
    ///
    /// With `rust-vt`, spawns config + termio + Rust VT via [`SurfaceSession`].
    pub fn create_surface(&mut self) -> Option<SurfaceId> {
        #[cfg(all(unix, feature = "rust-vt"))]
        {
            return self.create_surface_with_options(SurfaceSessionOptions::default());
        }
        #[cfg(not(all(unix, feature = "rust-vt")))]
        {
            let id = self.allocate_surface_id()?;
            self.register_surface(id, Surface::new(id));
            Some(id)
        }
    }

    /// Like [`create_surface`](Self::create_surface) with explicit session options.
    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn create_surface_with_options(
        &mut self,
        mut opts: SurfaceSessionOptions,
    ) -> Option<SurfaceId> {
        let id = opts.id.take().or_else(|| self.allocate_surface_id())?;
        opts.id = Some(id);
        let session = SurfaceSession::spawn(self.config.clone(), opts).ok()?;
        self.register_surface(id, Surface::with_session(id, session));
        Some(id)
    }

    pub fn push_event(&mut self, event: AppEvent) {
        self.pending_events.push(event);
    }

    /// Drain pending app events and tick all surface sessions.
    pub fn tick(&mut self) -> Vec<AppEvent> {
        let mut events = core::mem::take(&mut self.pending_events);

        #[cfg(all(unix, feature = "rust-vt"))]
        {
            for surface in &mut self.surfaces {
                let _ = surface.tick();
                if let Some(exit_code) = surface.poll_child_exit() {
                    events.push(AppEvent::Surface {
                        id: surface.id(),
                        event: SurfaceEvent::ChildExited { exit_code },
                    });
                }
            }
        }

        events
    }

    pub fn focus_surface(&mut self, id: SurfaceId) -> bool {
        if self.find_surface(id).is_none() {
            return false;
        }
        for surface in &mut self.surfaces {
            surface.set_focused(surface.id() == id);
        }
        self.focused_surface = Some(id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_surface_and_find() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        assert!(app.is_first_surface());
        let id = app.create_surface().unwrap();
        assert!(!app.is_first_surface());
        assert_eq!(app.surface_count(), 1);
        assert!(app.find_surface(id).is_some());
        #[cfg(all(unix, feature = "rust-vt"))]
        assert!(app.find_surface(id).unwrap().has_session());
    }

    #[test]
    fn tick_drains_events() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        app.push_event(AppEvent::Quit);
        let events = app.tick();
        assert_eq!(events, vec![AppEvent::Quit]);
        assert!(app.tick().is_empty());
    }

    #[test]
    fn create_surface_focuses_first() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        let id = app.create_surface().unwrap();
        assert!(app.find_surface(id).unwrap().focused());
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    mod session {
        use std::time::{Duration, Instant};

        use ghostty_termio::{CommandBuilder, CommandSpec};

        use super::*;
        use crate::surface_session::SurfaceSessionOptions;

        fn printf_spec(text: &str) -> CommandSpec {
            CommandBuilder::new()
                .path("/bin/sh")
                .arg("sh")
                .arg("-c")
                .arg(format!("printf '{text}'"))
                .build()
                .expect("spec")
        }

        #[test]
        fn create_surface_spawns_pty_session() {
            let mut app = App::with_defaults(RuntimeConfig::default());
            let id = app.create_surface().unwrap();
            let surface = app.find_surface(id).unwrap();
            assert!(surface.has_session());
            assert!(surface.session().unwrap().pid() > 0);
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
                    return;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
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
                    return;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            panic!("expected ChildExited event");
        }
    }
}
