use crate::constants::*;
use crate::mouse_encode::*;

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
