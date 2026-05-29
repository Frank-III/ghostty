//! Unicode helpers ported from `src/unicode/`.

pub mod grapheme;
mod props_table;
pub mod symbols;
mod symbols_table;
pub mod width;

pub use grapheme::{grapheme_break, grapheme_break_no_control, BreakState, GraphemeBreak};
pub use props_table::{get as props_get, Properties};
pub use symbols::is_symbol;
pub use width::{codepoint_width, width_zero_in_grapheme};
