#![allow(unused)]

use crate::style_types::{Underline, RGB};
use crate::vt_parser::MAX_PARAMS;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Name {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Name {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        if v < 8 {
            Some(unsafe { core::mem::transmute(v) })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Unknown {
    pub full: [u16; MAX_PARAMS],
    pub full_len: usize,
    pub partial_len: usize,
}

impl Default for Unknown {
    fn default() -> Self {
        Self {
            full: [0; MAX_PARAMS],
            full_len: 0,
            partial_len: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Attribute {
    Unset,
    Unknown(Unknown),
    Bold,
    ResetBold,
    Italic,
    ResetItalic,
    Faint,
    Underline(Underline),
    UnderlineColor(RGB),
    UnderlineColor256(u8),
    ResetUnderlineColor,
    Overline,
    ResetOverline,
    Blink,
    ResetBlink,
    Inverse,
    ResetInverse,
    Invisible,
    ResetInvisible,
    Strikethrough,
    ResetStrikethrough,
    DirectColorFg(RGB),
    DirectColorBg(RGB),
    Color8Bg(Name),
    Color8Fg(Name),
    ResetFg,
    ResetBg,
    Color8BrightBg(Name),
    Color8BrightFg(Name),
    Palette256Bg(u8),
    Palette256Fg(u8),
}

impl Default for Attribute {
    fn default() -> Self {
        Attribute::Unset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectColorKind {
    Fg,
    Bg,
    Underline,
}

pub struct Parser {
    params: [u16; MAX_PARAMS],
    params_len: usize,
    params_sep: u32,
    idx: usize,
}

impl Parser {
    pub const EMPTY: Self = Self {
        params: [0; MAX_PARAMS],
        params_len: 0,
        params_sep: 0,
        idx: 0,
    };

    #[inline]
    pub fn new(params: [u16; MAX_PARAMS], params_len: usize, params_sep: u32) -> Self {
        Self {
            params,
            params_len,
            params_sep,
            idx: 0,
        }
    }

    #[inline]
    fn is_sep_set(&self, bit: usize) -> bool {
        if bit >= 32 {
            return false;
        }
        (self.params_sep & (1 << bit)) != 0
    }

    #[inline]
    fn count_colon(&self) -> usize {
        let mut count = 0usize;
        let mut i = self.idx;
        while i < self.params_len.saturating_sub(1) && self.is_sep_set(i) {
            count += 1;
            i += 1;
        }
        count
    }

    #[inline]
    fn consume_unknown_colon(&mut self) {
        let count = self.count_colon();
        self.idx += count + 1;
    }

    pub fn next(&mut self) -> Option<Attribute> {
        if self.idx >= self.params_len {
            self.idx += 1;
            return if self.idx == 1 {
                Some(Attribute::Unset)
            } else {
                None
            };
        }

        let slice_start = self.idx;
        let slice_len = self.params_len - self.idx;
        if slice_len == 0 {
            return None;
        }

        let first = self.params[self.idx];
        let colon = self.is_sep_set(self.idx);
        self.idx += 1;

        if colon {
            match first {
                4 | 38 | 48 | 58 => {}
                _ => {
                    let start = self.idx;
                    while self.idx < self.params_len && self.is_sep_set(self.idx) {
                        self.idx += 1;
                    }
                    self.idx += 1;
                    let mut unk = Unknown::default();
                    let copy_len = self.params_len.min(MAX_PARAMS);
                    unk.full[..copy_len].copy_from_slice(&self.params[..copy_len]);
                    unk.full_len = self.params_len;
                    unk.partial_len =
                        (self.idx.saturating_sub(start).saturating_add(1)).min(slice_len);
                    return Some(Attribute::Unknown(unk));
                }
            }
        }

        match first {
            0 => Some(Attribute::Unset),
            1 => Some(Attribute::Bold),
            2 => Some(Attribute::Faint),
            3 => Some(Attribute::Italic),
            4 => {
                if colon {
                    if slice_len < 2 {
                        return Some(Attribute::Underline(Underline::Single));
                    }
                    if self.is_sep_set(self.idx) {
                        self.consume_unknown_colon();
                        return Some(Attribute::Underline(Underline::Single));
                    }
                    let sub = self.params[self.idx];
                    self.idx += 1;
                    return Some(Attribute::Underline(match sub {
                        0 => Underline::None,
                        1 => Underline::Single,
                        2 => Underline::Double,
                        3 => Underline::Curly,
                        4 => Underline::Dotted,
                        5 => Underline::Dashed,
                        _ => Underline::Single,
                    }));
                }
                Some(Attribute::Underline(Underline::Single))
            }
            5 | 6 => Some(Attribute::Blink),
            7 => Some(Attribute::Inverse),
            8 => Some(Attribute::Invisible),
            9 => Some(Attribute::Strikethrough),
            21 => Some(Attribute::Underline(Underline::Double)),
            22 => Some(Attribute::ResetBold),
            23 => Some(Attribute::ResetItalic),
            24 => Some(Attribute::Underline(Underline::None)),
            25 => Some(Attribute::ResetBlink),
            27 => Some(Attribute::ResetInverse),
            28 => Some(Attribute::ResetInvisible),
            29 => Some(Attribute::ResetStrikethrough),
            30..=37 => {
                let name = unsafe { core::mem::transmute::<u8, Name>((first - 30) as u8) };
                Some(Attribute::Color8Fg(name))
            }
            38 => {
                if slice_len >= 2 {
                    let mode = self.params[self.idx];
                    match mode {
                        2 => {
                            if let Some(attr) =
                                self.parse_direct_color(slice_start, colon, DirectColorKind::Fg)
                            {
                                return Some(attr);
                            }
                        }
                        5 => {
                            if slice_len >= 3 {
                                self.idx += 2;
                                return Some(Attribute::Palette256Fg(
                                    self.params[self.idx - 1] as u8,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                Self::make_unknown(&self.params, self.params_len, slice_len)
            }
            39 => Some(Attribute::ResetFg),
            40..=47 => {
                let name = unsafe { core::mem::transmute::<u8, Name>((first - 40) as u8) };
                Some(Attribute::Color8Bg(name))
            }
            48 => {
                if slice_len >= 2 {
                    let mode = self.params[self.idx];
                    match mode {
                        2 => {
                            if let Some(attr) =
                                self.parse_direct_color(slice_start, colon, DirectColorKind::Bg)
                            {
                                return Some(attr);
                            }
                        }
                        5 => {
                            if slice_len >= 3 {
                                self.idx += 2;
                                return Some(Attribute::Palette256Bg(
                                    self.params[self.idx - 1] as u8,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                Self::make_unknown(&self.params, self.params_len, slice_len)
            }
            49 => Some(Attribute::ResetBg),
            53 => Some(Attribute::Overline),
            55 => Some(Attribute::ResetOverline),
            58 => {
                if slice_len >= 2 {
                    let mode = self.params[self.idx];
                    match mode {
                        2 => {
                            if let Some(attr) = self.parse_direct_color(
                                slice_start,
                                colon,
                                DirectColorKind::Underline,
                            ) {
                                return Some(attr);
                            }
                        }
                        5 => {
                            if slice_len >= 3 {
                                self.idx += 2;
                                return Some(Attribute::UnderlineColor256(
                                    self.params[self.idx - 1] as u8,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                Self::make_unknown(&self.params, self.params_len, slice_len)
            }
            59 => Some(Attribute::ResetUnderlineColor),
            90..=97 => {
                let name = unsafe { core::mem::transmute::<u8, Name>((first - 82) as u8) };
                Some(Attribute::Color8BrightFg(name))
            }
            100..=107 => {
                let name = unsafe { core::mem::transmute::<u8, Name>((first - 92) as u8) };
                Some(Attribute::Color8BrightBg(name))
            }
            _ => Self::make_unknown(&self.params, self.params_len, slice_len),
        }
    }

    fn parse_direct_color(
        &mut self,
        slice_start: usize,
        colon: bool,
        kind: DirectColorKind,
    ) -> Option<Attribute> {
        let slice_len = self.params_len - slice_start;
        if slice_len < 5 {
            return None;
        }

        let slice = &self.params[slice_start..self.params_len];
        let rgb = if !colon {
            self.idx += 4;
            RGB::new(slice[2] as u8, slice[3] as u8, slice[4] as u8)
        } else {
            let count = self.count_colon();
            match count {
                3 => {
                    self.idx += 4;
                    RGB::new(slice[2] as u8, slice[3] as u8, slice[4] as u8)
                }
                4 => {
                    self.idx += 5;
                    RGB::new(slice[3] as u8, slice[4] as u8, slice[5] as u8)
                }
                _ => {
                    self.consume_unknown_colon();
                    return None;
                }
            }
        };

        Some(match kind {
            DirectColorKind::Fg => Attribute::DirectColorFg(rgb),
            DirectColorKind::Bg => Attribute::DirectColorBg(rgb),
            DirectColorKind::Underline => Attribute::UnderlineColor(rgb),
        })
    }

    fn make_unknown(
        params: &[u16; MAX_PARAMS],
        params_len: usize,
        slice_len: usize,
    ) -> Option<Attribute> {
        let mut unk = Unknown::default();
        let copy_len = params_len.min(MAX_PARAMS);
        unk.full[..copy_len].copy_from_slice(&params[..copy_len]);
        unk.full_len = params_len;
        unk.partial_len = slice_len;
        Some(Attribute::Unknown(unk))
    }
}
