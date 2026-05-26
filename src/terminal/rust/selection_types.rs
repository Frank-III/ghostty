use crate::early::*;
use crate::constants::*;
use crate::size_types::*;
use crate::highlight::Pin;

#[repr(u8)]
pub enum SelectionOrder {
    Forward = 0,
    Reverse = 1,
    MirroredForward = 2,
    MirroredReverse = 3,
}

#[repr(u8)]
pub enum SelectionAdjustment {
    Left = 0,
    Right = 1,
    Up = 2,
    Down = 3,
    Home = 4,
    End = 5,
    PageUp = 6,
    PageDown = 7,
    BeginningOfLine = 8,
    EndOfLine = 9,
}

#[derive(Clone, Copy)]
pub struct SelectionBoundsUntracked {
    pub start: Pin,
    pub end: Pin,
}

#[derive(Clone, Copy)]
pub struct SelectionBoundsTracked {
    pub start: *mut Pin,
    pub end: *mut Pin,
}

pub enum SelectionBounds {
    Untracked(SelectionBoundsUntracked),
    Tracked(SelectionBoundsTracked),
}

pub struct Selection {
    pub bounds: SelectionBounds,
    pub rectangle: bool,
}

impl Selection {
    pub fn init(start: Pin, end: Pin, rect: bool) -> Selection {
        Selection {
            bounds: SelectionBounds::Untracked(SelectionBoundsUntracked { start, end }),
            rectangle: rect,
        }
    }

    pub fn deinit(self: Selection) {
        match self.bounds {
            SelectionBounds::Tracked(_v) => {
                // TODO: requires screen.untrackPin
            }
            SelectionBounds::Untracked(_) => {}
        }
    }

    pub fn start(&self) -> Pin {
        match &self.bounds {
            SelectionBounds::Untracked(u) => u.start,
            SelectionBounds::Tracked(t) => unsafe { *t.start },
        }
    }

    pub fn end_pin(&self) -> Pin {
        match &self.bounds {
            SelectionBounds::Untracked(u) => u.end,
            SelectionBounds::Tracked(t) => unsafe { *t.end },
        }
    }

    pub fn start_ptr(&mut self) -> *mut Pin {
        match &mut self.bounds {
            SelectionBounds::Untracked(u) => &mut u.start,
            SelectionBounds::Tracked(t) => t.start,
        }
    }

    pub fn end_ptr(&mut self) -> *mut Pin {
        match &mut self.bounds {
            SelectionBounds::Untracked(u) => &mut u.end,
            SelectionBounds::Tracked(t) => t.end,
        }
    }

    pub fn is_tracked(&self) -> bool {
        matches!(&self.bounds, SelectionBounds::Tracked(_))
    }

    pub fn eql(self: &Selection, other: &Selection) -> bool {
        let s = self.start();
        let e = self.end_pin();
        let os = other.start();
        let oe = other.end_pin();
        s.eql(os) && e.eql(oe) && self.rectangle == other.rectangle
    }
}
