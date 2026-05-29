use crate::constants::*;
use crate::early::*;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum C0 {
    NUL = 0x00,
    SOH = 0x01,
    STX = 0x02,
    ENQ = 0x05,
    BEL = 0x07,
    BS = 0x08,
    HT = 0x09,
    LF = 0x0A,
    VT = 0x0B,
    FF = 0x0C,
    CR = 0x0D,
    SO = 0x0E,
    SI = 0x0F,
}

impl Default for C0 {
    fn default() -> Self {
        Self::NUL
    }
}

impl C0 {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0x00 => Self::NUL,
            0x01 => Self::SOH,
            0x02 => Self::STX,
            0x05 => Self::ENQ,
            0x07 => Self::BEL,
            0x08 => Self::BS,
            0x09 => Self::HT,
            0x0A => Self::LF,
            0x0B => Self::VT,
            0x0C => Self::FF,
            0x0D => Self::CR,
            0x0E => Self::SO,
            0x0F => Self::SI,
            _ => Self::default(),
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum RenditionAspect {
    DEFAULT = 0,
    BOLD = 1,
    DEFAULT_FG = 39,
    DEFAULT_BG = 49,
}

impl Default for RenditionAspect {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl RenditionAspect {
    pub fn from_u16(v: u16) -> Self {
        match v {
            0 => Self::DEFAULT,
            1 => Self::BOLD,
            39 => Self::DEFAULT_FG,
            49 => Self::DEFAULT_BG,
            _ => Self::default(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum CursorStyle {
    DEFAULT = 0,
    BLINKING_BLOCK = 1,
    STEADY_BLOCK = 2,
    BLINKING_UNDERLINE = 3,
    STEADY_UNDERLINE = 4,
    BLINKING_BAR = 5,
    STEADY_BAR = 6,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl CursorStyle {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::DEFAULT,
            1 => Self::BLINKING_BLOCK,
            2 => Self::STEADY_BLOCK,
            3 => Self::BLINKING_UNDERLINE,
            4 => Self::STEADY_UNDERLINE,
            5 => Self::BLINKING_BAR,
            6 => Self::STEADY_BAR,
            _ => Self::default(),
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum StatusLineType {
    NONE = 0,
    INDICATOR = 1,
    HOST_WRITABLE = 2,
}

impl Default for StatusLineType {
    fn default() -> Self {
        Self::NONE
    }
}

impl StatusLineType {
    pub fn from_u16(v: u16) -> Self {
        match v {
            0 => Self::NONE,
            1 => Self::INDICATOR,
            2 => Self::HOST_WRITABLE,
            _ => Self::default(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum StatusDisplay {
    MAIN = 0,
    STATUS_LINE = 1,
}

impl Default for StatusDisplay {
    fn default() -> Self {
        Self::MAIN
    }
}

impl StatusDisplay {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::MAIN,
            1 => Self::STATUS_LINE,
            _ => Self::default(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum ModifyKeyFormat {
    LEGACY = 0,
    CURSOR_KEYS = 1,
    FUNCTION_KEYS = 2,
    OTHER_KEYS_NONE = 3,
    OTHER_KEYS_NUMERIC_EXCEPT = 4,
    OTHER_KEYS_NUMERIC = 5,
}

impl Default for ModifyKeyFormat {
    fn default() -> Self {
        Self::LEGACY
    }
}

impl ModifyKeyFormat {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::LEGACY,
            1 => Self::CURSOR_KEYS,
            2 => Self::FUNCTION_KEYS,
            3 => Self::OTHER_KEYS_NONE,
            4 => Self::OTHER_KEYS_NUMERIC_EXCEPT,
            5 => Self::OTHER_KEYS_NUMERIC,
            _ => Self::default(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum ProtectedMode {
    OFF = 0,
    ISO = 1,
    DEC = 2,
}

impl Default for ProtectedMode {
    fn default() -> Self {
        Self::OFF
    }
}

impl ProtectedMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::OFF,
            1 => Self::ISO,
            2 => Self::DEC,
            _ => Self::default(),
        }
    }
}
