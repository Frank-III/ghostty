//! Map terminal color targets to `ghostty_action_color_kind_e` values.

use crate::kitty_color::{Kind, Special};
use crate::osc_parser_color::{ColorTarget, DynamicColor};

/// `ghostty_action_color_kind_e`: palette index (0+) or negative dynamic kinds.
pub(crate) fn color_target_kind(target: ColorTarget) -> Option<i32> {
    match target {
        ColorTarget::Palette(i) => Some(i as i32),
        ColorTarget::Dynamic(DynamicColor::Foreground) => Some(-1),
        ColorTarget::Dynamic(DynamicColor::Background) => Some(-2),
        ColorTarget::Dynamic(DynamicColor::Cursor) => Some(-3),
        ColorTarget::Dynamic(_) | ColorTarget::Special(_) => None,
    }
}

pub(crate) fn kitty_kind_to_action(kind: Kind) -> Option<i32> {
    match kind {
        Kind::Palette(i) => Some(i as i32),
        Kind::Special(Special::Foreground) => Some(-1),
        Kind::Special(Special::Background) => Some(-2),
        Kind::Special(Special::Cursor) => Some(-3),
        Kind::Special(_) => None,
    }
}
