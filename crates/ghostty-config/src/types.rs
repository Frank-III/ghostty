//! Shared config value types (`src/config/path.zig`, `io.zig`, padding helpers).

use crate::error::ConfigError;
use crate::string_literal;

/// RGB color (`src/config/Config.zig` `Color` subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn parse_cli(input: &str) -> Result<Self, ConfigError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ConfigError::InvalidValue);
        }
        if let Some(hex) = input.strip_prefix('#') {
            return parse_hex(hex);
        }
        match input.to_ascii_lowercase().as_str() {
            "black" => Ok(Self { r: 0, g: 0, b: 0 }),
            "white" => Ok(Self {
                r: 0xff,
                g: 0xff,
                b: 0xff,
            }),
            "red" => Ok(Self {
                r: 0xff,
                g: 0,
                b: 0,
            }),
            "green" => Ok(Self {
                r: 0,
                g: 0x80,
                b: 0,
            }),
            "blue" => Ok(Self {
                r: 0,
                g: 0,
                b: 0xff,
            }),
            "yellow" => Ok(Self {
                r: 0xff,
                g: 0xff,
                b: 0,
            }),
            "cyan" | "aqua" => Ok(Self {
                r: 0,
                g: 0xff,
                b: 0xff,
            }),
            "magenta" | "fuchsia" => Ok(Self {
                r: 0xff,
                g: 0,
                b: 0xff,
            }),
            "gray" | "grey" => Ok(Self {
                r: 0x80,
                g: 0x80,
                b: 0x80,
            }),
            "silver" => Ok(Self {
                r: 0xc0,
                g: 0xc0,
                b: 0xc0,
            }),
            "maroon" => Ok(Self {
                r: 0x80,
                g: 0,
                b: 0,
            }),
            "olive" => Ok(Self {
                r: 0x80,
                g: 0x80,
                b: 0,
            }),
            "navy" => Ok(Self {
                r: 0,
                g: 0,
                b: 0x80,
            }),
            "purple" => Ok(Self {
                r: 0x80,
                g: 0,
                b: 0x80,
            }),
            "teal" => Ok(Self {
                r: 0,
                g: 0x80,
                b: 0x80,
            }),
            "orange" => Ok(Self {
                r: 0xff,
                g: 0xa5,
                b: 0,
            }),
            "pink" => Ok(Self {
                r: 0xff,
                g: 0xc0,
                b: 0xcb,
            }),
            "brown" => Ok(Self {
                r: 0xa5,
                g: 0x2a,
                b: 0x2a,
            }),
            "lime" => Ok(Self {
                r: 0,
                g: 0xff,
                b: 0,
            }),
            _ => Err(ConfigError::InvalidValue),
        }
    }
}

fn parse_hex(hex: &str) -> Result<RgbColor, ConfigError> {
    let hex = hex.trim();
    if hex.len() != 6 {
        return Err(ConfigError::InvalidValue);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ConfigError::InvalidValue)?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ConfigError::InvalidValue)?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ConfigError::InvalidValue)?;
    Ok(RgbColor { r, g, b })
}

/// Config path with optional `?` prefix (`src/config/path.zig`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigPath {
    Optional(String),
    Required(String),
}

impl ConfigPath {
    pub fn parse(input: Option<&str>) -> Result<Option<Self>, ConfigError> {
        let mut value = input.ok_or(ConfigError::ValueRequired)?;
        if value.is_empty() {
            return Ok(None);
        }

        let optional = if let Some(rest) = value.strip_prefix('?') {
            value = rest;
            true
        } else {
            false
        };

        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = &value[1..value.len() - 1];
        }

        let mut parsed = String::new();
        string_literal::parse(&mut parsed, value)?;
        if optional {
            Ok(Some(Self::Optional(parsed)))
        } else {
            Ok(Some(Self::Required(parsed)))
        }
    }
}

/// Readable IO source: raw string or deferred path (`src/config/io.zig`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadableIo {
    Raw(String),
    Path(String),
}

impl ReadableIo {
    pub fn parse_cli(input: Option<&str>) -> Result<Self, ConfigError> {
        let input = input.ok_or(ConfigError::ValueRequired)?;
        if input.is_empty() {
            return Err(ConfigError::ValueRequired);
        }

        {
            let mut buf = String::new();
            string_literal::parse(&mut buf, input)?;
        }

        if let Some(rest) = input.strip_prefix("raw:") {
            let mut parsed = String::new();
            string_literal::parse(&mut parsed, rest)?;
            return Ok(Self::Raw(parsed));
        }
        if let Some(rest) = input.strip_prefix("path:") {
            let mut parsed = String::new();
            string_literal::parse(&mut parsed, rest)?;
            return Ok(Self::Path(parsed));
        }

        let mut parsed = String::new();
        string_literal::parse(&mut parsed, input)?;
        Ok(Self::Raw(parsed))
    }
}

/// Window padding pair (`Config.WindowPadding`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowPadding {
    pub top_left: u32,
    pub bottom_right: u32,
}

impl WindowPadding {
    pub fn parse_cli(input: Option<&str>) -> Result<Self, ConfigError> {
        let input = input.ok_or(ConfigError::ValueRequired)?;
        let input = input.trim_matches(|c: char| c == ' ' || c == '\t');
        if let Some(idx) = input.find(',') {
            let left = input[..idx].trim();
            let right = input[idx + 1..].trim();
            let top_left = left.parse().map_err(|_| ConfigError::InvalidValue)?;
            let bottom_right = right.parse().map_err(|_| ConfigError::InvalidValue)?;
            Ok(Self {
                top_left,
                bottom_right,
            })
        } else {
            let value: u32 = input.parse().map_err(|_| ConfigError::InvalidValue)?;
            Ok(Self {
                top_left: value,
                bottom_right: value,
            })
        }
    }
}

/// Cursor shape (`Config.CursorStyle` in Zig).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Bar,
    Underline,
}

impl CursorStyle {
    pub fn parse_cli(input: &str) -> Result<Self, ConfigError> {
        match input.trim() {
            "block" => Ok(Self::Block),
            "bar" => Ok(Self::Bar),
            "underline" => Ok(Self::Underline),
            _ => Err(ConfigError::InvalidValue),
        }
    }
}

/// Mouse shift capture mode (`Config.MouseShiftCapture` in Zig).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseShiftCapture {
    False,
    True,
    Always,
    Never,
}

impl MouseShiftCapture {
    pub fn parse_cli(input: &str) -> Result<Self, ConfigError> {
        match input.trim() {
            "false" => Ok(Self::False),
            "true" => Ok(Self::True),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(ConfigError::InvalidValue),
        }
    }
}

/// Link preview mode (`Config.LinkPreviews` in Zig).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkPreviews {
    True,
    False,
    Osc8,
}

impl LinkPreviews {
    pub fn parse_cli(input: &str) -> Result<Self, ConfigError> {
        match input.trim() {
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "osc8" => Ok(Self::Osc8),
            _ => Err(ConfigError::InvalidValue),
        }
    }
}

/// Grapheme width method (`Config.GraphemeWidthMethod` in Zig).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphemeWidthMethod {
    Unicode,
    Legacy,
}

impl GraphemeWidthMethod {
    pub fn parse_cli(input: &str) -> Result<Self, ConfigError> {
        match input.trim() {
            "unicode" => Ok(Self::Unicode),
            "legacy" => Ok(Self::Legacy),
            _ => Err(ConfigError::InvalidValue),
        }
    }
}

/// Window background blur (`Config.BackgroundBlur` subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundBlur {
    False,
    True,
}

impl BackgroundBlur {
    pub fn parse_cli(input: &str) -> Result<Self, ConfigError> {
        match input.trim() {
            "false" => Ok(Self::False),
            "true" => Ok(Self::True),
            _ => Err(ConfigError::InvalidValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_color_hex_and_named() {
        assert_eq!(
            RgbColor::parse_cli("#AABBCC").unwrap(),
            RgbColor {
                r: 0xaa,
                g: 0xbb,
                b: 0xcc,
            }
        );
        assert_eq!(
            RgbColor::parse_cli("black").unwrap(),
            RgbColor { r: 0, g: 0, b: 0 }
        );
    }

    #[test]
    fn window_padding_scalar() {
        let v = WindowPadding::parse_cli(Some("100")).unwrap();
        assert_eq!(v.top_left, 100);
        assert_eq!(v.bottom_right, 100);
    }

    #[test]
    fn window_padding_pair() {
        let v = WindowPadding::parse_cli(Some("100,200")).unwrap();
        assert_eq!(v.top_left, 100);
        assert_eq!(v.bottom_right, 200);
    }

    #[test]
    fn readable_io_raw_and_path() {
        assert_eq!(
            ReadableIo::parse_cli(Some("foo")).unwrap(),
            ReadableIo::Raw("foo".into())
        );
        assert_eq!(
            ReadableIo::parse_cli(Some("raw:foo")).unwrap(),
            ReadableIo::Raw("foo".into())
        );
        assert_eq!(
            ReadableIo::parse_cli(Some("path:foo")).unwrap(),
            ReadableIo::Path("foo".into())
        );
    }

    #[test]
    fn config_path_optional() {
        let p = ConfigPath::parse(Some("?/tmp/x")).unwrap().unwrap();
        assert_eq!(p, ConfigPath::Optional("/tmp/x".into()));
    }
}
