//! Surface identity (`Surface.zig` `id` field).
//!
//! Non-zero `u64` exposed to embedders (e.g. `GHOSTTY_SURFACE_ID` env, GTK DBus).

use core::num::NonZeroU64;

/// Opaque surface identifier. Zero is reserved (DBus / null sentinel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(NonZeroU64);

impl SurfaceId {
    /// Returns `None` if `raw` is zero.
    pub const fn from_raw(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl From<SurfaceId> for u64 {
    fn from(id: SurfaceId) -> u64 {
        id.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero() {
        assert!(SurfaceId::from_raw(0).is_none());
    }

    #[test]
    fn round_trip() {
        let id = SurfaceId::from_raw(42).unwrap();
        assert_eq!(id.get(), 42);
    }
}
