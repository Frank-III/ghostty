use crate::constants::*;
use crate::early::*;
use crate::highlight::Pin;
use crate::page_list_types::PageList;
use crate::size_types::*;
use core::ptr;

#[repr(u8)]
#[derive(PartialEq, Eq)]
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

    pub fn deinit(self: Selection, pages: *mut PageList) {
        match self.bounds {
            SelectionBounds::Tracked(v) => {
                if pages.is_null() {
                    return;
                }
                unsafe {
                    let pl: &mut PageList = &mut *pages;
                    if !v.start.is_null() {
                        pl.untrack_pin(v.start);
                    }
                    if !v.end.is_null() {
                        pl.untrack_pin(v.end);
                    }
                }
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

    pub fn track(self: &Selection, pages: *mut PageList) -> Option<Selection> {
        debug_assert!(!self.is_tracked());
        if pages.is_null() {
            return None;
        }
        let u = match &self.bounds {
            SelectionBounds::Untracked(u) => u,
            SelectionBounds::Tracked(_) => return None,
        };
        unsafe {
            let pl: &mut PageList = &mut *pages;
            let tracked_start = pl.track_pin(u.start);
            if tracked_start.is_null() {
                return None;
            }
            let tracked_end = pl.track_pin(u.end);
            if tracked_end.is_null() {
                pl.untrack_pin(tracked_start);
                return None;
            }
            Some(Selection {
                bounds: SelectionBounds::Tracked(SelectionBoundsTracked {
                    start: tracked_start,
                    end: tracked_end,
                }),
                rectangle: self.rectangle,
            })
        }
    }

    pub fn eql(self: &Selection, other: &Selection) -> bool {
        let s = self.start();
        let e = self.end_pin();
        let os = other.start();
        let oe = other.end_pin();
        s.eql(os) && e.eql(oe) && self.rectangle == other.rectangle
    }
}
