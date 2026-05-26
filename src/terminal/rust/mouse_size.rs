use crate::mouse_types::*;

pub(crate) fn mouse_size_from_parts(
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
) -> GhosttyMouseSize {
    GhosttyMouseSize {
        screen_width,
        screen_height,
        cell_width,
        cell_height,
        padding_top,
        padding_bottom,
        padding_right,
        padding_left,
    }
}

pub(crate) fn mouse_size_has_cell_size(size: GhosttyMouseSize) -> bool {
    size.cell_width != 0 && size.cell_height != 0
}
