//! Application skeleton (`App.zig`, `apprt/embedded.zig` `App`).
//!
//! No mailbox drain, renderer, or surface lifecycle beyond ID allocation.

use crate::{AppConfig, AppEvent, RuntimeConfig, Surface, SurfaceId};

/// Primary GUI application state (bootstrap).
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

    pub fn find_surface(&self, id: SurfaceId) -> Option<&Surface> {
        self.surfaces.iter().find(|s| s.id() == id)
    }

    pub fn find_surface_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces.iter_mut().find(|s| s.id() == id)
    }

    /// Allocate a new surface ID and register a skeleton surface.
    pub fn create_surface(&mut self) -> Option<SurfaceId> {
        let id = SurfaceId::from_raw(self.next_surface_id)?;
        self.next_surface_id = self.next_surface_id.saturating_add(1);
        if self.next_surface_id == 0 {
            self.next_surface_id = 1;
        }
        self.surfaces.push(Surface::new(id));
        self.first = false;
        Some(id)
    }

    pub fn push_event(&mut self, event: AppEvent) {
        self.pending_events.push(event);
    }

    /// Drain pending app events (no runtime mailbox yet).
    pub fn tick(&mut self) -> Vec<AppEvent> {
        core::mem::take(&mut self.pending_events)
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
    }

    #[test]
    fn tick_drains_events() {
        let mut app = App::with_defaults(RuntimeConfig::default());
        app.push_event(AppEvent::Quit);
        let events = app.tick();
        assert_eq!(events, vec![AppEvent::Quit]);
        assert!(app.tick().is_empty());
    }
}
