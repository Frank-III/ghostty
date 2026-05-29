//! Derived configuration slices for core, termio, font, and renderer.
//!
//! Port target: `DerivedConfig` helpers in `src/config/Config.zig`.

use crate::types::{CursorStyle, RgbColor, ShellIntegration};
use crate::Config;

/// App-level fields (shell integration, theme path).
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedAppConfig {
    pub shell_integration: ShellIntegration,
    pub theme: Option<std::path::PathBuf>,
}

/// Surface/core session fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedCoreConfig {
    pub scrollback_limit: usize,
    pub focus_follows_mouse: bool,
    pub selection_clear_on_copy: bool,
    pub selection_clear_on_typing: bool,
}

/// Termio/exec fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedTermioConfig {
    pub command: Option<String>,
    pub wait_after_command: bool,
    pub abnormal_command_exit_runtime: u32,
    pub env: Vec<(String, String)>,
    pub term: String,
    pub working_directory: Option<std::path::PathBuf>,
}

/// Font discovery/sizing fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedFontConfig {
    pub font_size: f32,
    pub font_family: Option<String>,
    pub font_thicken: bool,
    pub font_thicken_strength: u8,
    pub grapheme_width_method: crate::types::GraphemeWidthMethod,
}

/// Renderer color/cursor fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedRendererConfig {
    pub background: RgbColor,
    pub foreground: RgbColor,
    pub background_opacity: f64,
    pub cursor_color: Option<RgbColor>,
    pub cursor_style: CursorStyle,
    pub cursor_opacity: f64,
    pub minimum_contrast: f64,
}

impl From<&Config> for DerivedCoreConfig {
    fn from(cfg: &Config) -> Self {
        Self {
            scrollback_limit: cfg.scrollback_limit,
            focus_follows_mouse: cfg.focus_follows_mouse,
            selection_clear_on_copy: cfg.selection_clear_on_copy,
            selection_clear_on_typing: cfg.selection_clear_on_typing,
        }
    }
}

impl From<&Config> for DerivedTermioConfig {
    fn from(cfg: &Config) -> Self {
        let working_directory = cfg.working_directory.as_deref().map(crate::expand_path);
        Self {
            command: cfg.command.clone(),
            wait_after_command: cfg.wait_after_command,
            abnormal_command_exit_runtime: cfg.abnormal_command_exit_runtime,
            env: cfg.env.clone(),
            term: cfg.term.clone(),
            working_directory,
        }
    }
}

impl From<&Config> for DerivedFontConfig {
    fn from(cfg: &Config) -> Self {
        Self {
            font_size: cfg.font_size,
            font_family: cfg.font_family.clone(),
            font_thicken: cfg.font_thicken,
            font_thicken_strength: cfg.font_thicken_strength,
            grapheme_width_method: cfg.grapheme_width_method,
        }
    }
}

impl From<&Config> for DerivedAppConfig {
    fn from(cfg: &Config) -> Self {
        let theme = cfg.theme.as_deref().map(crate::expand_path);
        Self {
            shell_integration: cfg.shell_integration,
            theme,
        }
    }
}

impl From<&Config> for DerivedRendererConfig {
    fn from(cfg: &Config) -> Self {
        Self {
            background: cfg.background,
            foreground: cfg.foreground,
            background_opacity: cfg.background_opacity,
            cursor_color: cfg.cursor_color,
            cursor_style: cfg.cursor_style,
            cursor_opacity: cfg.cursor_opacity,
            minimum_contrast: cfg.minimum_contrast,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_slices_from_defaults() {
        let cfg = Config::with_defaults();
        let core: DerivedCoreConfig = (&cfg).into();
        assert_eq!(core.scrollback_limit, cfg.scrollback_limit);
        let termio: DerivedTermioConfig = (&cfg).into();
        assert_eq!(termio.term, cfg.term);
        let font: DerivedFontConfig = (&cfg).into();
        assert_eq!(font.font_size, cfg.font_size);
        let renderer: DerivedRendererConfig = (&cfg).into();
        assert_eq!(renderer.background, cfg.background);
        let app: DerivedAppConfig = (&cfg).into();
        assert_eq!(app.shell_integration, cfg.shell_integration);
    }
}
