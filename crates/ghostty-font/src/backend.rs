//! Font backend selection. Port target: `src/font/backend.zig`.

/// Compile-time / runtime font stack backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Backend {
    Freetype,
    FreetypeWindows,
    FontconfigFreetype,
    Coretext,
    CoretextFreetype,
    CoretextHarfbuzz,
    CoretextNoshape,
    WebCanvas,
}

impl Backend {
    /// Default backend for the current target (mirrors Zig `Backend.default`).
    pub const fn default_for_target() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            return Self::WebCanvas;
        }
        #[cfg(target_os = "windows")]
        {
            return Self::FreetypeWindows;
        }
        #[cfg(target_os = "macos")]
        {
            return Self::Coretext;
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "windows", target_os = "macos")))]
        {
            Self::FontconfigFreetype
        }
    }

    pub const fn has_freetype(self) -> bool {
        matches!(
            self,
            Self::Freetype
                | Self::FreetypeWindows
                | Self::FontconfigFreetype
                | Self::CoretextFreetype
        )
    }

    pub const fn has_fontconfig(self) -> bool {
        matches!(self, Self::FontconfigFreetype)
    }

    pub const fn has_coretext(self) -> bool {
        matches!(
            self,
            Self::Coretext
                | Self::CoretextFreetype
                | Self::CoretextHarfbuzz
                | Self::CoretextNoshape
        )
    }

    pub const fn has_discovery(self) -> bool {
        matches!(
            self,
            Self::FreetypeWindows
                | Self::FontconfigFreetype
                | Self::Coretext
                | Self::CoretextFreetype
                | Self::CoretextHarfbuzz
                | Self::CoretextNoshape
        )
    }
}
