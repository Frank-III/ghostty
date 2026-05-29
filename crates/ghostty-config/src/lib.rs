//! Configuration schema and loader for the Ghostty Rust port (Phase 2).
//!
//! Port targets:
//! - `src/config/Config.zig` (subset)
//! - `src/config/file_load.zig`
//! - `src/config/string.zig`
//! - `src/config/path.zig`, `io.zig` (value types)
//! - `src/cli/args.zig` (`LineIterator` for config files)

pub mod config;
pub mod derived_config;
pub mod error;
pub mod file_load;
pub mod parse;
pub mod string_literal;
pub mod types;

pub use config::{Config, OptionalFileAction};
pub use derived_config::{
    DerivedCoreConfig, DerivedFontConfig, DerivedRendererConfig, DerivedTermioConfig,
};
pub use error::{
    ConfigError, Diagnostic, DiagnosticList, LoadError, SourceLocation,
};
pub use file_load::{
    default_xdg_path, legacy_default_xdg_path, preferred_default_file_path,
    validate_config_path,
};
#[cfg(target_os = "macos")]
pub use file_load::{default_app_support_path, legacy_default_app_support_path};
pub use parse::{strip_utf8_bom, ConfigLine, LineIter};
pub use string_literal::parse as parse_string_literal;
pub use types::{ConfigPath, CursorStyle, MouseShiftCapture, ReadableIo, RgbColor, WindowPadding};
pub use ghostty_foundation::{FoundationError, FoundationResult};
