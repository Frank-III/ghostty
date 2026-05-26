use core::ffi::c_int;

use crate::constants::*;

#[no_mangle]
pub extern "C" fn ghostty_rust_render_row_cell_get(
    has_cells: bool,
    has_cell: bool,
    data: c_int,
    has_out: bool,
) -> c_int {
    render_row_cell_get_impl(has_cells, has_cell, data, has_out)
}

pub(crate) fn render_row_cell_get_impl(
    has_cells: bool,
    has_cell: bool,
    data: c_int,
    has_out: bool,
) -> c_int {
    if !has_cells || !has_cell || !has_out {
        return RENDER_RESULT_INVALID_VALUE;
    }

    match data {
        RENDER_ROW_CELL_DATA_RAW
        | RENDER_ROW_CELL_DATA_STYLE
        | RENDER_ROW_CELL_DATA_GRAPHEMES_LEN
        | RENDER_ROW_CELL_DATA_GRAPHEMES_BUF
        | RENDER_ROW_CELL_DATA_BG_COLOR
        | RENDER_ROW_CELL_DATA_FG_COLOR
        | RENDER_ROW_CELL_DATA_SELECTED => RENDER_RESULT_SUCCESS,
        _ => RENDER_RESULT_INVALID_VALUE,
    }
}
