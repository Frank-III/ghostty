use crate::style::GhosttyColorRgb;

pub(crate) type Palette = [GhosttyColorRgb; 256];

pub(crate) fn default_named_rgb(index: u8) -> Option<GhosttyColorRgb> {
    match index {
        0 => Some(rgb(0x1D, 0x1F, 0x21)),
        1 => Some(rgb(0xCC, 0x66, 0x66)),
        2 => Some(rgb(0xB5, 0xBD, 0x68)),
        3 => Some(rgb(0xF0, 0xC6, 0x74)),
        4 => Some(rgb(0x81, 0xA2, 0xBE)),
        5 => Some(rgb(0xB2, 0x94, 0xBB)),
        6 => Some(rgb(0x8A, 0xBE, 0xB7)),
        7 => Some(rgb(0xC5, 0xC8, 0xC6)),
        8 => Some(rgb(0x66, 0x66, 0x66)),
        9 => Some(rgb(0xD5, 0x4E, 0x53)),
        10 => Some(rgb(0xB9, 0xCA, 0x4A)),
        11 => Some(rgb(0xE7, 0xC5, 0x47)),
        12 => Some(rgb(0x7A, 0xA6, 0xDA)),
        13 => Some(rgb(0xC3, 0x97, 0xD8)),
        14 => Some(rgb(0x70, 0xC0, 0xB1)),
        15 => Some(rgb(0xEA, 0xEA, 0xEA)),
        _ => None,
    }
}

pub(crate) fn default_palette_color(index: u8) -> GhosttyColorRgb {
    if let Some(color) = default_named_rgb(index) {
        return color;
    }

    if index < 232 {
        let cube = index - 16;
        let r = cube / 36;
        let g = (cube / 6) % 6;
        let b = cube % 6;
        return rgb(cube_component(r), cube_component(g), cube_component(b));
    }

    let value = ((index - 232) * 10) + 8;
    rgb(value, value, value)
}

pub(crate) fn default_palette() -> Palette {
    let mut result = [rgb(0, 0, 0); 256];
    let mut index = 0usize;
    while index < 256 {
        result[index] = default_palette_color(index as u8);
        index += 1;
    }
    result
}

pub(crate) fn special_osc4(index: u8) -> Option<u16> {
    if index < 5 {
        Some(u16::from(index) + 256)
    } else {
        None
    }
}

pub(crate) fn dynamic_next(dynamic: u8) -> Option<u8> {
    if (10..19).contains(&dynamic) {
        Some(dynamic + 1)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DynamicRgb {
    override_color: Option<GhosttyColorRgb>,
    default_color: Option<GhosttyColorRgb>,
}

impl DynamicRgb {
    pub(crate) const UNSET: DynamicRgb = DynamicRgb {
        override_color: None,
        default_color: None,
    };

    pub(crate) const fn init(default_color: GhosttyColorRgb) -> DynamicRgb {
        DynamicRgb {
            override_color: None,
            default_color: Some(default_color),
        }
    }

    pub(crate) fn get(&self) -> Option<GhosttyColorRgb> {
        match self.override_color {
            Some(color) => Some(color),
            None => self.default_color,
        }
    }

    pub(crate) fn set(&mut self, color: GhosttyColorRgb) {
        self.override_color = Some(color);
    }

    pub(crate) fn reset(&mut self) {
        self.override_color = self.default_color;
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> GhosttyColorRgb {
    GhosttyColorRgb { r, g, b }
}

const fn cube_component(value: u8) -> u8 {
    if value == 0 {
        0
    } else {
        value * 40 + 55
    }
}
