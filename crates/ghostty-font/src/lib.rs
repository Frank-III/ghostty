//! Font discovery, metrics, and face types for the Ghostty Rust port (Phase 4).
//!
//! Port targets:
//! - `src/font/main.zig`
//! - `src/font/face.zig`
//! - `src/font/Metrics.zig`
//! - `src/font/discovery.zig`
//! - `src/font/backend.zig`
//!
//! Platform face loading (CoreText, FreeType, web canvas) and shaping/atlas
//! remain Zig-owned until FFI is wired.

pub mod backend;
pub mod descriptor;
pub mod discovery;
pub mod face;
pub mod metrics;
pub mod style;

pub use backend::Backend;
pub use descriptor::Descriptor;
pub use discovery::{discover_for_backend, Discover, DiscoveredFont, DiscoveryError};
pub use face::{
    default_dpi, DesiredSize, GlyphSize, Options, RenderOptions, Variation, VariationId,
};
pub use metrics::{calc, FaceMetrics, Key, Metrics, Modifier, ModifierSet, ParseError};
pub use style::{Presentation, Style};
