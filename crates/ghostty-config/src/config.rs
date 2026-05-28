//! Core configuration (`src/config/Config.zig` minimal subset).

use std::path::Path;

use crate::error::{ConfigError, DiagnosticList, LoadError};
use crate::file_load;
use crate::parse::{strip_utf8_bom, LineIter};
use crate::types::{RgbColor, WindowPadding};

/// Subset of Ghostty config fields used across the stack (full schema deferred).
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub font_size: f32,
    pub abnormal_command_exit_runtime: u32,
    pub window_padding_x: WindowPadding,
    pub window_padding_y: WindowPadding,
    pub window_inherit_font_size: bool,
    pub language: Option<String>,
    pub background: RgbColor,
    pub foreground: RgbColor,
    pub background_opacity: f64,
    pub cursor_color: Option<RgbColor>,
    pub font_family: Option<String>,
    pub scrollback_limit: usize,
    pub minimum_contrast: f64,
    diagnostics: DiagnosticList,
}

impl Default for Config {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl Config {
    /// Built-in defaults (`Config.default` in Zig).
    pub fn with_defaults() -> Self {
        Self {
            font_size: default_font_size(),
            abnormal_command_exit_runtime: 250,
            window_padding_x: WindowPadding {
                top_left: 2,
                bottom_right: 2,
            },
            window_padding_y: WindowPadding {
                top_left: 2,
                bottom_right: 2,
            },
            window_inherit_font_size: true,
            language: None,
            background: default_background(),
            foreground: default_foreground(),
            background_opacity: 1.0,
            cursor_color: None,
            font_family: None,
            scrollback_limit: 10_000_000,
            minimum_contrast: 1.0,
            diagnostics: DiagnosticList::new(),
        }
    }

    pub fn diagnostics(&self) -> &DiagnosticList {
        &self.diagnostics
    }

    pub fn diagnostics_mut(&mut self) -> &mut DiagnosticList {
        &mut self.diagnostics
    }

    /// Parse config from an in-memory file body (`loadReader`).
    pub fn load_from_str(&mut self, content: &str, source_path: &str) {
        let content = strip_utf8_bom(content);
        for line in LineIter::new(content) {
            let loc = Some(line.location(source_path));
            if let Err(err) = self.apply_line(&line.key, line.value.as_deref()) {
                let message = match err {
                    ConfigError::InvalidValue => "invalid value",
                    ConfigError::ValueRequired => "value required",
                    _ => "invalid field",
                };
                if matches!(err, ConfigError::InvalidField) {
                    self.diagnostics.invalid_field(line.key, loc);
                } else {
                    self.diagnostics.invalid_value(line.key, message, loc);
                }
            }
        }
    }

    /// Load absolute config file path (`loadFile`).
    pub fn load_file(&mut self, path: &Path) -> Result<(), LoadError> {
        file_load::validate_config_path(path)?;
        let content =
            std::fs::read_to_string(path).map_err(|_| LoadError::FileOpenFailed)?;
        let path_str = path.to_string_lossy();
        self.load_from_str(&content, &path_str);
        Ok(())
    }

    /// Load optional file; returns whether it was loaded (`loadOptionalFile`).
    pub fn load_optional_file(&mut self, path: &Path) -> OptionalFileAction {
        match self.load_file(path) {
            Ok(()) => OptionalFileAction::Loaded,
            Err(LoadError::FileNotFound) => OptionalFileAction::NotFound,
            Err(_) => OptionalFileAction::Error,
        }
    }

    /// Load default user config files (`loadDefaultFiles` subset).
    pub fn load_default_files(&mut self) {
        let legacy = file_load::legacy_default_xdg_path();
        let xdg = file_load::default_xdg_path();
        let _ = self.load_optional_file(&legacy);
        let _ = self.load_optional_file(&xdg);

        #[cfg(target_os = "macos")]
        {
            let legacy_app = file_load::legacy_default_app_support_path();
            let app = file_load::default_app_support_path();
            let _ = self.load_optional_file(&legacy_app);
            let _ = self.load_optional_file(&app);
        }
    }

    fn apply_line(&mut self, key: &str, value: Option<&str>) -> Result<(), ConfigError> {
        match key {
            "font-size" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.font_size = v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "abnormal-command-exit-runtime" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.abnormal_command_exit_runtime =
                    v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "window-padding-x" => {
                self.window_padding_x = WindowPadding::parse_cli(value)?;
            }
            "window-padding-y" => {
                self.window_padding_y = WindowPadding::parse_cli(value)?;
            }
            "window-inherit-font-size" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.window_inherit_font_size =
                    parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "language" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                let mut parsed = String::new();
                crate::string_literal::parse(&mut parsed, v)?;
                self.language = Some(parsed);
            }
            "background" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.background = RgbColor::parse_cli(v)?;
            }
            "foreground" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.foreground = RgbColor::parse_cli(v)?;
            }
            "background-opacity" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.background_opacity = v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "cursor-color" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.cursor_color = Some(RgbColor::parse_cli(v)?);
            }
            "font-family" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                let mut parsed = String::new();
                crate::string_literal::parse(&mut parsed, v)?;
                self.font_family = Some(parsed);
            }
            "scrollback-limit" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.scrollback_limit = v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "minimum-contrast" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                let parsed: f64 = v.parse().map_err(|_| ConfigError::InvalidValue)?;
                self.minimum_contrast = parsed.clamp(1.0, 21.0);
            }
            _ => return Err(ConfigError::InvalidField),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalFileAction {
    Loaded,
    NotFound,
    Error,
}

fn default_font_size() -> f32 {
    if cfg!(target_os = "macos") {
        13.0
    } else {
        12.0
    }
}

fn default_background() -> RgbColor {
    RgbColor {
        r: 0x28,
        g: 0x2c,
        b: 0x34,
    }
}

fn default_foreground() -> RgbColor {
    RgbColor {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    }
}

fn parse_bool(s: &str) -> Result<bool, ()> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_scrollback_and_contrast() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "scrollback-limit = 5000000\nminimum-contrast = 42\n",
            "/tmp/config.ghostty",
        );
        assert!(cfg.diagnostics().is_empty());
        assert_eq!(cfg.scrollback_limit, 5_000_000);
        assert_eq!(cfg.minimum_contrast, 21.0);
    }

    #[test]
    fn parse_extended_keys() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "background-opacity = 0.75\ncursor-color = #AABBCC\nfont-family = \"JetBrains Mono\"\n",
            "/tmp/config.ghostty",
        );
        assert!(cfg.diagnostics().is_empty());
        assert_eq!(cfg.background_opacity, 0.75);
        assert_eq!(
            cfg.cursor_color,
            Some(RgbColor {
                r: 0xaa,
                g: 0xbb,
                b: 0xcc,
            })
        );
        assert_eq!(cfg.font_family.as_deref(), Some("JetBrains Mono"));
    }

    #[test]
    fn parse_colors() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "background = #112233\nforeground = white\n",
            "/tmp/config.ghostty",
        );
        assert!(cfg.diagnostics().is_empty());
        assert_eq!(cfg.background, RgbColor { r: 0x11, g: 0x22, b: 0x33 });
        assert_eq!(cfg.foreground, RgbColor { r: 0xff, g: 0xff, b: 0xff });
    }

    #[test]
    fn defaults_match_zig() {
        let cfg = Config::with_defaults();
        assert_eq!(cfg.abnormal_command_exit_runtime, 250);
        assert_eq!(cfg.window_padding_x.top_left, 2);
        assert!(cfg.window_inherit_font_size);
        assert_eq!(cfg.background, default_background());
        assert_eq!(cfg.foreground, default_foreground());
        assert_eq!(cfg.scrollback_limit, 10_000_000);
        assert_eq!(cfg.minimum_contrast, 1.0);
        if cfg!(target_os = "macos") {
            assert_eq!(cfg.font_size, 13.0);
        } else {
            assert_eq!(cfg.font_size, 12.0);
        }
    }

    #[test]
    fn parse_snippet() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "abnormal-command-exit-runtime = 2500\nfont-size = 14\n",
            "/home/ghostty/.config/ghostty/config.ghostty",
        );
        assert!(cfg.diagnostics().is_empty());
        assert_eq!(cfg.abnormal_command_exit_runtime, 2500);
        assert_eq!(cfg.font_size, 14.0);
    }

    #[test]
    fn parse_bom() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "\u{feff}abnormal-command-exit-runtime = 2500\n",
            "/home/ghostty/.config/ghostty/config.ghostty",
        );
        assert!(cfg.diagnostics().is_empty());
        assert_eq!(cfg.abnormal_command_exit_runtime, 2500);
    }

    #[test]
    fn unknown_field_diagnostic() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str("not-a-real-key = 1\n", "/tmp/config.ghostty");
        assert_eq!(cfg.diagnostics().len(), 1);
        assert_eq!(
            cfg.diagnostics().items()[0].key.as_deref(),
            Some("not-a-real-key")
        );
    }

    #[test]
    fn load_temp_file() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "ghostty-config-test-{}.ghostty",
            std::process::id()
        ));
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "window-padding-x = 4,8").unwrap();
        }
        let mut cfg = Config::with_defaults();
        cfg.load_file(&path).unwrap();
        assert_eq!(cfg.window_padding_x.top_left, 4);
        assert_eq!(cfg.window_padding_x.bottom_right, 8);
        let _ = std::fs::remove_file(path);
    }
}
