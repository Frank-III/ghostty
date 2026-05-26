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

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_layer_matches(layer: c_int, z: i32) -> bool {
    let below_bg_boundary = i32::MIN / 2;
    match layer {
        KITTY_PLACEMENT_LAYER_ALL => true,
        KITTY_PLACEMENT_LAYER_BELOW_BG => z < below_bg_boundary,
        KITTY_PLACEMENT_LAYER_BELOW_TEXT => z >= below_bg_boundary && z < 0,
        KITTY_PLACEMENT_LAYER_ABOVE_TEXT => z >= 0,
        _ => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_iterator_set(
    option: c_int,
    layer: c_int,
    out_layer: *mut c_int,
) -> c_int {
    if out_layer.is_null() || option != KITTY_PLACEMENT_ITERATOR_OPTION_LAYER {
        return GHOSTTY_INVALID_VALUE;
    }

    match layer {
        KITTY_PLACEMENT_LAYER_ALL
        | KITTY_PLACEMENT_LAYER_BELOW_BG
        | KITTY_PLACEMENT_LAYER_BELOW_TEXT
        | KITTY_PLACEMENT_LAYER_ABOVE_TEXT => unsafe {
            ptr::write(out_layer, layer);
        },
        _ => return GHOSTTY_INVALID_VALUE,
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_image_get(
    data: c_int,
    id: u32,
    number: u32,
    width: u32,
    height: u32,
    format: c_int,
    compression: c_int,
    data_ptr: *const u8,
    data_len: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        kitty_image_get_write(
            data,
            id,
            number,
            width,
            height,
            format,
            compression,
            data_ptr,
            data_len,
            out,
        )
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_image_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    id: u32,
    number: u32,
    width: u32,
    height: u32,
    format: c_int,
    compression: c_int,
    data_ptr: *const u8,
    data_len: usize,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut index = 0usize;
    while index < count {
        let key = unsafe { ptr::read(keys.add(index)) };
        let out = unsafe { ptr::read(values.add(index)) };
        let result = unsafe {
            kitty_image_get_write(
                key,
                id,
                number,
                width,
                height,
                format,
                compression,
                data_ptr,
                data_len,
                out,
            )
        };
        if result != GHOSTTY_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, index);
                }
            }
            return result;
        }

        index = index.wrapping_add(1);
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn kitty_image_get_write(
    data: c_int,
    id: u32,
    number: u32,
    width: u32,
    height: u32,
    format: c_int,
    compression: c_int,
    data_ptr: *const u8,
    data_len: usize,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match data {
            KITTY_IMAGE_DATA_ID => ptr::write(out.cast::<u32>(), id),
            KITTY_IMAGE_DATA_NUMBER => ptr::write(out.cast::<u32>(), number),
            KITTY_IMAGE_DATA_WIDTH => ptr::write(out.cast::<u32>(), width),
            KITTY_IMAGE_DATA_HEIGHT => ptr::write(out.cast::<u32>(), height),
            KITTY_IMAGE_DATA_FORMAT => ptr::write(out.cast::<c_int>(), format),
            KITTY_IMAGE_DATA_COMPRESSION => ptr::write(out.cast::<c_int>(), compression),
            KITTY_IMAGE_DATA_PTR => ptr::write(out.cast::<*const u8>(), data_ptr),
            KITTY_IMAGE_DATA_LEN => ptr::write(out.cast::<usize>(), data_len),
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_get(
    data: c_int,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        kitty_placement_get_write(
            data,
            image_id,
            placement_id,
            is_virtual,
            x_offset,
            y_offset,
            source_x,
            source_y,
            source_width,
            source_height,
            columns,
            rows,
            z,
            out,
        )
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_get_multi(
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
) -> c_int {
    if keys.is_null() || values.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut index = 0usize;
    while index < count {
        let key = unsafe { ptr::read(keys.add(index)) };
        let out = unsafe { ptr::read(values.add(index)) };
        let result = unsafe {
            kitty_placement_get_write(
                key,
                image_id,
                placement_id,
                is_virtual,
                x_offset,
                y_offset,
                source_x,
                source_y,
                source_width,
                source_height,
                columns,
                rows,
                z,
                out,
            )
        };
        if result != GHOSTTY_SUCCESS {
            if !out_written.is_null() {
                unsafe {
                    ptr::write(out_written, index);
                }
            }
            return result;
        }

        index = index.wrapping_add(1);
    }

    if !out_written.is_null() {
        unsafe {
            ptr::write(out_written, count);
        }
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn kitty_placement_get_write(
    data: c_int,
    image_id: u32,
    placement_id: u32,
    is_virtual: bool,
    x_offset: u32,
    y_offset: u32,
    source_x: u32,
    source_y: u32,
    source_width: u32,
    source_height: u32,
    columns: u32,
    rows: u32,
    z: i32,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match data {
            KITTY_PLACEMENT_DATA_IMAGE_ID => ptr::write(out.cast::<u32>(), image_id),
            KITTY_PLACEMENT_DATA_PLACEMENT_ID => ptr::write(out.cast::<u32>(), placement_id),
            KITTY_PLACEMENT_DATA_IS_VIRTUAL => ptr::write(out.cast::<bool>(), is_virtual),
            KITTY_PLACEMENT_DATA_X_OFFSET => ptr::write(out.cast::<u32>(), x_offset),
            KITTY_PLACEMENT_DATA_Y_OFFSET => ptr::write(out.cast::<u32>(), y_offset),
            KITTY_PLACEMENT_DATA_SOURCE_X => ptr::write(out.cast::<u32>(), source_x),
            KITTY_PLACEMENT_DATA_SOURCE_Y => ptr::write(out.cast::<u32>(), source_y),
            KITTY_PLACEMENT_DATA_SOURCE_WIDTH => ptr::write(out.cast::<u32>(), source_width),
            KITTY_PLACEMENT_DATA_SOURCE_HEIGHT => ptr::write(out.cast::<u32>(), source_height),
            KITTY_PLACEMENT_DATA_COLUMNS => ptr::write(out.cast::<u32>(), columns),
            KITTY_PLACEMENT_DATA_ROWS => ptr::write(out.cast::<u32>(), rows),
            KITTY_PLACEMENT_DATA_Z => ptr::write(out.cast::<i32>(), z),
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}
