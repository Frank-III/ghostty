//! Font discovery skeleton. Port target: `src/font/discovery.zig`.

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
}

/// Platform-neutral discovery API.
pub trait Discover {
    fn list_matching(&self, descriptor: &Descriptor) -> Result<Vec<DiscoveredFont>, DiscoveryError>;
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

/// Backends with no discovery (plain FreeType / wasm).
#[derive(Debug, Clone, Copy)]
pub struct NoDiscovery;

impl Discover for NoDiscovery {
    fn list_matching(&self, _descriptor: &Descriptor) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
        Err(DiscoveryError::NoDiscovery)
    }
}

/// Placeholder when the target OS module is not compiled in.
#[derive(Debug, Clone, Copy)]
struct StubDiscovery(Backend);

impl Discover for StubDiscovery {
    fn list_matching(&self, _descriptor: &Descriptor) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
        let _ = self.0;
        Err(DiscoveryError::BackendNotImplemented)
    }
}

pub mod platform {
    #[cfg(target_os = "macos")]
    pub mod coretext {
        use super::super::*;

        /// CoreText discovery stub (`src/font/discovery.zig` CoreText path).
        #[derive(Debug, Clone, Copy, Default)]
        pub struct CoreTextDiscovery;

        impl Discover for CoreTextDiscovery {
            fn list_matching(
                &self,
                _descriptor: &Descriptor,
            ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
                Err(DiscoveryError::BackendNotImplemented)
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

        #[derive(Debug, Clone, Copy, Default)]
        pub struct FontconfigDiscovery;

        impl Discover for FontconfigDiscovery {
            fn list_matching(
                &self,
                _descriptor: &Descriptor,
            ) -> Result<Vec<DiscoveredFont>, DiscoveryError> {
                Err(DiscoveryError::BackendNotImplemented)
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
        let mut desc = Descriptor::default();
        desc.family = Some("JetBrains Mono".to_string());
        desc.monospace = true;
        desc.size = 12.0;
        assert_eq!(desc.family.as_deref(), Some("JetBrains Mono"));
        let discover = discover_for_backend(Backend::default_for_target());
        let _ = discover.list_matching(&desc);
    }
}
