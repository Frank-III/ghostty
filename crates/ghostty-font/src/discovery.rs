//! Font discovery skeleton. Port target: `src/font/discovery.zig`.
//!
//! macOS/Linux use known system monospace paths until narrow CoreText/Fontconfig FFI
//! lands; see `platform_font_paths` and `select_primary`.

use crate::backend::Backend;
use crate::descriptor::Descriptor;

/// A font candidate returned by discovery (metadata only; face load deferred).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFont {
    pub family: String,
    pub style: Option<String>,
    pub path: Option<String>,
    pub face_index: i32,
}

/// Errors from the discovery layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError {
    /// This build backend does not perform font discovery.
    NoDiscovery,
    /// Platform FFI is not linked in the Rust port yet.
    BackendNotImplemented,
    /// No font matched the descriptor on this system.
    NotFound,
}

/// Platform-neutral discovery API.
pub trait Discover {
    fn list_matching(&self, descriptor: &Descriptor)
        -> Result<Vec<DiscoveredFont>, DiscoveryError>;
}

/// Selects a discovery implementation for the given backend.
pub fn discover_for_backend(backend: Backend) -> Box<dyn Discover + Send + Sync> {
    match backend {
        Backend::Freetype | Backend::WebCanvas => Box::new(NoDiscovery),
        #[cfg(target_os = "macos")]
        b if b.has_coretext() => Box::new(platform::coretext::CoreTextDiscovery),
        #[cfg(target_os = "windows")]
        Backend::FreetypeWindows => Box::new(platform::windows::WindowsDiscovery),
        #[cfg(all(unix, not(target_os = "macos")))]
        Backend::FontconfigFreetype => Box::new(platform::fontconfig::FontconfigDiscovery),
        _ => Box::new(StubDiscovery(backend)),
    }
}

/// Build a monospace search descriptor from config font fields.
pub fn descriptor_from_font_family(family: Option<&str>, size: f32) -> Descriptor {
    Descriptor {
        family: family.map(str::to_string),
        size,
        monospace: true,
        ..Descriptor::default()
    }
}

/// Discover fonts for a descriptor and return the best match (path preferred).
pub fn select_primary(descriptor: &Descriptor) -> Result<DiscoveredFont, DiscoveryError> {
    let backend = Backend::default_for_target();
    let discover = discover_for_backend(backend);
    let mut fonts = discover.list_matching(descriptor)?;
    if fonts.is_empty() {
        return Err(DiscoveryError::NotFound);
    }
    rank_fonts(&mut fonts, descriptor);
    fonts.into_iter().next().ok_or(DiscoveryError::NotFound)
}

fn rank_fonts(fonts: &mut [DiscoveredFont], descriptor: &Descriptor) {
    let want = descriptor.family.as_deref().map(str::to_ascii_lowercase);
    fonts.sort_by(|a, b| {
        let score_a = family_match_score(&a.family, want.as_deref());
        let score_b = family_match_score(&b.family, want.as_deref());
        let path_a = a.path.is_some();
        let path_b = b.path.is_some();
        score_b
            .cmp(&score_a)
            .then_with(|| path_b.cmp(&path_a))
            .then_with(|| a.family.cmp(&b.family))
    });
}

fn family_match_score(family: &str, want: Option<&str>) -> u8 {
    let Some(want) = want else {
        return 0;
    };
    let family = family.to_ascii_lowercase();
    if family == want {
        2
    } else if family.contains(want) || want.contains(&family) {
        1
    } else {
        0
    }
}

/// Backends with no discovery (plain FreeType / wasm).
#[derive(Debug, Clone, Copy)]
pub struct NoDiscovery;

impl Discover for NoDiscovery {
    fn list_matching(
        &self,
        _descriptor: &Descriptor,
    ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
        Err(DiscoveryError::NoDiscovery)
    }
}

/// Placeholder when the target OS module is not compiled in.
#[derive(Debug, Clone, Copy)]
struct StubDiscovery(Backend);

impl Discover for StubDiscovery {
    fn list_matching(
        &self,
        _descriptor: &Descriptor,
    ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
        let _ = self.0;
        Err(DiscoveryError::BackendNotImplemented)
    }
}

/// Known system monospace paths when platform FFI is not linked.
fn platform_font_paths(descriptor: &Descriptor) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
    let family = descriptor
        .family
        .as_deref()
        .unwrap_or("monospace")
        .to_string();
    let candidates: &[(&str, &str)] = &[
        #[cfg(target_os = "macos")]
        ("Menlo", "/System/Library/Fonts/Menlo.ttc"),
        #[cfg(target_os = "macos")]
        ("SF Mono", "/System/Library/Fonts/SFNSMono.ttf"),
        #[cfg(all(unix, not(target_os = "macos")))]
        (
            "DejaVu Sans Mono",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        ),
        #[cfg(all(unix, not(target_os = "macos")))]
        (
            "JetBrains Mono",
            "/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf",
        ),
        #[cfg(all(unix, not(target_os = "macos")))]
        (
            "JetBrains Mono",
            "/usr/share/fonts/truetype/jetbrains-mono/JetBrainsMono-Regular.ttf",
        ),
    ];
    let mut out = Vec::new();
    for (name, path) in candidates {
        if std::path::Path::new(path).is_file() {
            out.push(DiscoveredFont {
                family: name.to_string(),
                style: None,
                path: Some((*path).to_string()),
                face_index: 0,
            });
        }
    }
    if out.is_empty() {
        // Still return a metadata-only candidate so callers can proceed without a path.
        out.push(DiscoveredFont {
            family,
            style: None,
            path: None,
            face_index: 0,
        });
        return Ok(out);
    }
    Ok(out)
}

pub mod platform {
    #[cfg(target_os = "macos")]
    pub mod coretext {
        use super::super::*;
        use core_text::font;

        /// CoreText discovery via `CTFontCreateWithName`, with known-path fallback.
        #[derive(Debug, Clone, Copy, Default)]
        pub struct CoreTextDiscovery;

        fn ct_match(descriptor: &Descriptor) -> Option<DiscoveredFont> {
            let mut names: Vec<&str> = Vec::new();
            if let Some(family) = descriptor.family.as_deref() {
                names.push(family);
            }
            names.extend(["Menlo", "SF Mono", ".AppleSystemUIFontMonospaced-Regular"]);

            let size = f64::from(descriptor.size.max(1.0));
            for name in names {
                let Ok(font) = font::new_from_name(name, size) else {
                    continue;
                };
                let family = font.family_name();
                let path = font
                    .copy_descriptor()
                    .font_path()
                    .map(|p| p.to_string_lossy().into_owned());
                if path
                    .as_deref()
                    .is_some_and(|p| std::path::Path::new(p).is_file())
                {
                    return Some(DiscoveredFont {
                        family,
                        style: None,
                        path,
                        face_index: 0,
                    });
                }
                if path.is_some() || !family.is_empty() {
                    return Some(DiscoveredFont {
                        family,
                        style: None,
                        path,
                        face_index: 0,
                    });
                }
            }
            None
        }

        impl Discover for CoreTextDiscovery {
            fn list_matching(
                &self,
                descriptor: &Descriptor,
            ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
                if let Some(font) = ct_match(descriptor) {
                    return Ok(vec![font]);
                }
                platform_font_paths(descriptor)
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub mod windows {
        use super::super::*;

        #[derive(Debug, Clone, Copy, Default)]
        pub struct WindowsDiscovery;

        impl Discover for WindowsDiscovery {
            fn list_matching(
                &self,
                _descriptor: &Descriptor,
            ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
                Err(DiscoveryError::BackendNotImplemented)
            }
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub mod fontconfig {
        use super::super::*;
        use std::process::Command;

        /// Fontconfig discovery via `fc-match` when available, else known paths.
        #[derive(Debug, Clone, Copy, Default)]
        pub struct FontconfigDiscovery;

        fn fc_match(descriptor: &Descriptor) -> Option<DiscoveredFont> {
            let family = descriptor.family.as_deref().unwrap_or("monospace");
            let pattern = format!("{family}:spacing=mono");
            let output = Command::new("fc-match")
                .arg("-f")
                .arg("%{family}\n%{file}\n")
                .arg(&pattern)
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let text = String::from_utf8_lossy(&output.stdout);
            let mut lines = text.lines();
            let family_name = lines.next()?.trim();
            let path = lines.next()?.trim();
            if family_name.is_empty() {
                return None;
            }
            let path = if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            };
            Some(DiscoveredFont {
                family: family_name.to_string(),
                style: None,
                path,
                face_index: 0,
            })
        }

        impl Discover for FontconfigDiscovery {
            fn list_matching(
                &self,
                descriptor: &Descriptor,
            ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
                if let Some(font) = fc_match(descriptor) {
                    if font
                        .path
                        .as_deref()
                        .is_some_and(|p| std::path::Path::new(p).is_file())
                    {
                        return Ok(vec![font]);
                    }
                }
                platform_font_paths(descriptor)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freetype_has_no_discovery() {
        let d = discover_for_backend(Backend::Freetype);
        assert_eq!(
            d.list_matching(&Descriptor::default()).unwrap_err(),
            DiscoveryError::NoDiscovery
        );
    }

    #[test]
    fn default_backend_selects_discover() {
        let backend = Backend::default_for_target();
        let _discover = discover_for_backend(backend);
    }

    #[test]
    fn fixture_monospace_descriptor_round_trip() {
        let desc = descriptor_from_font_family(Some("JetBrains Mono"), 12.0);
        assert_eq!(desc.family.as_deref(), Some("JetBrains Mono"));
        assert!(desc.monospace);
        let discover = discover_for_backend(Backend::default_for_target());
        let _ = discover.list_matching(&desc);
    }

    #[test]
    fn select_primary_returns_font_on_platform() {
        let desc = descriptor_from_font_family(None, 13.0);
        let font = select_primary(&desc);
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            assert!(font.is_ok(), "{font:?}");
            let font = font.unwrap();
            assert!(!font.family.is_empty());
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let _ = font;
        }
    }

    #[test]
    fn rank_prefers_family_match() {
        let mut fonts = vec![
            DiscoveredFont {
                family: "Menlo".into(),
                style: None,
                path: Some("/a".into()),
                face_index: 0,
            },
            DiscoveredFont {
                family: "JetBrains Mono".into(),
                style: None,
                path: Some("/b".into()),
                face_index: 0,
            },
        ];
        let desc = descriptor_from_font_family(Some("JetBrains Mono"), 12.0);
        rank_fonts(&mut fonts, &desc);
        assert_eq!(fonts[0].family, "JetBrains Mono");
    }
}
