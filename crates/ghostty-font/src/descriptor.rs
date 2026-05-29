//! Font search descriptor. Port target: `src/font/discovery.zig` (`Descriptor`).

use std::hash::{Hash, Hasher};

use crate::face::Variation;

/// Describes a font to search for. Only `family` is required for a useful query;
/// other fields filter when set.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Descriptor {
    pub family: Option<String>,
    pub style: Option<String>,
    pub codepoint: u32,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
    pub monospace: bool,
    pub variations: Vec<Variation>,
}

impl Descriptor {
    pub fn hash<H: Hasher>(&self, state: &mut H) {
        self.family.hash(state);
        self.style.hash(state);
        self.codepoint.hash(state);
        self.size.to_bits().hash(state);
        self.bold.hash(state);
        self.italic.hash(state);
        self.monospace.hash(state);
        self.variations.len().hash(state);
        for v in &self.variations {
            v.id.hash(state);
            (v.value as i64).hash(state);
        }
    }

    /// Stable hash for caches (not cryptographic).
    pub fn hashcode(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Deep copy (allocates strings and variation slice).
    pub fn clone_into(
        &self,
        family: Option<String>,
        style: Option<String>,
        variations: Vec<Variation>,
    ) -> Self {
        Self {
            family,
            style,
            codepoint: self.codepoint,
            size: self.size,
            bold: self.bold,
            italic: self.italic,
            monospace: self.monospace,
            variations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::face::VariationId;

    #[test]
    fn hashcode_stable_for_same_descriptor() {
        let a = Descriptor {
            family: Some("Fira Code".into()),
            bold: true,
            ..Default::default()
        };
        let b = a.clone();
        assert_eq!(a.hashcode(), b.hashcode());
    }

    #[test]
    fn hashcode_differs_when_family_changes() {
        let a = Descriptor {
            family: Some("A".into()),
            ..Default::default()
        };
        let b = Descriptor {
            family: Some("B".into()),
            ..Default::default()
        };
        assert_ne!(a.hashcode(), b.hashcode());
    }

    #[test]
    fn variations_affect_hashcode() {
        let mut a = Descriptor::default();
        let b = Descriptor::default();
        assert_eq!(a.hashcode(), b.hashcode());
        a.variations.push(Variation {
            id: VariationId::from_tag(b"wght"),
            value: 400.0,
        });
        assert_ne!(a.hashcode(), b.hashcode());
    }
}
