use core::ffi::c_int;
use core::ptr;

use crate::early::*;
use crate::kitty_geometry::*;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_source_rect(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    out_x: *mut u32,
    out_y: *mut u32,
    out_width: *mut u32,
    out_height: *mut u32,
) -> c_int {
    unsafe {
        kitty_source_rect_impl(
            image_width,
            image_height,
            source_x,
            source_y,
            source_width,
            source_height,
            out_x,
            out_y,
            out_width,
            out_height,
        )
    }
}

pub(crate) unsafe fn kitty_source_rect_impl(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    out_x: *mut u32,
    out_y: *mut u32,
    out_width: *mut u32,
    out_height: *mut u32,
) -> c_int {
    if out_x.is_null() || out_y.is_null() || out_width.is_null() || out_height.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (x, y, width, height) = kitty_source_rect(
        image_width,
        image_height,
        source_x,
        source_y,
        source_width,
        source_height,
    );

    unsafe {
        ptr::write(out_x, x);
        ptr::write(out_y, y);
        ptr::write(out_width, width);
        ptr::write(out_height, height);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_pixel_size(
    image_width: u32,
    image_height: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    out_width: *mut u32,
    out_height: *mut u32,
) -> c_int {
    unsafe {
        kitty_pixel_size_impl(
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
            out_width,
            out_height,
        )
    }
}

pub(crate) unsafe fn kitty_pixel_size_impl(
    image_width: u32,
    image_height: u32,
    source_width: u32,
    source_height: u32,
    placement_columns: u32,
    placement_rows: u32,
    terminal_width_px: u32,
    terminal_height_px: u32,
    terminal_cols: u16,
    terminal_rows: u16,
    out_width: *mut u32,
    out_height: *mut u32,
) -> c_int {
    if out_width.is_null() || out_height.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (width, height) = kitty_pixel_size(
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

    unsafe {
        ptr::write(out_width, width);
        ptr::write(out_height, height);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_grid_size(
    image_width: u32,
    image_height: u32,
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
    out_cols: *mut u32,
    out_rows: *mut u32,
) -> c_int {
    unsafe {
        kitty_grid_size_impl(
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
            out_cols,
            out_rows,
        )
    }
}

pub(crate) unsafe fn kitty_grid_size_impl(
    image_width: u32,
    image_height: u32,
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
    out_cols: *mut u32,
    out_rows: *mut u32,
) -> c_int {
    if out_cols.is_null() || out_rows.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let (cols, rows) = kitty_grid_size(
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

    unsafe {
        ptr::write(out_cols, cols);
        ptr::write(out_rows, rows);
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_viewport_pos(
    pin_screen_x: i32,
    pin_screen_y: i32,
    viewport_screen_y: i32,
    grid_rows: i32,
    terminal_rows: u16,
    out_col: *mut i32,
    out_row: *mut i32,
    out_visible: *mut bool,
) -> c_int {
    unsafe {
        kitty_viewport_pos_impl(
            pin_screen_x,
            pin_screen_y,
            viewport_screen_y,
            grid_rows,
            terminal_rows,
            out_col,
            out_row,
            out_visible,
        )
    }
}

pub(crate) unsafe fn kitty_viewport_pos_impl(
    pin_screen_x: i32,
    pin_screen_y: i32,
    viewport_screen_y: i32,
    grid_rows: i32,
    terminal_rows: u16,
    out_col: *mut i32,
    out_row: *mut i32,
    out_visible: *mut bool,
) -> c_int {
    if out_col.is_null() || out_row.is_null() || out_visible.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let row = pin_screen_y.wrapping_sub(viewport_screen_y);
    let terminal_rows_i32 = i32::from(terminal_rows);
    let visible = row.wrapping_add(grid_rows) > 0 && row < terminal_rows_i32;

    unsafe {
        ptr::write(out_col, pin_screen_x);
        ptr::write(out_row, row);
        ptr::write(out_visible, visible);
    }

    GHOSTTY_SUCCESS
}
