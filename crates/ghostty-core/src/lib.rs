//! Application core for the Ghostty Rust port (Phase 6).
//!
//! Port targets:
//! - `src/App.zig`
//! - `src/Surface.zig`
//! - `src/apprt/embedded.zig`

mod app;
mod app_config;
mod events;
mod runtime;
mod surface;
mod surface_id;

pub use app::App;
pub use app_config::AppConfig;
pub use events::{ActionTag, ActionTargetTag, AppEvent, SurfaceEvent};
pub use runtime::RuntimeConfig;
pub use surface::Surface;
pub use surface_id::SurfaceId;
