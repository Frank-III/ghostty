use crate::early::*;
use crate::constants::*;
use crate::size_types::*;

#[repr(u8)]
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
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::ACTIVE,
            1 => Self::VIEWPORT,
            2 => Self::SCREEN,
            3 => Self::HISTORY,
            _ => Self::default(),
        }
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PointC {
    pub(crate) tag: PointTag,
    pub(crate) active: Coordinate,
    pub(crate) viewport: Coordinate,
    pub(crate) screen: Coordinate,
    pub(crate) history: Coordinate,
}

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
            tag: PointTag::default(),
            active: Coordinate::default(),
            viewport: Coordinate::default(),
            screen: Coordinate::default(),
            history: Coordinate::default(),
        };
        match self {
            Self::Active(_) => {
                out.tag = PointTag::ACTIVE;
                out.active = coord;
            }
            Self::Viewport(_) => {
                out.tag = PointTag::VIEWPORT;
                out.viewport = coord;
            }
            Self::Screen(_) => {
                out.tag = PointTag::SCREEN;
                out.screen = coord;
            }
            Self::History(_) => {
                out.tag = PointTag::HISTORY;
                out.history = coord;
            }
        }
        out
    }

    pub fn from_c(pt: PointC) -> Self {
        match pt.tag {
            PointTag::ACTIVE => Self::Active(pt.active),
            PointTag::VIEWPORT => Self::Viewport(pt.viewport),
            PointTag::SCREEN => Self::Screen(pt.screen),
            PointTag::HISTORY => Self::History(pt.history),
        }
    }
}
