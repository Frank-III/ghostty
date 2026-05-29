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
    /// Events from the most recent [`tick`](Self::tick) for FFI polling.
    tick_events: Vec<AppEvent>,
}

impl App {
    pub fn new(mut config: AppConfig, mut runtime: RuntimeConfig) -> Self {
        if runtime.resources_dir.is_none() {
            runtime.resources_dir =
                std::env::var_os("GHOSTTY_RESOURCES_DIR").map(std::path::PathBuf::from);
        }
        config
            .config_mut()
            .finalize(runtime.resources_dir.as_deref());
        Self {
            config,
            runtime,
            surfaces: Vec::new(),
            next_surface_id: 1,
            focused: true,
            focused_surface: None,
            first: true,
            pending_events: Vec::new(),
            tick_events: Vec::new(),
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

    pub fn runtime_mut(&mut self) -> &mut RuntimeConfig {
        &mut self.runtime
    }

    /// Set the Ghostty resources directory (shell integration, theme search).
    pub fn set_resources_dir(&mut self, dir: std::path::PathBuf) {
        self.runtime.resources_dir = Some(dir);
        self.config
            .config_mut()
            .finalize(self.runtime.resources_dir.as_deref());
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn focused_surface(&self) -> Option<SurfaceId> {
        self.focused_surface
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

    fn push_surface_event(&mut self, id: SurfaceId, event: SurfaceEvent) {
        self.pending_events.push(AppEvent::Surface { id, event });
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
        if opts.resources_dir.is_none() {
            opts.resources_dir = self.runtime.resources_dir.clone();
        }
        let session = SurfaceSession::spawn(self.config.clone(), opts).ok()?;
        self.register_surface(id, Surface::with_session(id, session));
        Some(id)
    }

    /// Remove a surface by id, shutting down its session when present.
    pub fn delete_surface(&mut self, id: SurfaceId) -> bool {
        let Some(index) = self.surfaces.iter().position(|s| s.id() == id) else {
            return false;
        };

        #[cfg(all(unix, feature = "rust-vt"))]
        {
            if let Some(surface) = self.surfaces.get_mut(index) {
                let _ = surface.shutdown();
            }
        }

        self.surfaces.remove(index);

        if self.focused_surface == Some(id) {
            self.focused_surface = self.surfaces.first().map(|s| s.id());
            for surface in &mut self.surfaces {
                surface.set_focused(self.focused_surface == Some(surface.id()));
            }
            if let Some(new_focus) = self.focused_surface {
                self.push_surface_event(new_focus, SurfaceEvent::FocusChanged { focused: true });
            }
        }

        self.push_event(AppEvent::CloseSurface { id });
        true
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
                for event in surface.drain_session_events() {
                    events.push(AppEvent::Surface {
                        id: surface.id(),
                        event,
                    });
                }
                if let Some(exit_code) = surface.poll_child_exit() {
                    events.push(AppEvent::Surface {
                        id: surface.id(),
                        event: SurfaceEvent::ChildExited { exit_code },
                    });
                }
            }
        }

        self.tick_events = events.clone();
        if !events.is_empty() {
            if let Some(wakeup) = self.runtime.wakeup_cb {
                // SAFETY: embedder-provided callback; userdata valid for app lifetime.
                unsafe { wakeup(self.runtime.userdata) };
            }
        }
        events
    }

    /// Pop the next event produced by the last [`tick`](Self::tick).
    pub fn take_polled_event(&mut self) -> Option<AppEvent> {
        if self.tick_events.is_empty() {
            return None;
        }
        Some(self.tick_events.remove(0))
    }

    pub fn focus_surface(&mut self, id: SurfaceId) -> bool {
        if self.find_surface(id).is_none() {
            return false;
        }
        let previous = self.focused_surface;
        let mut focus_events = Vec::new();
        for surface in &mut self.surfaces {
            let now_focused = surface.id() == id;
            if surface.focused() != now_focused {
                surface.set_focused(now_focused);
                focus_events.push((
                    surface.id(),
                    SurfaceEvent::FocusChanged {
                        focused: now_focused,
                    },
                ));
            }
        }
        for (sid, event) in focus_events {
            self.push_surface_event(sid, event);
        }
        if previous != Some(id) {
            self.focused_surface = Some(id);
        }
        true
    }

    /// Update surface title and enqueue a title-changed event.
    pub fn set_surface_title(&mut self, id: SurfaceId, title: impl Into<String>) -> bool {
        let Some(surface) = self.find_surface_mut(id) else {
            return false;
        };
        let title = title.into();
        surface.set_title(title.clone());
        self.push_surface_event(id, SurfaceEvent::TitleChanged { title });
        true
    }

    /// Write bytes to a surface PTY (input path).
    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn write_surface(&mut self, id: SurfaceId, bytes: &[u8]) -> bool {
        self.find_surface_mut(id)
            .map(|s| s.write(bytes).is_ok())
            .unwrap_or(false)
    }

    /// Resize a surface grid.
    #[cfg(all(unix, feature = "rust-vt"))]
    pub fn resize_surface(&mut self, id: SurfaceId, cols: u16, rows: u16) -> bool {
        self.find_surface_mut(id)
            .map(|s| s.resize(cols, rows).is_ok())
            .unwrap_or(false)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        #[cfg(all(unix, feature = "rust-vt"))]
        {
            for surface in &mut self.surfaces {
                let _ = surface.shutdown();
            }
        }
        self.surfaces.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(unix, feature = "rust-vt"))]
    fn cleanup_app(app: &mut App) {
        let ids: Vec<_> = app.surfaces().iter().map(|s| s.id()).collect();
        for id in ids {
            let _ = app.delete_surface(id);
        }
    }

    #[cfg(all(unix, feature = "rust-vt"))]
    fn short_lived_surface(app: &mut App) -> SurfaceId {
        use crate::surface_session::SurfaceSessionOptions;
        use ghostty_termio::{CommandBuilder, CommandSpec};

        let spec: CommandSpec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("printf ok")
            .build()
            .expect("spec");
        app.create_surface_with_options(SurfaceSessionOptions {
            command: Some(spec),
            ..Default::default()
        })
        .expect("surface")
    }

    #[test]
    fn create_surface_and_find() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        assert!(app.is_first_surface());
        #[cfg(all(unix, feature = "rust-vt"))]
        let id = short_lived_surface(&mut app);
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let id = app.create_surface().unwrap();
        assert!(!app.is_first_surface());
        assert_eq!(app.surface_count(), 1);
        assert!(app.find_surface(id).is_some());
        #[cfg(all(unix, feature = "rust-vt"))]
        assert!(app.find_surface(id).unwrap().has_session());
        #[cfg(all(unix, feature = "rust-vt"))]
        cleanup_app(&mut app);
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
        #[cfg(all(unix, feature = "rust-vt"))]
        let id = short_lived_surface(&mut app);
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let id = app.create_surface().unwrap();
        assert_eq!(app.focused_surface(), Some(id));
        assert!(app.find_surface(id).unwrap().focused());
        #[cfg(all(unix, feature = "rust-vt"))]
        cleanup_app(&mut app);
    }

    #[test]
    fn multiple_surfaces_and_delete() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        #[cfg(all(unix, feature = "rust-vt"))]
        let a = short_lived_surface(&mut app);
        #[cfg(all(unix, feature = "rust-vt"))]
        let b = short_lived_surface(&mut app);
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let a = app.create_surface().unwrap();
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let b = app.create_surface().unwrap();
        assert_eq!(app.surface_count(), 2);
        app.focus_surface(b);
        assert_eq!(app.focused_surface(), Some(b));
        assert!(app.delete_surface(a));
        assert_eq!(app.surface_count(), 1);
        assert!(app.find_surface(a).is_none());
        assert_eq!(app.focused_surface(), Some(b));
        #[cfg(all(unix, feature = "rust-vt"))]
        cleanup_app(&mut app);
    }

    #[test]
    fn focus_emits_event_on_tick() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        #[cfg(all(unix, feature = "rust-vt"))]
        let a = short_lived_surface(&mut app);
        #[cfg(all(unix, feature = "rust-vt"))]
        let b = short_lived_surface(&mut app);
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let a = app.create_surface().unwrap();
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let b = app.create_surface().unwrap();
        app.focus_surface(b);
        let events = app.tick();
        assert!(events.iter().any(|e| {
            matches!(
                e,
                AppEvent::Surface {
                    id,
                    event: SurfaceEvent::FocusChanged { focused: true },
                } if *id == b
            )
        }));
        assert!(events.iter().any(|e| {
            matches!(
                e,
                AppEvent::Surface {
                    id,
                    event: SurfaceEvent::FocusChanged { focused: false },
                } if *id == a
            )
        }));
        #[cfg(all(unix, feature = "rust-vt"))]
        cleanup_app(&mut app);
    }

    #[test]
    fn set_surface_title_emits_event() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        #[cfg(all(unix, feature = "rust-vt"))]
        let id = short_lived_surface(&mut app);
        #[cfg(not(all(unix, feature = "rust-vt")))]
        let id = app.create_surface().unwrap();
        assert!(app.set_surface_title(id, "term"));
        let events = app.tick();
        assert!(events.iter().any(|e| {
            matches!(
                e,
                AppEvent::Surface {
                    event: SurfaceEvent::TitleChanged { title },
                    ..
                } if title == "term"
            )
        }));
        #[cfg(all(unix, feature = "rust-vt"))]
        cleanup_app(&mut app);
    }
}
