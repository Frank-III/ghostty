//! Core configuration (`src/config/Config.zig` minimal subset).

use std::collections::HashSet;
use std::path::Path;

use crate::error::{ConfigError, DiagnosticList, LoadError};
use crate::file_load;
use crate::parse::{strip_utf8_bom, LineIter};
use crate::types::{
    BackgroundBlur, CursorStyle, GraphemeWidthMethod, LinkPreviews, MouseShiftCapture, RgbColor,
    WindowPadding,
};

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
    pub mouse_hide_while_typing: bool,
    pub font_thicken: bool,
    pub font_thicken_strength: u8,
    pub term: String,
    pub enquiry_response: String,
    pub cursor_style: CursorStyle,
    pub cursor_opacity: f64,
    pub cursor_click_to_move: bool,
    pub mouse_shift_capture: MouseShiftCapture,
    pub mouse_reporting: bool,
    pub clipboard_paste_protection: bool,
    pub command: Option<String>,
    pub wait_after_command: bool,
    pub selection_clear_on_typing: bool,
    pub focus_follows_mouse: bool,
    pub selection_clear_on_copy: bool,
    pub background_opacity_cells: bool,
    pub link_url: bool,
    pub link_previews: LinkPreviews,
    pub palette_generate: bool,
    pub palette_harmonious: bool,
    pub unfocused_split_opacity: f64,
    pub background_image_opacity: f32,
    pub cursor_style_blink: Option<bool>,
    pub maximize: bool,
    pub grapheme_width_method: GraphemeWidthMethod,
    pub background_blur: BackgroundBlur,
    pub env: Vec<(String, String)>,
    pub working_directory: Option<String>,
    pub initial_window: bool,
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
            mouse_hide_while_typing: false,
            font_thicken: false,
            font_thicken_strength: 255,
            term: "xterm-ghostty".to_string(),
            enquiry_response: String::new(),
            cursor_style: CursorStyle::Block,
            cursor_opacity: 1.0,
            cursor_click_to_move: true,
            mouse_shift_capture: MouseShiftCapture::False,
            mouse_reporting: true,
            clipboard_paste_protection: true,
            command: None,
            wait_after_command: false,
            selection_clear_on_typing: true,
            focus_follows_mouse: false,
            selection_clear_on_copy: false,
            background_opacity_cells: false,
            link_url: true,
            link_previews: LinkPreviews::True,
            palette_generate: false,
            palette_harmonious: false,
            unfocused_split_opacity: 0.7,
            background_image_opacity: 1.0,
            cursor_style_blink: None,
            maximize: false,
            grapheme_width_method: GraphemeWidthMethod::Unicode,
            background_blur: BackgroundBlur::False,
            env: Vec::new(),
            working_directory: None,
            initial_window: true,
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
        let _ = self.load_from_str_collect_includes(content, source_path);
    }

    /// Parse lines and return `config-file` entries discovered in this file.
    pub fn load_from_str_collect_includes(
        &mut self,
        content: &str,
        source_path: &str,
    ) -> Vec<String> {
        let content = strip_utf8_bom(content);
        let mut includes = Vec::new();
        for line in LineIter::new(content) {
            let loc = Some(line.location(source_path));
            if line.key == "config-file" {
                if let Some(value) = line.value.as_deref() {
                    includes.push(value.trim().to_string());
                } else {
                    self.diagnostics
                        .invalid_value("config-file", "value required", loc);
                }
                continue;
            }
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
        includes
    }

    /// Load absolute config file path (`loadFile`).
    pub fn load_file(&mut self, path: &Path) -> Result<(), LoadError> {
        let mut visited = HashSet::new();
        file_load::load_recursive(self, path, &mut visited)
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
            "mouse-hide-while-typing" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.mouse_hide_while_typing =
                    parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "font-thicken" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.font_thicken = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "font-thicken-strength" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.font_thicken_strength = v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "term" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.term = v.to_string();
            }
            "enquiry-response" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.enquiry_response = v.to_string();
            }
            "cursor-style" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.cursor_style = CursorStyle::parse_cli(v)?;
            }
            "cursor-opacity" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.cursor_opacity = v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "cursor-click-to-move" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.cursor_click_to_move = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "mouse-shift-capture" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.mouse_shift_capture = MouseShiftCapture::parse_cli(v)?;
            }
            "mouse-reporting" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.mouse_reporting = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "clipboard-paste-protection" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.clipboard_paste_protection =
                    parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "command" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.command = if v.is_empty() {
                    None
                } else {
                    Some(v.to_string())
                };
            }
            "wait-after-command" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.wait_after_command = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "selection-clear-on-typing" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.selection_clear_on_typing =
                    parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "focus-follows-mouse" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.focus_follows_mouse = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "selection-clear-on-copy" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.selection_clear_on_copy =
                    parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "background-opacity-cells" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.background_opacity_cells =
                    parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "link-url" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.link_url = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "link-previews" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.link_previews = LinkPreviews::parse_cli(v)?;
            }
            "palette-generate" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.palette_generate = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "palette-harmonious" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.palette_harmonious = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "unfocused-split-opacity" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                let parsed: f64 = v.parse().map_err(|_| ConfigError::InvalidValue)?;
                self.unfocused_split_opacity = parsed.clamp(0.15, 1.0);
            }
            "background-image-opacity" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.background_image_opacity = v.parse().map_err(|_| ConfigError::InvalidValue)?;
            }
            "cursor-style-blink" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.cursor_style_blink =
                    Some(parse_bool(v).map_err(|_| ConfigError::InvalidValue)?);
            }
            "maximize" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.maximize = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
            }
            "grapheme-width-method" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.grapheme_width_method = GraphemeWidthMethod::parse_cli(v)?;
            }
            "background-blur" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.background_blur = BackgroundBlur::parse_cli(v)?;
            }
            "env" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                if let Some(eq) = v.find('=') {
                    let key = v[..eq].trim().to_string();
                    let val = v[eq + 1..].trim().to_string();
                    self.env.push((key, val));
                } else {
                    return Err(ConfigError::InvalidValue);
                }
            }
            "working-directory" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                let mut parsed = String::new();
                crate::string_literal::parse(&mut parsed, v)?;
                self.working_directory = Some(parsed);
            }
            "initial-window" => {
                let v = value.ok_or(ConfigError::ValueRequired)?;
                self.initial_window = parse_bool(v).map_err(|_| ConfigError::InvalidValue)?;
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
    match s.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_mouse_hide_while_typing() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str("mouse-hide-while-typing = true\n", "/tmp/config.ghostty");
        assert!(cfg.diagnostics().is_empty());
        assert!(cfg.mouse_hide_while_typing);
    }

    #[test]
    fn parse_font_thicken() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "font-thicken = true\nfont-thicken-strength = 128\n",
            "/tmp/config.ghostty",
        );
        assert!(cfg.diagnostics().is_empty());
        assert!(cfg.font_thicken);
        assert_eq!(cfg.font_thicken_strength, 128);
    }

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
        assert_eq!(
            cfg.background,
            RgbColor {
                r: 0x11,
                g: 0x22,
                b: 0x33
            }
        );
        assert_eq!(
            cfg.foreground,
            RgbColor {
                r: 0xff,
                g: 0xff,
                b: 0xff
            }
        );
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
        assert_eq!(cfg.term, "xterm-ghostty");
        assert_eq!(cfg.cursor_style, CursorStyle::Block);
        assert_eq!(cfg.cursor_opacity, 1.0);
        assert!(cfg.cursor_click_to_move);
        assert_eq!(cfg.mouse_shift_capture, MouseShiftCapture::False);
        assert!(cfg.mouse_reporting);
        assert!(cfg.clipboard_paste_protection);
        assert!(cfg.selection_clear_on_typing);
        assert!(!cfg.focus_follows_mouse);
        assert!(!cfg.wait_after_command);
        assert!(!cfg.selection_clear_on_copy);
        assert!(!cfg.background_opacity_cells);
        assert!(cfg.link_url);
        assert_eq!(cfg.link_previews, LinkPreviews::True);
        assert!(!cfg.palette_generate);
        assert!(!cfg.palette_harmonious);
        assert_eq!(cfg.unfocused_split_opacity, 0.7);
        assert_eq!(cfg.background_image_opacity, 1.0);
        assert_eq!(cfg.cursor_style_blink, None);
        assert!(!cfg.maximize);
        assert_eq!(cfg.grapheme_width_method, GraphemeWidthMethod::Unicode);
        assert_eq!(cfg.background_blur, BackgroundBlur::False);
        assert!(cfg.command.is_none());
        assert!(cfg.env.is_empty());
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
    fn parse_new_fields() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "term = xterm-256color\ncursor-style = bar\ncursor-opacity = 0.8\n\
             cursor-click-to-move = false\nmouse-shift-capture = always\n\
             mouse-reporting = false\nclipboard-paste-protection = false\n\
             selection-clear-on-typing = false\nfocus-follows-mouse = true\n\
             wait-after-command = true\ncommand = /bin/bash\n",
            "/tmp/config",
        );
        assert!(cfg.diagnostics().is_empty(), "{:?}", cfg.diagnostics());
        assert_eq!(cfg.term, "xterm-256color");
        assert_eq!(cfg.cursor_style, CursorStyle::Bar);
        assert_eq!(cfg.cursor_opacity, 0.8);
        assert!(!cfg.cursor_click_to_move);
        assert_eq!(cfg.mouse_shift_capture, MouseShiftCapture::Always);
        assert!(!cfg.mouse_reporting);
        assert!(!cfg.clipboard_paste_protection);
        assert!(!cfg.selection_clear_on_typing);
        assert!(cfg.focus_follows_mouse);
        assert!(cfg.wait_after_command);
        assert_eq!(cfg.command.as_deref(), Some("/bin/bash"));
    }

    #[test]
    fn parse_env_entries() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "env = EDITOR=nvim\nenv = TERM=xterm-256color\n",
            "/tmp/config",
        );
        assert!(cfg.diagnostics().is_empty(), "{:?}", cfg.diagnostics());
        assert_eq!(cfg.env.len(), 2);
        assert_eq!(cfg.env[0], ("EDITOR".to_string(), "nvim".to_string()));
        assert_eq!(
            cfg.env[1],
            ("TERM".to_string(), "xterm-256color".to_string())
        );
    }

    #[test]
    fn bool_parsing_case_insensitive() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "mouse-hide-while-typing = TRUE\nfont-thicken = False\n",
            "/tmp/config",
        );
        assert!(cfg.diagnostics().is_empty());
        assert!(cfg.mouse_hide_while_typing);
        assert!(!cfg.font_thicken);
    }

    #[test]
    fn named_colors_expanded() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str("background = navy\nforeground = orange\n", "/tmp/config");
        assert!(cfg.diagnostics().is_empty(), "{:?}", cfg.diagnostics());
        assert_eq!(
            cfg.background,
            RgbColor {
                r: 0,
                g: 0,
                b: 0x80
            }
        );
        assert_eq!(
            cfg.foreground,
            RgbColor {
                r: 0xff,
                g: 0xa5,
                b: 0
            }
        );
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
    fn parse_extended_slice_table() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str(
            "selection-clear-on-copy = true\nbackground-opacity-cells = true\nlink-url = false\n\
             link-previews = osc8\npalette-generate = true\npalette-harmonious = true\n\
             unfocused-split-opacity = 0.5\nbackground-image-opacity = 0.25\n\
             cursor-style-blink = false\nmaximize = true\ngrapheme-width-method = legacy\n\
             background-blur = true\n",
            "/tmp/config",
        );
        assert!(cfg.diagnostics().is_empty(), "{:?}", cfg.diagnostics());
        assert!(cfg.selection_clear_on_copy);
        assert!(cfg.background_opacity_cells);
        assert!(!cfg.link_url);
        assert_eq!(cfg.link_previews, LinkPreviews::Osc8);
        assert!(cfg.palette_generate);
        assert!(cfg.palette_harmonious);
        assert_eq!(cfg.unfocused_split_opacity, 0.5);
        assert_eq!(cfg.background_image_opacity, 0.25);
        assert_eq!(cfg.cursor_style_blink, Some(false));
        assert!(cfg.maximize);
        assert_eq!(cfg.grapheme_width_method, GraphemeWidthMethod::Legacy);
        assert_eq!(cfg.background_blur, BackgroundBlur::True);
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

    #[test]
    fn recursive_config_file_include() {
        let dir = std::env::temp_dir().join(format!("ghostty-recursive-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let child = dir.join("child.ghostty");
        let parent = dir.join("parent.ghostty");
        std::fs::write(&child, "font-size = 22\n").unwrap();
        std::fs::write(
            &parent,
            format!("config-file = \"{}\"\nfont-size = 11\n", child.display()),
        )
        .unwrap();
        let mut cfg = Config::with_defaults();
        cfg.load_file(&parent).unwrap();
        assert_eq!(cfg.font_size, 22.0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_working_directory() {
        let mut cfg = Config::with_defaults();
        cfg.load_from_str("working-directory = /var/tmp\n", "/tmp/config");
        assert_eq!(cfg.working_directory.as_deref(), Some("/var/tmp"));
        assert!(cfg.diagnostics().is_empty());
    }
}
