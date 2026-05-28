//! Terminal and shader color types.
//!
//! Port target: `src/terminal/color.zig` (RGB helpers), `src/renderer/shaders/shaders.metal`
//! (sRGB → Display P3 matrix used when `use_display_p3` is false).

/// 8-bit RGB terminal color (`terminal.color.RGB`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// GPU vertex color: `[r, g, b, a]` (`shaderpkg.CellBg` / `CellText.color`).
pub type ShaderRgba = [u8; 4];

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn eql(self, other: Rgb) -> bool {
        self == other
    }

    /// WCAG relative luminance (sRGB approximation).
    pub fn luminance(self) -> f64 {
        let r = component_luminance(self.r);
        let g = component_luminance(self.g);
        let b = component_luminance(self.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// AERT perceived luminance (light vs dark heuristics).
    pub fn perceived_luminance(self) -> f64 {
        0.299 * (f64::from(self.r) / 255.0)
            + 0.587 * (f64::from(self.g) / 255.0)
            + 0.114 * (f64::from(self.b) / 255.0)
    }

    /// Pack into shader byte color with explicit alpha.
    pub fn to_shader_rgba(self, alpha: u8) -> ShaderRgba {
        shader_rgba(self, alpha)
    }
}

fn component_luminance(c: u8) -> f64 {
    let normalized = f64::from(c) / 255.0;
    if normalized <= 0.03928 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

/// Build `[r, g, b, a]` bytes passed to cell shaders.
pub fn shader_rgba(rgb: Rgb, alpha: u8) -> ShaderRgba {
    [rgb.r, rgb.g, rgb.b, alpha]
}

/// sRGB gamma channel → linear ([0, 1]).
pub fn srgb_channel_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear channel → sRGB gamma ([0, 1]).
pub fn linear_channel_to_srgb(v: f32) -> f32 {
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// D50-adapted sRGB → XYZ → Display P3 (`sRGB_DP3` in `shaders.metal`).
pub fn linear_srgb_to_display_p3(linear: [f32; 3]) -> [f32; 3] {
    const M: [[f32; 3]; 3] = [
        [0.515102, 0.291965, 0.157153],
        [0.241182, 0.692236, 0.066582],
        [-0.001049, 0.041881, 0.784378],
    ];
    [
        M[0][0] * linear[0] + M[0][1] * linear[1] + M[0][2] * linear[2],
        M[1][0] * linear[0] + M[1][1] * linear[1] + M[1][2] * linear[2],
        M[2][0] * linear[0] + M[2][1] * linear[1] + M[2][2] * linear[2],
    ]
}

/// Normalize sRGB bytes to linear Display P3 bytes for CPU-side previews/tests.
pub fn srgb_bytes_to_display_p3_bytes(rgb: Rgb) -> Rgb {
    let linear = [
        srgb_channel_to_linear(f32::from(rgb.r) / 255.0),
        srgb_channel_to_linear(f32::from(rgb.g) / 255.0),
        srgb_channel_to_linear(f32::from(rgb.b) / 255.0),
    ];
    let p3 = linear_srgb_to_display_p3(linear);
    Rgb {
        r: float01_to_u8(linear_channel_to_srgb(p3[0])),
        g: float01_to_u8(linear_channel_to_srgb(p3[1])),
        b: float01_to_u8(linear_channel_to_srgb(p3[2])),
    }
}

fn float01_to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_rgba_packs_alpha() {
        assert_eq!(
            shader_rgba(Rgb::new(1, 2, 3), 200),
            [1, 2, 3, 200]
        );
    }

    #[test]
    fn white_has_high_luminance() {
        let white = Rgb::new(255, 255, 255);
        let black = Rgb::new(0, 0, 0);
        assert!(white.luminance() > black.luminance());
    }

    #[test]
    fn srgb_to_display_p3_differs_for_pure_red() {
        let red = Rgb::new(255, 0, 0);
        let converted = srgb_bytes_to_display_p3_bytes(red);
        assert_ne!(converted, red);
    }
}
