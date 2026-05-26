use crate::constants::*;

pub(crate) fn width_pixels(size: GhosttySizeReportSize) -> u64 {
    u64::from(size.columns) * u64::from(size.cell_width)
}

pub(crate) fn height_pixels(size: GhosttySizeReportSize) -> u64 {
    u64::from(size.rows) * u64::from(size.cell_height)
}
