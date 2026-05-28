//! Platform renderer backend selection.
//!
//! Port target: `src/renderer/backend.zig`.

/// GPU backend implementation (`renderer.Backend`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    OpenGl,
    Metal,
    WebGl,
}

impl Backend {
    /// Default backend for a host OS family (WASM always WebGL).
    pub fn default_for_os(is_darwin: bool, is_wasm: bool) -> Self {
        if is_wasm {
            Backend::WebGl
        } else if is_darwin {
            Backend::Metal
        } else {
            Backend::OpenGl
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_zig() {
        assert_eq!(Backend::default_for_os(true, false), Backend::Metal);
        assert_eq!(Backend::default_for_os(false, false), Backend::OpenGl);
        assert_eq!(Backend::default_for_os(false, true), Backend::WebGl);
    }
}
