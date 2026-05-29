//! Keyboard and mouse input for the Ghostty Rust port (Phase 3).
//!
//! Port targets: `src/input/`.

pub mod key;
pub mod key_mods;
pub mod kitty_sequence;
pub mod kitty_table;

pub use ghostty_foundation::{FoundationError, FoundationResult};
pub use key::{ctrl_seq, Action, CsiUMods, Key, KeyEvent};
pub use key_mods::{ctrl_or_super, Mod, ModKeys, ModSide, ModSides, Mods, OptionAsAlt};
pub use kitty_sequence::{KittyEvent, KittyMods, KittySequence};
pub use kitty_table::KittyTableEntry;
