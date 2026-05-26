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
