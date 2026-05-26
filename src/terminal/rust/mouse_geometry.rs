use crate::constants::*;
use crate::input::*;
use crate::mouse_types::*;

pub(crate) use crate::mouse_pixels::*;

pub(crate) fn mouse_pos_out_of_viewport(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> bool {
    pos.x < 0.0
        || pos.y < 0.0
        || pos.x > size.screen_width as f32
        || pos.y > size.screen_height as f32
}

pub(crate) fn mouse_grid_size(size: GhosttyMouseSize) -> GhosttyMouseCell {
    let terminal_width = size
        .screen_width
        .saturating_sub(size.padding_left.saturating_add(size.padding_right));
    let terminal_height = size
        .screen_height
        .saturating_sub(size.padding_top.saturating_add(size.padding_bottom));
    let columns = nonzero_u32_div(terminal_width, size.cell_width).max(1);
    let rows = nonzero_u32_div(terminal_height, size.cell_height).max(1);

    GhosttyMouseCell {
        x: columns.min(u32::from(u16::MAX)) as u16,
        y: rows,
    }
}

pub(crate) fn mouse_pos_to_cell(pos: GhosttyMousePosition, size: GhosttyMouseSize) -> GhosttyMouseCell {
    let grid = mouse_grid_size(size);
    let term_x = (pos.x - size.padding_left as f32).max(0.0);
    let term_y = (pos.y - size.padding_top as f32).max(0.0);
    let col = (term_x / size.cell_width as f32) as u32;
    let row = (term_y / size.cell_height as f32) as u32;

    GhosttyMouseCell {
        x: col.min(u32::from(grid.x.saturating_sub(1))) as u16,
        y: row.min(grid.y.saturating_sub(1)),
    }
}

pub(crate) fn nonzero_u32_div(numerator: u32, denominator: u32) -> u32 {
    numerator.checked_div(denominator).unwrap_or(0)
}
