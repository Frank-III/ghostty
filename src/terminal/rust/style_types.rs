use crate::style::GhosttyColorRgb;

pub const DEFAULT_ID: u16 = 0;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[inline]
    pub const fn eql(self, other: Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Underline {
    #[default]
    None = 0,
    Single = 1,
    Double = 2,
    Curly = 3,
    Dotted = 4,
    Dashed = 5,
}

impl Underline {
    #[inline]
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::None,
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Curly,
            4 => Self::Dotted,
            5 => Self::Dashed,
            _ => Self::None,
        }
    }
}

pub const FLAG_BOLD: u16 = 1 << 0;
pub const FLAG_ITALIC: u16 = 1 << 1;
pub const FLAG_FAINT: u16 = 1 << 2;
pub const FLAG_BLINK: u16 = 1 << 3;
pub const FLAG_INVERSE: u16 = 1 << 4;
pub const FLAG_INVISIBLE: u16 = 1 << 5;
pub const FLAG_STRIKETHROUGH: u16 = 1 << 6;
pub const FLAG_OVERLINE: u16 = 1 << 7;
pub const FLAG_UNDERLINE: u16 = 1 << 8;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags(pub u16);

impl Flags {
    #[inline(always)]
    pub const fn bold(self) -> bool {
        self.0 & FLAG_BOLD != 0
    }
    #[inline(always)]
    pub fn set_bold(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_BOLD
        } else {
            self.0 &= !FLAG_BOLD
        }
    }

    #[inline(always)]
    pub const fn italic(self) -> bool {
        self.0 & FLAG_ITALIC != 0
    }
    #[inline(always)]
    pub fn set_italic(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_ITALIC
        } else {
            self.0 &= !FLAG_ITALIC
        }
    }

    #[inline(always)]
    pub const fn faint(self) -> bool {
        self.0 & FLAG_FAINT != 0
    }
    #[inline(always)]
    pub fn set_faint(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_FAINT
        } else {
            self.0 &= !FLAG_FAINT
        }
    }

    #[inline(always)]
    pub const fn blink(self) -> bool {
        self.0 & FLAG_BLINK != 0
    }
    #[inline(always)]
    pub fn set_blink(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_BLINK
        } else {
            self.0 &= !FLAG_BLINK
        }
    }

    #[inline(always)]
    pub const fn inverse(self) -> bool {
        self.0 & FLAG_INVERSE != 0
    }
    #[inline(always)]
    pub fn set_inverse(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_INVERSE
        } else {
            self.0 &= !FLAG_INVERSE
        }
    }

    #[inline(always)]
    pub const fn invisible(self) -> bool {
        self.0 & FLAG_INVISIBLE != 0
    }
    #[inline(always)]
    pub fn set_invisible(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_INVISIBLE
        } else {
            self.0 &= !FLAG_INVISIBLE
        }
    }

    #[inline(always)]
    pub const fn strikethrough(self) -> bool {
        self.0 & FLAG_STRIKETHROUGH != 0
    }
    #[inline(always)]
    pub fn set_strikethrough(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_STRIKETHROUGH
        } else {
            self.0 &= !FLAG_STRIKETHROUGH
        }
    }

    #[inline(always)]
    pub const fn overline(self) -> bool {
        self.0 & FLAG_OVERLINE != 0
    }
    #[inline(always)]
    pub fn set_overline(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_OVERLINE
        } else {
            self.0 &= !FLAG_OVERLINE
        }
    }

    #[inline(always)]
    pub const fn underline(self) -> bool {
        self.0 & FLAG_UNDERLINE != 0
    }
    #[inline(always)]
    pub fn set_underline(&mut self, v: bool) {
        if v {
            self.0 |= FLAG_UNDERLINE
        } else {
            self.0 &= !FLAG_UNDERLINE
        }
    }
}

#[repr(C, u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    None,
    Palette(u8),
    Rgb(RGB),
}

impl Default for Color {
    fn default() -> Self {
        Self::None
    }
}

impl Color {
    #[inline]
    pub const fn eql(self, other: Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Palette(a), Self::Palette(b)) => a == b,
            (Self::Rgb(a), Self::Rgb(b)) => a.r == b.r && a.g == b.g && a.b == b.b,
            _ => false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Style {
    pub fg_color: Color,
    pub bg_color: Color,
    pub underline_color: Color,
    pub flags: Flags,
}

impl Style {
    #[inline]
    pub fn is_default(&self) -> bool {
        self.eql(&Self::default())
    }

    #[inline]
    pub fn eql(&self, other: &Self) -> bool {
        self.flags == other.flags
            && self.fg_color.eql(other.fg_color)
            && self.bg_color.eql(other.bg_color)
            && self.underline_color.eql(other.underline_color)
    }

    pub fn underline_rgb(&self, palette: &crate::color_palette::Palette) -> Option<RGB> {
        match self.underline_color {
            Color::None => None,
            Color::Palette(idx) => {
                let i = idx as usize;
                if i < palette.len() {
                    Some(rgb_from_ghostty(unsafe { *palette.get_unchecked(i) }))
                } else {
                    None
                }
            }
            Color::Rgb(rgb) => Some(rgb),
        }
    }

    fn packed_bytes(&self) -> [u8; 16] {
        let tag = |c: &Color| -> u8 {
            match c {
                Color::None => 0,
                Color::Palette(_) => 1,
                Color::Rgb(_) => 2,
            }
        };
        let data = |c: &Color| -> [u8; 3] {
            match c {
                Color::None => [0; 3],
                Color::Palette(idx) => [*idx, 0, 0],
                Color::Rgb(rgb) => [rgb.r, rgb.g, rgb.b],
            }
        };
        let fg = data(&self.fg_color);
        let bg = data(&self.bg_color);
        let ul = data(&self.underline_color);
        let fl = self.flags.0.to_le_bytes();
        [
            tag(&self.fg_color),
            tag(&self.bg_color),
            tag(&self.underline_color),
            fg[0],
            fg[1],
            fg[2],
            bg[0],
            bg[1],
            bg[2],
            ul[0],
            ul[1],
            ul[2],
            fl[0],
            fl[1],
            0,
            0,
        ]
    }

    pub fn hash(&self) -> u64 {
        let bytes = self.packed_bytes();
        let wide = unsafe { core::mem::transmute::<[u8; 16], [u64; 2]>(bytes) };
        hash_int(unsafe { wide.get_unchecked(0) ^ wide.get_unchecked(1) })
    }

    pub fn fmt_vt(&self, buf: &mut [u8]) -> usize {
        let mut pos: usize = 0;
        let push = |buf: &mut [u8], pos: &mut usize, s: &[u8]| {
            let mut i: usize = 0;
            while i < s.len() && *pos < buf.len() {
                unsafe {
                    *buf.get_unchecked_mut(*pos) = *s.get_unchecked(i);
                }
                *pos += 1;
                i += 1;
            }
        };
        push(buf, &mut pos, b"\x1b[0m");
        if self.flags.bold() {
            push(buf, &mut pos, b"\x1b[1m");
        }
        if self.flags.faint() {
            push(buf, &mut pos, b"\x1b[2m");
        }
        if self.flags.italic() {
            push(buf, &mut pos, b"\x1b[3m");
        }
        if self.flags.blink() {
            push(buf, &mut pos, b"\x1b[5m");
        }
        if self.flags.inverse() {
            push(buf, &mut pos, b"\x1b[7m");
        }
        if self.flags.invisible() {
            push(buf, &mut pos, b"\x1b[8m");
        }
        if self.flags.strikethrough() {
            push(buf, &mut pos, b"\x1b[9m");
        }
        if self.flags.overline() {
            push(buf, &mut pos, b"\x1b[53m");
        }
        match self.underline_style() {
            Underline::None => {}
            Underline::Single => push(buf, &mut pos, b"\x1b[4m"),
            Underline::Double => push(buf, &mut pos, b"\x1b[4:2m"),
            Underline::Curly => push(buf, &mut pos, b"\x1b[4:3m"),
            Underline::Dotted => push(buf, &mut pos, b"\x1b[4:4m"),
            Underline::Dashed => push(buf, &mut pos, b"\x1b[4:5m"),
        }
        pos = fmt_color_vt(buf, pos, 38, self.fg_color, None);
        pos = fmt_color_vt(buf, pos, 48, self.bg_color, None);
        pos = fmt_color_vt(buf, pos, 58, self.underline_color, None);
        pos
    }

    #[inline]
    fn underline_style(&self) -> Underline {
        Underline::None
        // The underline style is stored in flags; this is a placeholder until the
        // flags bit-layout is finalized. For now, fmt_vt emits based on flags.
    }
}

fn fmt_color_vt(
    buf: &mut [u8],
    mut pos: usize,
    prefix: u8,
    color: Color,
    palette: Option<&[GhosttyColorRgb; 256]>,
) -> usize {
    match color {
        Color::None => {}
        Color::Palette(idx) => {
            if let Some(p) = palette {
                let i = idx as usize;
                if i < p.len() {
                    let rgb = unsafe { p.get_unchecked(i) };
                    pos = write_fmt_rgb(buf, pos, prefix, rgb.r, rgb.g, rgb.b);
                } else {
                    pos = write_fmt_palette(buf, pos, prefix, idx);
                }
            } else {
                pos = write_fmt_palette(buf, pos, prefix, idx);
            }
        }
        Color::Rgb(rgb) => {
            pos = write_fmt_rgb(buf, pos, prefix, rgb.r, rgb.g, rgb.b);
        }
    }
    pos
}

fn write_fmt_rgb(buf: &mut [u8], pos: usize, prefix: u8, r: u8, g: u8, b: u8) -> usize {
    let mut tmp = [0u8; 32];
    let mut n: usize = 0;
    n = push_tmp_byte(&mut tmp, n, b'\x1b');
    n = push_tmp_byte(&mut tmp, n, b'[');
    n = decimal_to_buf(&mut tmp, n, prefix as u16);
    n = push_tmp_byte(&mut tmp, n, b';');
    n = push_tmp_byte(&mut tmp, n, b'2');
    n = push_tmp_byte(&mut tmp, n, b';');
    n = decimal_to_buf(&mut tmp, n, r as u16);
    n = push_tmp_byte(&mut tmp, n, b';');
    n = decimal_to_buf(&mut tmp, n, g as u16);
    n = push_tmp_byte(&mut tmp, n, b';');
    n = decimal_to_buf(&mut tmp, n, b as u16);
    n = push_tmp_byte(&mut tmp, n, b'm');
    copy_tmp_to_buf(buf, pos, &tmp, n)
}

fn write_fmt_palette(buf: &mut [u8], pos: usize, prefix: u8, idx: u8) -> usize {
    let mut tmp = [0u8; 24];
    let mut n: usize = 0;
    n = push_tmp_byte(&mut tmp, n, b'\x1b');
    n = push_tmp_byte(&mut tmp, n, b'[');
    n = decimal_to_buf(&mut tmp, n, prefix as u16);
    n = push_tmp_byte(&mut tmp, n, b';');
    n = push_tmp_byte(&mut tmp, n, b'5');
    n = push_tmp_byte(&mut tmp, n, b';');
    n = decimal_to_buf(&mut tmp, n, idx as u16);
    n = push_tmp_byte(&mut tmp, n, b'm');
    copy_tmp_to_buf(buf, pos, &tmp, n)
}

#[inline]
fn push_tmp_byte(tmp: &mut [u8], n: usize, b: u8) -> usize {
    if n < tmp.len() {
        unsafe {
            *tmp.get_unchecked_mut(n) = b;
        }
        n + 1
    } else {
        n
    }
}

#[inline]
fn copy_tmp_to_buf(buf: &mut [u8], pos: usize, tmp: &[u8], n: usize) -> usize {
    let mut j: usize = 0;
    while j < n && pos + j < buf.len() && j < tmp.len() {
        unsafe {
            *buf.get_unchecked_mut(pos + j) = *tmp.get_unchecked(j);
        }
        j += 1;
    }
    pos + j
}

fn decimal_to_buf(buf: &mut [u8], mut pos: usize, v: u16) -> usize {
    if v == 0 {
        return push_tmp_byte(buf, pos, b'0');
    }
    let mut tmp = [0u8; 5];
    let mut n = 0usize;
    let mut val = v;
    while val > 0 && n < tmp.len() {
        n = push_tmp_byte(&mut tmp, n, b'0' + (val % 10) as u8);
        val /= 10;
    }
    let mut i = n;
    while i > 0 {
        i -= 1;
        pos = push_tmp_byte(buf, pos, unsafe { *tmp.get_unchecked(i) });
    }
    pos
}

#[inline]
pub fn rgb_to_ghostty(rgb: RGB) -> GhosttyColorRgb {
    GhosttyColorRgb {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

#[inline]
pub const fn rgb_from_ghostty(c: GhosttyColorRgb) -> RGB {
    RGB {
        r: c.r,
        g: c.g,
        b: c.b,
    }
}

fn hash_int(x: u64) -> u64 {
    let mut h = x;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}
