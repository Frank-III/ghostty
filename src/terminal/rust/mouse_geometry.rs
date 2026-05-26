use crate::constants::*;
use crate::input::*;
use crate::mouse_types::*;

pub(crate) use crate::mouse_cell::*;
pub(crate) use crate::mouse_pixels::*;

pub(crate) fn mouse_pos_out_of_viewport(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> bool {
    pos.x < 0.0
        || pos.y < 0.0
        || pos.x > size.screen_width as f32
        || pos.y > size.screen_height as f32
}
