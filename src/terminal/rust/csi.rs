use crate::early::*;
use crate::constants::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseDisplay {
    Below = 0,
    Above = 1,
    Complete = 2,
    Scrollback = 3,
    ScrollComplete = 22,
}

impl Default for EraseDisplay {
    fn default() -> Self {
        EraseDisplay::Below
    }
}

impl EraseDisplay {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(EraseDisplay::Below),
            1 => Some(EraseDisplay::Above),
            2 => Some(EraseDisplay::Complete),
            3 => Some(EraseDisplay::Scrollback),
            22 => Some(EraseDisplay::ScrollComplete),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseLine {
    Right = 0,
    Left = 1,
    Complete = 2,
    RightUnlessPendingWrap = 4,
}

impl Default for EraseLine {
    fn default() -> Self {
        EraseLine::Right
    }
}

impl EraseLine {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(EraseLine::Right),
            1 => Some(EraseLine::Left),
            2 => Some(EraseLine::Complete),
            4 => Some(EraseLine::RightUnlessPendingWrap),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabClear {
    Current = 0,
    All = 3,
}

impl Default for TabClear {
    fn default() -> Self {
        TabClear::Current
    }
}

impl TabClear {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(TabClear::Current),
            3 => Some(TabClear::All),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeReportStyle {
    Csi14t = 0,
    Csi16t = 1,
    Csi18t = 2,
    Csi21t = 3,
}

impl Default for SizeReportStyle {
    fn default() -> Self {
        SizeReportStyle::Csi14t
    }
}

impl SizeReportStyle {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(SizeReportStyle::Csi14t),
            1 => Some(SizeReportStyle::Csi16t),
            2 => Some(SizeReportStyle::Csi18t),
            3 => Some(SizeReportStyle::Csi21t),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlePushPopOp {
    Push = 0,
    Pop = 1,
}

impl Default for TitlePushPopOp {
    fn default() -> Self {
        TitlePushPopOp::Push
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TitlePushPop {
    pub op: TitlePushPopOp,
    pub index: u16,
}

impl Default for TitlePushPop {
    fn default() -> Self {
        TitlePushPop {
            op: TitlePushPopOp::Push,
            index: 0,
        }
    }
}
