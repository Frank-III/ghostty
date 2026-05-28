//! Keyboard and mouse input for the Ghostty Rust port (Phase 3).
//!
//! Port targets: `src/input/`.

pub mod key_mods;
pub mod kitty_sequence;

pub use key_mods::{Mod, ModKeys, ModSide, ModSides, Mods, OptionAsAlt};
pub use kitty_sequence::{KittyEvent, KittyMods, KittySequence};
pub use ghostty_foundation::{FoundationError, FoundationResult};
