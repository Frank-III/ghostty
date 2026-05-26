use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_geometry::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;

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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_rect(
    start_node: *mut c_void,
    start_x: u16,
    start_y: u16,
    end_node: *mut c_void,
    end_y: u16,
    grid_cols_minus_one: u32,
    terminal_cols_minus_one: u16,
    out: *mut GhosttySelection,
) -> c_int {
    if start_node.is_null() || end_node.is_null() || out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let end_x = u32::from(start_x)
        .wrapping_add(grid_cols_minus_one)
        .min(u32::from(terminal_cols_minus_one)) as u16;

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*out).size),
            mem::size_of::<GhosttySelection>(),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*out).start.size),
            mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(core::ptr::addr_of_mut!((*out).start.node), start_node);
        ptr::write(core::ptr::addr_of_mut!((*out).start.x), start_x);
        ptr::write(core::ptr::addr_of_mut!((*out).start.y), start_y);
        ptr::write(
            core::ptr::addr_of_mut!((*out).end.size),
            mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(core::ptr::addr_of_mut!((*out).end.node), end_node);
        ptr::write(core::ptr::addr_of_mut!((*out).end.x), end_x);
        ptr::write(core::ptr::addr_of_mut!((*out).end.y), end_y);
        ptr::write(core::ptr::addr_of_mut!((*out).rectangle), true);
    }

    GHOSTTY_SUCCESS
}
