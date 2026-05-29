use core::ptr;

use crate::kitty_graphics_image::*;
use crate::kitty_graphics_storage::*;

pub(crate) const PLACEHOLDER_CODEPOINT: u32 = 0x10EEEE;

pub(crate) const DIACRITICS: [u32; 297] = [
    0x0305, 0x030D, 0x030E, 0x0310, 0x0312, 0x033D, 0x033E, 0x033F, 0x0346, 0x034A, 0x034B, 0x034C,
    0x0350, 0x0351, 0x0352, 0x0357, 0x035B, 0x0363, 0x0364, 0x0365, 0x0366, 0x0367, 0x0368, 0x0369,
    0x036A, 0x036B, 0x036C, 0x036D, 0x036E, 0x036F, 0x0483, 0x0484, 0x0485, 0x0486, 0x0487, 0x0592,
    0x0593, 0x0594, 0x0595, 0x0597, 0x0598, 0x0599, 0x059C, 0x059D, 0x059E, 0x059F, 0x05A0, 0x05A1,
    0x05A8, 0x05A9, 0x05AB, 0x05AC, 0x05AF, 0x05C4, 0x0610, 0x0611, 0x0612, 0x0613, 0x0614, 0x0615,
    0x0616, 0x0617, 0x0657, 0x0658, 0x0659, 0x065A, 0x065B, 0x065D, 0x065E, 0x06D6, 0x06D7, 0x06D8,
    0x06D9, 0x06DA, 0x06DB, 0x06DC, 0x06DF, 0x06E0, 0x06E1, 0x06E2, 0x06E4, 0x06E7, 0x06E8, 0x06EB,
    0x06EC, 0x0730, 0x0732, 0x0733, 0x0735, 0x0736, 0x073A, 0x073D, 0x073F, 0x0740, 0x0741, 0x0743,
    0x0745, 0x0747, 0x0749, 0x074A, 0x07EB, 0x07EC, 0x07ED, 0x07EE, 0x07EF, 0x07F0, 0x07F1, 0x07F3,
    0x0816, 0x0817, 0x0818, 0x0819, 0x081B, 0x081C, 0x081D, 0x081E, 0x081F, 0x0820, 0x0821, 0x0822,
    0x0823, 0x0825, 0x0826, 0x0827, 0x0829, 0x082A, 0x082B, 0x082C, 0x082D, 0x0951, 0x0953, 0x0954,
    0x0F82, 0x0F83, 0x0F86, 0x0F87, 0x135D, 0x135E, 0x135F, 0x17DD, 0x193A, 0x1A17, 0x1A75, 0x1A76,
    0x1A77, 0x1A78, 0x1A79, 0x1A7A, 0x1A7B, 0x1A7C, 0x1B6B, 0x1B6D, 0x1B6E, 0x1B6F, 0x1B70, 0x1B71,
    0x1B72, 0x1B73, 0x1CD0, 0x1CD1, 0x1CD2, 0x1CDA, 0x1CDB, 0x1CE0, 0x1DC0, 0x1DC1, 0x1DC3, 0x1DC4,
    0x1DC5, 0x1DC6, 0x1DC7, 0x1DC8, 0x1DC9, 0x1DCB, 0x1DCC, 0x1DD1, 0x1DD2, 0x1DD3, 0x1DD4, 0x1DD5,
    0x1DD6, 0x1DD7, 0x1DD8, 0x1DD9, 0x1DDA, 0x1DDB, 0x1DDC, 0x1DDD, 0x1DDE, 0x1DDF, 0x1DE0, 0x1DE1,
    0x1DE2, 0x1DE3, 0x1DE4, 0x1DE5, 0x1DE6, 0x1DFE, 0x20D0, 0x20D1, 0x20D4, 0x20D5, 0x20D6, 0x20D7,
    0x20DB, 0x20DC, 0x20E1, 0x20E7, 0x20E9, 0x20F0, 0x2CEF, 0x2CF0, 0x2CF1, 0x2DE0, 0x2DE1, 0x2DE2,
    0x2DE3, 0x2DE4, 0x2DE5, 0x2DE6, 0x2DE7, 0x2DE8, 0x2DE9, 0x2DEA, 0x2DEB, 0x2DEC, 0x2DED, 0x2DEE,
    0x2DEF, 0x2DF0, 0x2DF1, 0x2DF2, 0x2DF3, 0x2DF4, 0x2DF5, 0x2DF6, 0x2DF7, 0x2DF8, 0x2DF9, 0x2DFA,
    0x2DFB, 0x2DFC, 0x2DFD, 0x2DFE, 0x2DFF, 0xA66F, 0xA67C, 0xA67D, 0xA6F0, 0xA6F1, 0xA8E0, 0xA8E1,
    0xA8E2, 0xA8E3, 0xA8E4, 0xA8E5, 0xA8E6, 0xA8E7, 0xA8E8, 0xA8E9, 0xA8EA, 0xA8EB, 0xA8EC, 0xA8ED,
    0xA8EE, 0xA8EF, 0xA8F0, 0xA8F1, 0xAAB0, 0xAAB2, 0xAAB3, 0xAAB7, 0xAAB8, 0xAABE, 0xAABF, 0xAAC1,
    0xFE20, 0xFE21, 0xFE22, 0xFE23, 0xFE24, 0xFE25, 0xFE26, 0x10A0F, 0x10A38, 0x1D185, 0x1D186,
    0x1D187, 0x1D188, 0x1D189, 0x1D1AA, 0x1D1AB, 0x1D1AC, 0x1D1AD, 0x1D242, 0x1D243, 0x1D244,
];

pub(crate) fn get_diacritic_index(cp: u32) -> Option<u32> {
    let len = DIACRITICS.len();
    let mut lo: usize = 0;
    let mut hi: usize = len;

    while lo < hi {
        let mid = lo.wrapping_add(hi.wrapping_sub(lo) / 2);
        let val = unsafe { *DIACRITICS.get_unchecked(mid) };
        if val == cp {
            return Some(mid as u32);
        } else if val < cp {
            lo = mid.wrapping_add(1);
        } else {
            hi = mid;
        }
    }

    None
}

#[derive(Clone, Copy)]
pub(crate) struct UnicodePlacement {
    pub pin_node: *mut core::ffi::c_void,
    pub pin_x: u16,
    pub pin_y: u16,
    pub image_id: u32,
    pub placement_id: u32,
    pub col: u32,
    pub row: u32,
    pub width: u32,
    pub height: u32,
}

impl UnicodePlacement {
    pub(crate) fn new() -> Self {
        Self {
            pin_node: ptr::null_mut(),
            pin_x: 0,
            pin_y: 0,
            image_id: 0,
            placement_id: 0,
            col: 0,
            row: 0,
            width: 0,
            height: 0,
        }
    }

    pub(crate) fn render_placement(
        &self,
        storage: &ImageStorage,
        img: &Image,
        cell_width: u32,
        cell_height: u32,
    ) -> Option<RenderPlacement> {
        let p_grid = match self.grid(storage, img, cell_width, cell_height) {
            Some(g) => g,
            None => return None,
        };

        let img_width_f64 = img.width as f64;
        let img_height_f64 = img.height as f64;

        let p_rows_px = (p_grid.1 as f64) * (cell_height as f64);
        let p_cols_px = (p_grid.0 as f64) * (cell_width as f64);

        let (x_scale, y_scale, x_offset, y_offset) =
            if img_width_f64 * p_rows_px > img_height_f64 * p_cols_px {
                let xs = p_cols_px / max_f64(img_width_f64, 1.0);
                let ys = xs;
                let yo = (p_rows_px - img_height_f64 * ys) / 2.0;
                (xs, ys, 0.0f64, yo)
            } else {
                let ys = p_rows_px / max_f64(img_height_f64, 1.0);
                let xs = ys;
                let xo = (p_cols_px - img_width_f64 * xs) / 2.0;
                (xs, ys, xo, 0.0f64)
            };

        let img_scaled_x_offset = x_offset / max_f64(x_scale, 0.0001);
        let img_scaled_y_offset = y_offset / max_f64(y_scale, 0.0001);
        let img_scaled_width = img_width_f64 + (img_scaled_x_offset * 2.0);
        let img_scaled_height = img_height_f64 + (img_scaled_y_offset * 2.0);

        let vp_width = self.width as f64;
        let vp_height = self.height as f64;
        let vp_col = self.col as f64;
        let vp_row = self.row as f64;
        let p_grid_cols = p_grid.0 as f64;
        let p_grid_rows = p_grid.1 as f64;

        let mut iss_width = img_scaled_width * (vp_width / p_grid_cols);
        let mut iss_height = img_scaled_height * (vp_height / p_grid_rows);
        let mut iss_x = img_scaled_width * (vp_col / p_grid_cols);
        let mut iss_y = img_scaled_height * (vp_row / p_grid_rows);

        let mut p_dest_x_offset: f64 = 0.0;
        let mut p_dest_y_offset: f64 = 0.0;
        let mut p_dest_width = (self.width.wrapping_mul(cell_width)) as f64;
        let mut p_dest_height = (self.height.wrapping_mul(cell_height)) as f64;

        if iss_y < img_scaled_y_offset {
            let offset = img_scaled_y_offset - iss_y;
            iss_height -= offset;
            p_dest_y_offset = offset;
            p_dest_height -= offset * y_scale;
            iss_y = 0.0;
            if iss_height > img_height_f64 {
                iss_height = img_height_f64;
                p_dest_height = img_height_f64 * y_scale;
            }
        } else if iss_y + iss_height > img_scaled_height - img_scaled_y_offset {
            iss_y -= img_scaled_y_offset;
            iss_height = img_scaled_height - img_scaled_y_offset - iss_y;
            iss_height -= img_scaled_y_offset;
            p_dest_height = iss_height * y_scale;
        } else {
            iss_y -= img_scaled_y_offset;
        }

        if iss_x < img_scaled_x_offset {
            let offset = img_scaled_x_offset - iss_x;
            iss_width -= offset;
            p_dest_x_offset = offset;
            p_dest_width -= offset * x_scale;
            iss_x = 0.0;
            if iss_width > img_width_f64 {
                iss_width = img_width_f64;
                p_dest_width = img_width_f64 * x_scale;
            }
        } else if iss_x + iss_width > img_scaled_width - img_scaled_x_offset {
            iss_x -= img_scaled_x_offset;
            iss_width = img_scaled_width - img_scaled_x_offset - iss_x;
            iss_width -= img_scaled_x_offset;
            p_dest_width = iss_width * x_scale;
        } else {
            iss_x -= img_scaled_x_offset;
        }

        if iss_width <= 0.0 || iss_height <= 0.0 {
            return Some(RenderPlacement {
                pin_node: self.pin_node,
                pin_x: self.pin_x,
                pin_y: self.pin_y,
                offset_x: 0,
                offset_y: 0,
                source_x: 0,
                source_y: 0,
                source_width: 0,
                source_height: 0,
                dest_width: 0,
                dest_height: 0,
            });
        }

        Some(RenderPlacement {
            pin_node: self.pin_node,
            pin_x: self.pin_x,
            pin_y: self.pin_y,
            offset_x: round_f64_val(p_dest_x_offset * x_scale),
            offset_y: round_f64_val(p_dest_y_offset * y_scale),
            source_x: round_f64_val(iss_x),
            source_y: round_f64_val(iss_y),
            source_width: round_f64_val(iss_width),
            source_height: round_f64_val(iss_height),
            dest_width: round_f64_val(p_dest_width),
            dest_height: round_f64_val(p_dest_height),
        })
    }

    fn grid(
        &self,
        _storage: &ImageStorage,
        image: &Image,
        cell_width: u32,
        cell_height: u32,
    ) -> Option<(u32, u32)> {
        let mut rows: u32 = 0;
        let mut columns: u32 = 0;

        if rows == 0 {
            rows = (image.height.wrapping_add(cell_height).wrapping_sub(1)) / cell_height;
        }
        if columns == 0 {
            columns = (image.width.wrapping_add(cell_width).wrapping_sub(1)) / cell_width;
        }

        Some((columns, rows))
    }
}

fn max_f64(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

fn round_f64_val(v: f64) -> u32 {
    if v <= 0.0 {
        0
    } else if v >= u32::MAX as f64 {
        u32::MAX
    } else {
        (v + 0.5) as u32
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RenderPlacement {
    pub pin_node: *mut core::ffi::c_void,
    pub pin_x: u16,
    pub pin_y: u16,
    pub offset_x: u32,
    pub offset_y: u32,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub dest_width: u32,
    pub dest_height: u32,
}

impl RenderPlacement {
    pub(crate) fn new() -> Self {
        Self {
            pin_node: ptr::null_mut(),
            pin_x: 0,
            pin_y: 0,
            offset_x: 0,
            offset_y: 0,
            source_x: 0,
            source_y: 0,
            source_width: 0,
            source_height: 0,
            dest_width: 0,
            dest_height: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct IncompletePlacement {
    pin_node: *mut core::ffi::c_void,
    pin_x: u16,
    pin_y: u16,
    image_id_low: u32,
    image_id_high: Option<u8>,
    placement_id: Option<u32>,
    row: Option<u32>,
    col: Option<u32>,
    width: u32,
}

impl IncompletePlacement {
    fn new() -> Self {
        Self {
            pin_node: ptr::null_mut(),
            pin_x: 0,
            pin_y: 0,
            image_id_low: 0,
            image_id_high: None,
            placement_id: None,
            row: None,
            col: None,
            width: 1,
        }
    }

    fn can_append(&self, other: &IncompletePlacement) -> bool {
        if self.image_id_low != other.image_id_low {
            return false;
        }

        match (self.placement_id, other.placement_id) {
            (Some(a), Some(b)) => {
                if a != b {
                    return false;
                }
            }
            (None, Some(_)) => return false,
            (Some(_), None) => {}
            (None, None) => {}
        }

        match other.row {
            Some(r) => match self.row {
                Some(sr) => {
                    if r != sr {
                        return false;
                    }
                }
                None => return false,
            },
            None => {}
        }

        match other.col {
            Some(c) => match self.col {
                Some(sc) => {
                    let expected = sc.wrapping_add(self.width);
                    if c != expected {
                        return false;
                    }
                }
                None => return false,
            },
            None => {}
        }

        match (self.image_id_high, other.image_id_high) {
            (Some(a), Some(b)) => {
                if a != b {
                    return false;
                }
            }
            (None, Some(_)) => return false,
            _ => {}
        }

        true
    }

    fn complete(&self) -> UnicodePlacement {
        let low = self.image_id_low;
        let high = match self.image_id_high {
            Some(h) => h as u32,
            None => 0,
        };

        let pid = match self.placement_id {
            Some(v) => v,
            None => 0,
        };
        let col = match self.col {
            Some(v) => v,
            None => 0,
        };
        let row = match self.row {
            Some(v) => v,
            None => 0,
        };

        UnicodePlacement {
            pin_node: self.pin_node,
            pin_x: self.pin_x,
            pin_y: self.pin_y,
            image_id: low | (high << 24),
            placement_id: pid,
            col,
            row,
            width: self.width,
            height: 1,
        }
    }
}

pub(crate) fn color_to_id_24(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub(crate) fn palette_to_id_24(palette: u8) -> u32 {
    palette as u32
}
