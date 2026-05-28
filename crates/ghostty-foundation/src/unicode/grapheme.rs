//! Grapheme break logic ported from `src/unicode/grapheme.zig`.

include!("grapheme_table.rs");

use super::props_table;

/// Grapheme break class without control characters (uucode `GraphemeBreakNoControl`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphemeBreak {
    Other = 0,
    Prepend = 1,
    RegionalIndicator = 2,
    SpacingMark = 3,
    L = 4,
    V = 5,
    T = 6,
    Lv = 7,
    Lvt = 8,
    Zwj = 9,
    Zwnj = 10,
    ExtendedPictographic = 11,
    EmojiModifierBase = 12,
    EmojiModifier = 13,
    IndicConjunctBreakExtend = 14,
    IndicConjunctBreakLinker = 15,
    IndicConjunctBreakConsonant = 16,
}

impl GraphemeBreak {
    #[allow(dead_code)]
    const fn from_u5(value: u8) -> Self {
        match value {
            0 => Self::Other,
            1 => Self::Prepend,
            2 => Self::RegionalIndicator,
            3 => Self::SpacingMark,
            4 => Self::L,
            5 => Self::V,
            6 => Self::T,
            7 => Self::Lv,
            8 => Self::Lvt,
            9 => Self::Zwj,
            10 => Self::Zwnj,
            11 => Self::ExtendedPictographic,
            12 => Self::EmojiModifierBase,
            13 => Self::EmojiModifier,
            14 => Self::IndicConjunctBreakExtend,
            15 => Self::IndicConjunctBreakLinker,
            16 => Self::IndicConjunctBreakConsonant,
            _ => Self::Other,
        }
    }
}

/// Stateful grapheme break machine (uucode `BreakState`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BreakState {
    #[default]
    Default = 0,
    RegionalIndicator = 1,
    ExtendedPictographic = 2,
    IndicConjunctBreakConsonant = 3,
    IndicConjunctBreakLinker = 4,
}

impl BreakState {
    const fn from_u3(value: u8) -> Self {
        match value {
            0 => Self::Default,
            1 => Self::RegionalIndicator,
            2 => Self::ExtendedPictographic,
            3 => Self::IndicConjunctBreakConsonant,
            4 => Self::IndicConjunctBreakLinker,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TableValue {
    result: bool,
    state: BreakState,
}

fn decode_table_value(encoded: u8) -> TableValue {
    TableValue {
        result: encoded & 1 != 0,
        state: BreakState::from_u3((encoded >> 1) & 0x7),
    }
}

const fn key_index(state: BreakState, gb1: GraphemeBreak, gb2: GraphemeBreak) -> usize {
    (state as usize) | ((gb1 as usize) << 3) | ((gb2 as usize) << 8)
}

/// Returns whether there is a grapheme break between two codepoints.
///
/// Control characters must be filtered before calling this, matching Ghostty's Zig API.
pub fn grapheme_break(cp1: u32, cp2: u32, state: &mut BreakState) -> bool {
    let gb1 = props_table::get(cp1).grapheme_break;
    let gb2 = props_table::get(cp2).grapheme_break;
    grapheme_break_no_control(gb1, gb2, state)
}

/// Returns whether there is a grapheme break between two grapheme break classes.
///
/// This mirrors `unicode.graphemeBreak` once codepoints have been mapped to
/// grapheme break properties.
pub fn grapheme_break_no_control(
    gb1: GraphemeBreak,
    gb2: GraphemeBreak,
    state: &mut BreakState,
) -> bool {
    let value = decode_table_value(GRAPHEME_TABLE[key_index(*state, gb1, gb2)]);
    *state = value.state;
    value.result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoint_break_emoji_modifier() {
        let mut state = BreakState::Default;
        assert!(!grapheme_break(0x261D, 0x1F3FF, &mut state));

        let mut state = BreakState::Default;
        assert!(grapheme_break(0x22, 0x1F3FF, &mut state));
    }

    #[test]
    fn emoji_modifier_sequence() {
        let mut state = BreakState::Default;
        assert!(!grapheme_break_no_control(
            GraphemeBreak::EmojiModifierBase,
            GraphemeBreak::EmojiModifier,
            &mut state,
        ));

        let mut state = BreakState::Default;
        assert!(grapheme_break_no_control(
            GraphemeBreak::Other,
            GraphemeBreak::EmojiModifier,
            &mut state,
        ));
    }

    #[test]
    fn family_zwj_sequence() {
        let mut state = BreakState::Default;
        let sequence = [
            (GraphemeBreak::ExtendedPictographic, false),
            (GraphemeBreak::Zwj, false),
            (GraphemeBreak::ExtendedPictographic, false),
            (GraphemeBreak::Zwj, false),
            (GraphemeBreak::ExtendedPictographic, false),
            (GraphemeBreak::Zwj, false),
            (GraphemeBreak::ExtendedPictographic, false),
            (GraphemeBreak::Other, true),
        ];

        let mut gb1 = sequence[0].0;
        for &(gb2, expect_break) in &sequence[1..] {
            assert_eq!(
                grapheme_break_no_control(gb1, gb2, &mut state),
                expect_break,
                "gb1={gb1:?} gb2={gb2:?}",
            );
            gb1 = gb2;
        }
    }
}
