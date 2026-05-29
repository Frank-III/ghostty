use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::kitty_geometry::*;
use crate::mouse_encode::*;
use crate::render::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use core::ffi::c_int;
use core::ptr;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_render_info(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    x_offset: u32,
    y_offset: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    viewport_col: i32,
    viewport_row: i32,
    viewport_visible: bool,
    out_pixel_width: *mut u32,
    out_pixel_height: *mut u32,
    out_grid_cols: *mut u32,
    out_grid_rows: *mut u32,
    out_viewport_col: *mut i32,
    out_viewport_row: *mut i32,
    out_viewport_visible: *mut bool,
    out_source_x: *mut u32,
    out_source_y: *mut u32,
    out_source_width: *mut u32,
    out_source_height: *mut u32,
) -> c_int {
    unsafe {
        kitty_render_info_impl(
            image_width,
            image_height,
            source_x,
            source_y,
            source_width,
            source_height,
            placement_columns,
            placement_rows,
            x_offset,
            y_offset,
            terminal_width_px,
            terminal_height_px,
            terminal_cols,
            terminal_rows,
            viewport_col,
            viewport_row,
            viewport_visible,
            out_pixel_width,
            out_pixel_height,
            out_grid_cols,
            out_grid_rows,
            out_viewport_col,
            out_viewport_row,
            out_viewport_visible,
            out_source_x,
            out_source_y,
            out_source_width,
            out_source_height,
        )
    }
}

pub(crate) unsafe fn kitty_render_info_impl(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    x_offset: u32,
    y_offset: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    viewport_col: i32,
    viewport_row: i32,
    viewport_visible: bool,
    out_pixel_width: *mut u32,
    out_pixel_height: *mut u32,
    out_grid_cols: *mut u32,
    out_grid_rows: *mut u32,
    out_viewport_col: *mut i32,
    out_viewport_row: *mut i32,
    out_viewport_visible: *mut bool,
    out_source_x: *mut u32,
    out_source_y: *mut u32,
    out_source_width: *mut u32,
    out_source_height: *mut u32,
) -> c_int {
    if out_pixel_width.is_null()
        || out_pixel_height.is_null()
        || out_grid_cols.is_null()
        || out_grid_rows.is_null()
        || out_viewport_col.is_null()
        || out_viewport_row.is_null()
        || out_viewport_visible.is_null()
        || out_source_x.is_null()
        || out_source_y.is_null()
        || out_source_width.is_null()
        || out_source_height.is_null()
    {
        return GHOSTTY_INVALID_VALUE;
    }

    let (pixel_width, pixel_height) = kitty_pixel_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );
    let (grid_cols, grid_rows) = kitty_grid_size(
        image_width,
        image_height,
        source_width,
        source_height,
        placement_columns,
        placement_rows,
        x_offset,
        y_offset,
        terminal_width_px,
        terminal_height_px,
        terminal_cols,
        terminal_rows,
    );
    let (source_rect_x, source_rect_y, source_rect_width, source_rect_height) = kitty_source_rect(
        image_width,
        image_height,
        source_x,
        source_y,
        source_width,
        source_height,
    );

    unsafe {
        ptr::write(out_pixel_width, pixel_width);
        ptr::write(out_pixel_height, pixel_height);
        ptr::write(out_grid_cols, grid_cols);
        ptr::write(out_grid_rows, grid_rows);
        ptr::write(out_viewport_col, viewport_col);
        ptr::write(out_viewport_row, viewport_row);
        ptr::write(out_viewport_visible, viewport_visible);
        ptr::write(out_source_x, source_rect_x);
        ptr::write(out_source_y, source_rect_y);
        ptr::write(out_source_width, source_rect_width);
        ptr::write(out_source_height, source_rect_height);
    }

    GHOSTTY_SUCCESS
}
