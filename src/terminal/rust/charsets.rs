use crate::constants::*;
use crate::early::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharsetSlot {
    G0 = 0,
    G1 = 1,
    G2 = 2,
    G3 = 3,
}

impl Default for CharsetSlot {
    fn default() -> Self {
        CharsetSlot::G0
    }
}

impl CharsetSlot {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(CharsetSlot::G0),
            1 => Some(CharsetSlot::G1),
            2 => Some(CharsetSlot::G2),
            3 => Some(CharsetSlot::G3),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveSlot {
    GL = 0,
    GR = 1,
}

impl Default for ActiveSlot {
    fn default() -> Self {
        ActiveSlot::GL
    }
}

impl ActiveSlot {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(ActiveSlot::GL),
            1 => Some(ActiveSlot::GR),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharsetId {
    Utf8 = 0,
    Ascii = 1,
    British = 2,
    DecSpecial = 3,
}

impl Default for CharsetId {
    fn default() -> Self {
        CharsetId::Utf8
    }
}

impl CharsetId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(CharsetId::Utf8),
            1 => Some(CharsetId::Ascii),
            2 => Some(CharsetId::British),
            3 => Some(CharsetId::DecSpecial),
            _ => None,
        }
    }
}

const TABLE_LEN: usize = 256;

const fn identity_table() -> [u16; TABLE_LEN] {
    let mut t = [0u16; TABLE_LEN];
    let mut i: usize = 0;
    while i < TABLE_LEN {
        t[i] = i as u16;
        i += 1;
    }
    t
}

const fn british_table() -> [u16; TABLE_LEN] {
    let mut t = identity_table();
    t[0x23] = 0x00a3;
    t
}

const fn dec_special_table() -> [u16; TABLE_LEN] {
    let mut t = identity_table();
    t[0x60] = 0x25C6;
    t[0x61] = 0x2592;
    t[0x62] = 0x2409;
    t[0x63] = 0x240C;
    t[0x64] = 0x240D;
    t[0x65] = 0x240A;
    t[0x66] = 0x00B0;
    t[0x67] = 0x00B1;
    t[0x68] = 0x2424;
    t[0x69] = 0x240B;
    t[0x6A] = 0x2518;
    t[0x6B] = 0x2510;
    t[0x6C] = 0x250C;
    t[0x6D] = 0x2514;
    t[0x6E] = 0x253C;
    t[0x6F] = 0x23BA;
    t[0x70] = 0x23BB;
    t[0x71] = 0x2500;
    t[0x72] = 0x23BC;
    t[0x73] = 0x23BD;
    t[0x74] = 0x251C;
    t[0x75] = 0x2524;
    t[0x76] = 0x2534;
    t[0x77] = 0x252C;
    t[0x78] = 0x2502;
    t[0x79] = 0x2264;
    t[0x7A] = 0x2265;
    t[0x7B] = 0x03C0;
    t[0x7C] = 0x2260;
    t[0x7D] = 0x00A3;
    t[0x7E] = 0x00B7;
    t
}

pub static ASCII_TABLE: [u16; TABLE_LEN] = identity_table();
pub static BRITISH_TABLE: [u16; TABLE_LEN] = british_table();
pub static DEC_SPECIAL_TABLE: [u16; TABLE_LEN] = dec_special_table();

pub fn charset_table(id: CharsetId) -> *const [u16; TABLE_LEN] {
    match id {
        CharsetId::Ascii => &ASCII_TABLE,
        CharsetId::British => &BRITISH_TABLE,
        CharsetId::DecSpecial => &DEC_SPECIAL_TABLE,
        CharsetId::Utf8 => &ASCII_TABLE,
    }
}
