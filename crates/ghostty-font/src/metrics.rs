//! Terminal grid metrics from font face data. Port target: `src/font/Metrics.zig`.

use std::collections::HashMap;

/// Recommended cell width and height for a monospace grid using this font.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub cell_width: u32,
    pub cell_height: u32,
    pub cell_baseline: u32,
    pub underline_position: u32,
    pub underline_thickness: u32,
    pub strikethrough_position: u32,
    pub strikethrough_thickness: u32,
    pub overline_position: i32,
    pub overline_thickness: u32,
    pub box_thickness: u32,
    pub cursor_thickness: u32,
    pub cursor_height: u32,
    pub icon_height: f64,
    pub icon_height_single: f64,
    pub face_width: f64,
    pub face_height: f64,
    pub face_y: f64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl Metrics {
    pub const fn zeroed() -> Self {
        Self {
            cell_width: 0,
            cell_height: 0,
            cell_baseline: 0,
            underline_position: 0,
            underline_thickness: 0,
            strikethrough_position: 0,
            strikethrough_thickness: 0,
            overline_position: 0,
            overline_thickness: 0,
            box_thickness: 0,
            cursor_thickness: 1,
            cursor_height: 0,
            icon_height: 0.0,
            icon_height_single: 0.0,
            face_width: 0.0,
            face_height: 0.0,
            face_y: 0.0,
        }
    }

    fn clamp(&mut self) {
        self.cell_width = self.cell_width.max(1);
        self.cell_height = self.cell_height.max(1);
        self.underline_thickness = self.underline_thickness.max(1);
        self.strikethrough_thickness = self.strikethrough_thickness.max(1);
        self.overline_thickness = self.overline_thickness.max(1);
        self.box_thickness = self.box_thickness.max(1);
        self.cursor_thickness = self.cursor_thickness.max(1);
        self.cursor_height = self.cursor_height.max(1);
        self.icon_height = self.icon_height.max(1.0);
        self.icon_height_single = self.icon_height_single.max(1.0);
        self.face_height = self.face_height.max(1.0);
        self.face_width = self.face_width.max(1.0);
    }

    pub fn apply(&mut self, mods: &ModifierSet) {
        for (&key, modifier) in mods.iter() {
            match key {
                Key::CellWidth | Key::CellHeight => {
                    let original = self.get_u32(key);
                    let new = modifier.apply_u32(original).max(1);
                    if new == original {
                        continue;
                    }
                    self.set_u32(key, new);

                    if key == Key::CellHeight {
                        let original_f64 = f64::from(original);
                        let new_f64 = f64::from(new);
                        let diff = new_f64 - original_f64;
                        let half_diff = diff / 2.0;

                        let position_with_respect_to_center =
                            self.face_y - (original_f64 - self.face_height) / 2.0;

                        let (diff_top, diff_bottom) = if position_with_respect_to_center > 0.0 {
                            (half_diff.ceil(), half_diff.floor())
                        } else {
                            (half_diff.floor(), half_diff.ceil())
                        };

                        add_float_to_int(&mut self.cell_baseline, diff_bottom);
                        self.face_y += diff_bottom;

                        add_float_to_int(&mut self.underline_position, diff_top);
                        add_float_to_int(&mut self.strikethrough_position, diff_top);
                        self.overline_position = self
                            .overline_position
                            .saturating_add(diff_top as i32);
                    }
                }
                Key::IconHeight => {
                    self.icon_height = modifier.apply_f64(self.icon_height);
                    self.icon_height_single = modifier.apply_f64(self.icon_height_single);
                }
                key => {
                    if let Some(v) = self.get_u32_mut(key) {
                        *v = modifier.apply_u32(*v);
                    } else if let Some(v) = self.get_i32_mut(key) {
                        *v = modifier.apply_i32(*v);
                    } else if let Some(v) = self.get_f64_mut(key) {
                        *v = modifier.apply_f64(*v);
                    }
                }
            }
        }
        self.clamp();
    }

    fn get_u32(&self, key: Key) -> u32 {
        match key {
            Key::CellWidth => self.cell_width,
            Key::CellHeight => self.cell_height,
            Key::UnderlineThickness => self.underline_thickness,
            Key::StrikethroughThickness => self.strikethrough_thickness,
            Key::OverlineThickness => self.overline_thickness,
            Key::BoxThickness => self.box_thickness,
            Key::CursorThickness => self.cursor_thickness,
            Key::CursorHeight => self.cursor_height,
            _ => 0,
        }
    }

    fn set_u32(&mut self, key: Key, value: u32) {
        match key {
            Key::CellWidth => self.cell_width = value,
            Key::CellHeight => self.cell_height = value,
            Key::UnderlineThickness => self.underline_thickness = value,
            Key::StrikethroughThickness => self.strikethrough_thickness = value,
            Key::OverlineThickness => self.overline_thickness = value,
            Key::BoxThickness => self.box_thickness = value,
            Key::CursorThickness => self.cursor_thickness = value,
            Key::CursorHeight => self.cursor_height = value,
            _ => {}
        }
    }

    fn get_u32_mut(&mut self, key: Key) -> Option<&mut u32> {
        Some(match key {
            Key::CellWidth => &mut self.cell_width,
            Key::CellHeight => &mut self.cell_height,
            Key::UnderlineThickness => &mut self.underline_thickness,
            Key::StrikethroughThickness => &mut self.strikethrough_thickness,
            Key::OverlineThickness => &mut self.overline_thickness,
            Key::BoxThickness => &mut self.box_thickness,
            Key::CursorThickness => &mut self.cursor_thickness,
            Key::CursorHeight => &mut self.cursor_height,
            _ => return None,
        })
    }

    fn get_i32_mut(&mut self, key: Key) -> Option<&mut i32> {
        match key {
            Key::OverlinePosition => Some(&mut self.overline_position),
            _ => None,
        }
    }

    fn get_f64_mut(&mut self, key: Key) -> Option<&mut f64> {
        match key {
            Key::IconHeight => None, // handled separately
            _ => None,
        }
    }
}

fn add_float_to_int(int: &mut u32, float: f64) {
    debug_assert!(float.floor() == float);
    if float >= 0.0 {
        *int = int.saturating_add(float as u32);
    } else {
        *int = int.saturating_sub((-float) as u32);
    }
}

/// Metrics extracted from a font face before grid rounding.
#[derive(Debug, Clone, Copy, Default)]
pub struct FaceMetrics {
    pub px_per_em: f64,
    pub cell_width: f64,
    pub ascent: f64,
    pub descent: f64,
    pub line_gap: f64,
    pub underline_position: Option<f64>,
    pub underline_thickness: Option<f64>,
    pub strikethrough_position: Option<f64>,
    pub strikethrough_thickness: Option<f64>,
    pub cap_height: Option<f64>,
    pub ex_height: Option<f64>,
    pub ascii_height: Option<f64>,
    pub ic_width: Option<f64>,
}

impl FaceMetrics {
    pub fn line_height(self) -> f64 {
        self.ascent - self.descent + self.line_gap
    }

    pub fn cap_height(self) -> f64 {
        if let Some(v) = self.cap_height {
            if v > 0.0 {
                return v;
            }
        }
        0.75 * self.ascent
    }

    pub fn ex_height(self) -> f64 {
        if let Some(v) = self.ex_height {
            if v > 0.0 {
                return v;
            }
        }
        0.75 * self.cap_height()
    }

    pub fn ascii_height(self) -> f64 {
        if let Some(v) = self.ascii_height {
            if v > 0.0 {
                return v;
            }
        }
        1.5 * self.cap_height()
    }

    pub fn ic_width(self) -> f64 {
        if let Some(v) = self.ic_width {
            if v > 0.0 {
                return v;
            }
        }
        self.ascii_height().min(2.0 * self.cell_width)
    }

    pub fn underline_thickness(self) -> f64 {
        if let Some(v) = self.underline_thickness {
            if v > 0.0 {
                return v;
            }
        }
        0.15 * self.ex_height()
    }

    pub fn strikethrough_thickness(self) -> f64 {
        if let Some(v) = self.strikethrough_thickness {
            if v > 0.0 {
                return v;
            }
        }
        self.underline_thickness()
    }

    pub fn underline_position(self) -> f64 {
        self.underline_position
            .unwrap_or(-self.underline_thickness())
    }

    pub fn strikethrough_position(self) -> f64 {
        self.strikethrough_position
            .unwrap_or((self.ex_height() + self.strikethrough_thickness()) * 0.5)
    }
}

/// Calculate grid metrics from face measurements (mirrors Zig `Metrics.calc`).
pub fn calc(face: FaceMetrics) -> Metrics {
    let face_width = face.cell_width;
    let face_height = face.line_height();

    let cell_width = face_width.round();
    let cell_height = face_height.round();

    let half_line_gap = face.line_gap / 2.0;
    let face_baseline = half_line_gap - face.descent;
    let cell_baseline = (face_baseline - (cell_height - face_height) / 2.0).round();
    let face_y = cell_baseline - face_baseline;

    let top_to_baseline = cell_height - cell_baseline;

    let cap_height = face.cap_height();
    let underline_thickness = face.underline_thickness().ceil().max(1.0);
    let strikethrough_thickness = face.strikethrough_thickness().ceil().max(1.0);
    let underline_position = (top_to_baseline - face.underline_position()).round();
    let strikethrough_position = (top_to_baseline - face.strikethrough_position()).round();

    let icon_height = face_height;
    let icon_height_single = (2.0 * cap_height + face_height) / 3.0;

    let mut result = Metrics {
        cell_width: cell_width as u32,
        cell_height: cell_height as u32,
        cell_baseline: cell_baseline as u32,
        underline_position: underline_position as u32,
        underline_thickness: underline_thickness as u32,
        strikethrough_position: strikethrough_position as u32,
        strikethrough_thickness: strikethrough_thickness as u32,
        overline_position: 0,
        overline_thickness: underline_thickness as u32,
        box_thickness: underline_thickness as u32,
        cursor_height: cell_height as u32,
        icon_height,
        icon_height_single,
        face_width,
        face_height,
        face_y,
        cursor_thickness: 1,
    };

    result.clamp();
    result
}

/// Configurable metric keys (subset of [`Metrics`] fields).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    CellWidth,
    CellHeight,
    UnderlineThickness,
    StrikethroughThickness,
    OverlineThickness,
    BoxThickness,
    CursorThickness,
    CursorHeight,
    IconHeight,
    OverlinePosition,
}

/// A delta modifier (`20%` = 20% larger, not 20% of value).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modifier {
    Percent(f64),
    Absolute(i32),
}

#[derive(Debug, Default)]
pub struct ModifierSet(HashMap<Key, Modifier>);

impl ModifierSet {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: Key, modifier: Modifier) {
        self.0.insert(key, modifier);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Modifier)> {
        self.0.iter()
    }
}

impl Modifier {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        if let Some(stem) = input.strip_suffix('%') {
            let mut percent: f64 = stem.parse().map_err(|_| ParseError::InvalidFormat)?;
            percent /= 100.0;

            if percent <= -1.0 {
                return Ok(Modifier::Percent(0.0));
            }
            if percent < 0.0 {
                return Ok(Modifier::Percent(1.0 + percent));
            }
            return Ok(Modifier::Percent(1.0 + percent));
        }

        let absolute: i32 = input.parse().map_err(|_| ParseError::InvalidFormat)?;
        Ok(Modifier::Absolute(absolute))
    }

    pub fn apply_u32(self, v: u32) -> u32 {
        match self {
            Modifier::Percent(p) => {
                let p_clamped = p.max(0.0);
                let v_f64 = f64::from(v);
                let applied = (v_f64 * p_clamped).round();
                applied.max(0.0) as u32
            }
            Modifier::Absolute(abs) => {
                let applied = i64::from(v).saturating_add(i64::from(abs));
                applied.max(0) as u32
            }
        }
    }

    pub fn apply_i32(self, v: i32) -> i32 {
        match self {
            Modifier::Percent(p) => {
                let p_clamped = p.max(0.0);
                (f64::from(v) * p_clamped).round() as i32
            }
            Modifier::Absolute(abs) => v.saturating_add(abs),
        }
    }

    pub fn apply_f64(self, v: f64) -> f64 {
        match self {
            Modifier::Percent(p) => v * p.max(0.0),
            Modifier::Absolute(abs) => v + f64::from(abs),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_rounds_cell_dimensions() {
        let face = FaceMetrics {
            cell_width: 9.4,
            ascent: 12.0,
            descent: -3.0,
            line_gap: 1.2,
            ..Default::default()
        };
        let m = calc(face);
        assert_eq!(m.cell_width, 9);
        assert_eq!(m.cell_height, 16);
        assert!(m.cell_baseline > 0);
    }

    #[test]
    fn face_metrics_estimators() {
        let face = FaceMetrics {
            ascent: 800.0,
            descent: -200.0,
            cell_width: 500.0,
            ..Default::default()
        };
        assert!((face.cap_height() - 600.0).abs() < f64::EPSILON);
        assert!((face.ex_height() - 450.0).abs() < f64::EPSILON);
    }

    #[test]
    fn apply_cell_width_percent() {
        let mut set = ModifierSet::new();
        set.insert(Key::CellWidth, Modifier::Percent(1.2));

        let mut m = Metrics::zeroed();
        m.cell_width = 100;
        m.apply(&set);
        assert_eq!(m.cell_width, 120);
    }

    #[test]
    fn adjust_cell_height_smaller() {
        let mut set = ModifierSet::new();
        set.insert(Key::CellHeight, Modifier::Percent(0.75));

        let mut m = Metrics::zeroed();
        m.face_y = 0.33;
        m.cell_baseline = 50;
        m.underline_position = 55;
        m.strikethrough_position = 30;
        m.overline_position = 0;
        m.cell_height = 100;
        m.face_height = 99.67;
        m.cursor_height = 100;
        m.apply(&set);

        assert!((m.face_y - (-12.67)).abs() < 0.01);
        assert_eq!(m.cell_height, 75);
        assert_eq!(m.cell_baseline, 37);
        assert_eq!(m.underline_position, 43);
        assert_eq!(m.strikethrough_position, 18);
        assert_eq!(m.overline_position, -12);
        assert_eq!(m.cursor_height, 100);
    }

    #[test]
    fn adjust_cell_height_larger() {
        let mut set = ModifierSet::new();
        set.insert(Key::CellHeight, Modifier::Percent(1.75));

        let mut m = Metrics::zeroed();
        m.face_y = 0.33;
        m.cell_baseline = 50;
        m.underline_position = 55;
        m.strikethrough_position = 30;
        m.overline_position = 0;
        m.cell_height = 100;
        m.face_height = 99.67;
        m.cursor_height = 100;
        m.apply(&set);

        assert!((m.face_y - 37.33).abs() < 0.01);
        assert_eq!(m.cell_height, 175);
        assert_eq!(m.cell_baseline, 87);
        assert_eq!(m.underline_position, 93);
        assert_eq!(m.strikethrough_position, 68);
        assert_eq!(m.overline_position, 38);
        assert_eq!(m.cursor_height, 100);
    }

    #[test]
    fn adjust_icon_height() {
        let mut set = ModifierSet::new();
        set.insert(Key::IconHeight, Modifier::Percent(0.75));

        let mut m = Metrics::zeroed();
        m.icon_height = 100.0;
        m.icon_height_single = 80.0;
        m.face_height = 100.0;
        m.face_y = 1.0;
        m.apply(&set);

        assert!((m.icon_height - 75.0).abs() < f64::EPSILON);
        assert!((m.icon_height_single - 60.0).abs() < f64::EPSILON);
        assert!((m.face_height - 100.0).abs() < f64::EPSILON);
        assert!((m.face_y - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn modifier_parse() {
        assert_eq!(Modifier::parse("100").unwrap(), Modifier::Absolute(100));
        assert_eq!(Modifier::parse("-100").unwrap(), Modifier::Absolute(-100));
        assert_eq!(Modifier::parse("20%").unwrap(), Modifier::Percent(1.2));
        assert_eq!(Modifier::parse("-20%").unwrap(), Modifier::Percent(0.8));
        assert_eq!(Modifier::parse("0%").unwrap(), Modifier::Percent(1.0));
    }

    #[test]
    fn modifier_apply() {
        assert_eq!(Modifier::Percent(0.8).apply_u32(100), 80);
        assert_eq!(Modifier::Percent(1.8).apply_u32(100), 180);
        assert_eq!(Modifier::Absolute(-100).apply_u32(100), 0);
        assert_eq!(Modifier::Absolute(-120).apply_u32(100), 0);
        assert_eq!(Modifier::Absolute(100).apply_u32(100), 200);
    }

    #[test]
    fn desired_size_pixels_from_points() {
        use crate::face::DesiredSize;
        let size = DesiredSize {
            points: 12.0,
            xdpi: 96,
            ydpi: 96,
        };
        assert!((size.pixels() - 16.0).abs() < 0.01);
    }
}
