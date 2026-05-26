use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::style::*;

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

pub(crate) fn kitty_pixel_size(
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
) -> (u32, u32) {
    let width = if source_width > 0 {
        source_width
    } else {
        image_width
    };
    let height = if source_height > 0 {
        source_height
    } else {
        image_height
    };

    if placement_columns == 0 && placement_rows == 0 {
        return (width, height);
    }

    let cell_width = nonzero_u32_div(terminal_width_px, u32::from(terminal_cols));
    let cell_height = nonzero_u32_div(terminal_height_px, u32::from(terminal_rows));

    if placement_columns > 0 && placement_rows > 0 {
        return (
            cell_width.wrapping_mul(placement_columns),
            cell_height.wrapping_mul(placement_rows),
        );
    }

    if placement_columns > 0 {
        let calc_width = cell_width.wrapping_mul(placement_columns);
        let aspect = (height as f64) / (width as f64);
        return (calc_width, round_f64_to_u32((calc_width as f64) * aspect));
    }

    let calc_height = cell_height.wrapping_mul(placement_rows);
    let aspect = (width as f64) / (height as f64);
    (round_f64_to_u32((calc_height as f64) * aspect), calc_height)
}

pub(crate) fn kitty_grid_size(
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
) -> (u32, u32) {
    if placement_columns > 0 && placement_rows > 0 {
        return (placement_columns, placement_rows);
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
    let cell_width = nonzero_u32_div(terminal_width_px, u32::from(terminal_cols));
    let cell_height = nonzero_u32_div(terminal_height_px, u32::from(terminal_rows));

    (
        div_ceil_u32(pixel_width.wrapping_add(x_offset), cell_width),
        div_ceil_u32(pixel_height.wrapping_add(y_offset), cell_height),
    )
}

pub(crate) fn div_ceil_u32(numerator: u32, denominator: u32) -> u32 {
    let quotient = numerator.checked_div(denominator).unwrap_or(0);
    let remainder = numerator.checked_rem(denominator).unwrap_or(0);
    quotient.wrapping_add(if remainder == 0 { 0 } else { 1 })
}

pub(crate) fn round_f64_to_u32(value: f64) -> u32 {
    if value <= 0.0 {
        0
    } else if value >= u32::MAX as f64 {
        u32::MAX
    } else {
        (value + 0.5) as u32
    }
}

pub(crate) fn kitty_source_rect(
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
) -> (u32, u32, u32, u32) {
    let x = source_x.min(image_width);
    let y = source_y.min(image_height);
    let width = if source_width > 0 {
        source_width
    } else {
        image_width
    }
    .min(image_width.saturating_sub(x));
    let height = if source_height > 0 {
        source_height
    } else {
        image_height
    }
    .min(image_height.saturating_sub(y));

    (x, y, width, height)
}
