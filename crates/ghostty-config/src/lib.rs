//! Configuration schema and loader for the Ghostty Rust port (Phase 2).
//!
//! Port targets:
//! - `src/config/Config.zig` (subset)
//! - `src/config/file_load.zig`
//! - `src/config/string.zig`
//! - `src/config/path.zig`, `io.zig` (value types)
//! - `src/cli/args.zig` (`LineIterator` for config files)

pub mod cli;
pub mod config;
pub mod derived_config;
pub mod error;
pub mod file_load;
pub mod parse;
pub mod path;
pub mod string_literal;
pub mod theme;
pub mod types;

pub use cli::CliArgs;
pub use config::{Config, OptionalFileAction};
pub use derived_config::{
    DerivedAppConfig, DerivedCoreConfig, DerivedFontConfig, DerivedRendererConfig,
    DerivedStreamConfig, DerivedTermioConfig,
};
pub use error::{ConfigError, Diagnostic, DiagnosticList, LoadError, SourceLocation};
#[cfg(target_os = "macos")]
pub use file_load::{default_app_support_path, legacy_default_app_support_path};
pub use file_load::{
    default_xdg_path, legacy_default_xdg_path, preferred_default_file_path, validate_config_path,
};
pub use ghostty_foundation::{FoundationError, FoundationResult};
pub use parse::{strip_utf8_bom, ConfigLine, LineIter};
pub use path::{expand_path, expand_tilde, resolve_relative};
pub use string_literal::parse as parse_string_literal;
pub use theme::{
    resolve_theme_path, theme_search_dirs, user_theme_dirs, user_themes_dir, ThemeResolveError,
};
pub use types::{
    ClipboardAccess, ConfigPath, CursorStyle, MouseShiftCapture, OscColorReportFormat,
    ReadableIo, RgbColor, ShellIntegration, WindowPadding,
};
