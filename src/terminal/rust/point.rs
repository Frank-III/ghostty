use crate::constants::*;
use crate::early::*;
use crate::size_types::*;

/// Matches `GhosttyPointTag` / Zig `point.Point.C` tag (`c_int` for C ABI).
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum PointTag {
    ACTIVE = 0,
    VIEWPORT = 1,
    SCREEN = 2,
    HISTORY = 3,
}

impl Default for PointTag {
    fn default() -> Self {
        Self::ACTIVE
    }
}

impl PointTag {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::ACTIVE,
            1 => Self::VIEWPORT,
            2 => Self::SCREEN,
            3 => Self::HISTORY,
            _ => Self::default(),
        }
    }

    pub fn from_u8(v: u8) -> Self {
        Self::from_i32(v as i32)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Coordinate {
    pub(crate) x: CellCountInt,
    pub(crate) y: u32,
}

impl Coordinate {
    pub fn eql(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

/// C ABI layout for `point.Point.C` (tag + value union + padding).
#[repr(C)]
#[derive(Clone, Copy)]
pub union PointCValue {
    pub active: Coordinate,
    pub viewport: Coordinate,
    pub screen: Coordinate,
    pub history: Coordinate,
    pub _padding: [u64; 2],
}

/// C ABI layout for `GhosttyPoint` / Zig `point.Point.C`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PointC {
    pub(crate) tag: PointTag,
    pub(crate) value: PointCValue,
}

const _: () = {
    assert!(core::mem::size_of::<Coordinate>() == 8);
    assert!(core::mem::size_of::<PointCValue>() == 16);
    assert!(core::mem::size_of::<PointC>() == 24);
};

pub enum Point {
    Active(Coordinate),
    Viewport(Coordinate),
    Screen(Coordinate),
    History(Coordinate),
}

impl Point {
    pub fn coord(&self) -> Coordinate {
        match self {
            Self::Active(c) | Self::Viewport(c) | Self::Screen(c) | Self::History(c) => *c,
        }
    }

    pub fn cval(&self) -> PointC {
        let coord = self.coord();
        let mut out = PointC {
            tag: PointTag::ACTIVE,
            value: PointCValue { _padding: [0; 2] },
        };
        match self {
            Self::Active(_) => {
                out.tag = PointTag::ACTIVE;
                out.value.active = coord;
            }
            Self::Viewport(_) => {
                out.tag = PointTag::VIEWPORT;
                out.value.viewport = coord;
            }
            Self::Screen(_) => {
                out.tag = PointTag::SCREEN;
                out.value.screen = coord;
            }
            Self::History(_) => {
                out.tag = PointTag::HISTORY;
                out.value.history = coord;
            }
        }
        out
    }

    pub fn from_c(pt: PointC) -> Self {
        unsafe {
            match pt.tag {
                PointTag::ACTIVE => Self::Active(pt.value.active),
                PointTag::VIEWPORT => Self::Viewport(pt.value.viewport),
                PointTag::SCREEN => Self::Screen(pt.value.screen),
                PointTag::HISTORY => Self::History(pt.value.history),
            }
        }
    }
}
