use crate::mouse_types::*;

pub(crate) fn mouse_pos_to_pixels(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> GhosttyMousePixels {
    GhosttyMousePixels {
        x: round_f32_to_i32(pos.x - size.padding_left as f32),
        y: round_f32_to_i32(pos.y - size.padding_top as f32),
    }
}

pub(crate) fn round_f32_to_i32(value: f32) -> i32 {
    if value >= 0.0 {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    }
}
