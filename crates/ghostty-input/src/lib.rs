//! Keyboard and mouse input for the Ghostty Rust port (Phase 3).
//!
//! Port targets: `src/input/`.

pub mod key;
pub mod key_mods;
pub mod kitty_sequence;
pub mod kitty_table;

pub use key::{Action, CsiUMods, Key, KeyEvent, ctrl_seq};
pub use key_mods::{Mod, ModKeys, ModSide, ModSides, Mods, OptionAsAlt, ctrl_or_super};
pub use kitty_sequence::{KittyEvent, KittyMods, KittySequence};
pub use kitty_table::KittyTableEntry;
pub use ghostty_foundation::{FoundationError, FoundationResult};
