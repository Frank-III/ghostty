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

#[derive(Clone, Copy)]
pub(crate) struct DynamicPalette {
    current: Palette,
    original: Palette,
    mask: PaletteMask,
}

impl DynamicPalette {
    pub(crate) fn init(default_palette: Palette) -> DynamicPalette {
        DynamicPalette {
            current: default_palette,
            original: default_palette,
            mask: PaletteMask::empty(),
        }
    }

    pub(crate) fn current(&self) -> &Palette {
        &self.current
    }

    pub(crate) fn original(&self) -> &Palette {
        &self.original
    }

    pub(crate) fn changed_count(&self) -> u32 {
        self.mask.count()
    }

    pub(crate) fn set(&mut self, index: u8, color: GhosttyColorRgb) {
        self.current[index as usize] = color;
        self.mask.set(index);
    }

    pub(crate) fn reset(&mut self, index: u8) {
        self.current[index as usize] = self.original[index as usize];
        self.mask.unset(index);
    }

    pub(crate) fn reset_all(&mut self) {
        *self = DynamicPalette::init(self.original);
    }

    pub(crate) fn change_default(&mut self, default_palette: Palette) {
        self.original = default_palette;
        if self.mask.is_empty() {
            self.current = self.original;
            return;
        }

        let previous = self.current;
        self.current = default_palette;
        let mut index = 0u16;
        while index < 256 {
            let idx = index as u8;
            if self.mask.is_set(idx) {
                self.current[index as usize] = previous[index as usize];
            }
            index += 1;
        }
    }
}

#[derive(Clone, Copy)]
struct PaletteMask {
    words: [u64; 4],
}

impl PaletteMask {
    const fn empty() -> PaletteMask {
        PaletteMask { words: [0; 4] }
    }

    fn is_empty(&self) -> bool {
        self.words[0] | self.words[1] | self.words[2] | self.words[3] == 0
    }

    fn count(&self) -> u32 {
        self.words[0].count_ones()
            + self.words[1].count_ones()
            + self.words[2].count_ones()
            + self.words[3].count_ones()
    }

    fn set(&mut self, index: u8) {
        let word = usize::from(index / 64);
        let bit = index % 64;
        self.words[word] |= 1u64 << bit;
    }

    fn unset(&mut self, index: u8) {
        let word = usize::from(index / 64);
        let bit = index % 64;
        self.words[word] &= !(1u64 << bit);
    }

    fn is_set(&self, index: u8) -> bool {
        let word = usize::from(index / 64);
        let bit = index % 64;
        (self.words[word] & (1u64 << bit)) != 0
    }
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

pub(crate) fn rgb_equal(a: GhosttyColorRgb, b: GhosttyColorRgb) -> bool {
    a.r == b.r && a.g == b.g && a.b == b.b
}

pub(crate) fn rgb_contrast(a: GhosttyColorRgb, b: GhosttyColorRgb) -> f64 {
    let a_luminance = rgb_luminance(a);
    let b_luminance = rgb_luminance(b);
    let (lighter, darker) = if a_luminance > b_luminance {
        (a_luminance, b_luminance)
    } else {
        (b_luminance, a_luminance)
    };
    (lighter + 0.05) / (darker + 0.05)
}

pub(crate) fn rgb_luminance(color: GhosttyColorRgb) -> f64 {
    0.2126 * rgb_component_luminance(color.r)
        + 0.7152 * rgb_component_luminance(color.g)
        + 0.0722 * rgb_component_luminance(color.b)
}

pub(crate) fn rgb_perceived_luminance(color: GhosttyColorRgb) -> f64 {
    let r = f64::from(color.r) / 255.0;
    let g = f64::from(color.g) / 255.0;
    let b = f64::from(color.b) / 255.0;
    0.299 * r + 0.587 * g + 0.114 * b
}

pub(crate) fn rgb_from_hex_component(value: &[u8]) -> Option<u8> {
    let divisor = match value.len() {
        1 => 0x000fusize,
        2 => 0x00ffusize,
        3 => 0x0fffusize,
        4 => 0xffffusize,
        _ => return None,
    };

    let mut color = 0usize;
    let mut index = 0usize;
    while index < value.len() {
        color = (color << 4) | usize::from(hex_digit(value[index])?);
        index += 1;
    }

    Some(((color * 0xff) / divisor) as u8)
}

pub(crate) fn rgb_from_intensity_component(value: &[u8]) -> Option<u8> {
    let value = core::str::from_utf8(value).ok()?;
    let intensity = value.parse::<f64>().ok()?;
    if !(0.0..=1.0).contains(&intensity) {
        return None;
    }

    Some((intensity * f64::from(u8::MAX)) as u8)
}

pub(crate) fn rgb_parse(value: &[u8]) -> Option<GhosttyColorRgb> {
    if value.is_empty() {
        return None;
    }

    if value[0] == b'#' {
        return rgb_parse_hash(value);
    }

    rgb_parse_hex_spec(value)
}

pub(crate) fn rgb_parse_hash(value: &[u8]) -> Option<GhosttyColorRgb> {
    if value.first().copied()? != b'#' {
        return None;
    }

    let digits_per_component = match value.len() {
        4 => 1,
        7 => 2,
        10 => 3,
        13 => 4,
        _ => return None,
    };

    let r_start = 1;
    let g_start = r_start + digits_per_component;
    let b_start = g_start + digits_per_component;
    Some(rgb(
        rgb_from_hex_component(&value[r_start..g_start])?,
        rgb_from_hex_component(&value[g_start..b_start])?,
        rgb_from_hex_component(&value[b_start..])?,
    ))
}

pub(crate) fn rgb_parse_hex_spec(value: &[u8]) -> Option<GhosttyColorRgb> {
    if value.len() < b"rgb:a/a/a".len() || !value.starts_with(b"rgb:") {
        return None;
    }

    let mut index = 4usize;
    let red_end = find_byte(value, index, b'/')?;
    let red = rgb_from_hex_component(&value[index..red_end])?;
    index = red_end + 1;

    let green_end = find_byte(value, index, b'/')?;
    let green = rgb_from_hex_component(&value[index..green_end])?;
    index = green_end + 1;

    let blue = rgb_from_hex_component(&value[index..])?;
    Some(rgb(red, green, blue))
}

fn rgb_component_luminance(component: u8) -> f64 {
    let normalized = f64::from(component) / 255.0;
    if normalized <= 0.03928 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

fn find_byte(value: &[u8], start: usize, needle: u8) -> Option<usize> {
    let mut index = start;
    while index < value.len() {
        if value[index] == needle {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
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
