use core::ffi::c_int;
use core::ptr;

use crate::early::*;
use crate::highlight::Pin;
use crate::page_list_methods::cell_iterator_at_pin;
use crate::page_list_types::{PageList, PageListDirection};
use crate::page_types::Cell;
use crate::point::PointTag;
use crate::screen_types::Screen;
use crate::selection_copy::{copy_selection, selection_from_ghostty, selection_to_ghostty};
use crate::selection_types::{Selection, SelectionAdjustment, SelectionOrder};
use crate::selection::{selection_write_bool_impl, selection_write_order_impl, GhosttySelection};
use crate::point::PointC;

fn pages_from_screen(screen: &Screen) -> Option<&PageList> {
    if screen.pages.is_null() {
        return None;
    }
    Some(unsafe { &*screen.pages })
}

impl Selection {
    pub fn order(&self, screen: &Screen) -> Option<SelectionOrder> {
        let pages = match pages_from_screen(screen) {
            Some(p) => p,
            None => return None,
        };
        let (start_x, start_y) = match pages.point_from_pin(PointTag::SCREEN, self.start()) {
            Some(pt) => pt,
            None => return None,
        };
        let (end_x, end_y) = match pages.point_from_pin(PointTag::SCREEN, self.end_pin()) {
            Some(pt) => pt,
            None => return None,
        };

        if self.rectangle {
            if start_y > end_y && start_x >= end_x {
                return Some(SelectionOrder::Reverse);
            }
            if start_y >= end_y && start_x > end_x {
                return Some(SelectionOrder::Reverse);
            }
            if start_y > end_y && start_x < end_x {
                return Some(SelectionOrder::MirroredReverse);
            }
            if start_y < end_y && start_x > end_x {
                return Some(SelectionOrder::MirroredForward);
            }
            return Some(SelectionOrder::Forward);
        }

        if start_y < end_y {
            return Some(SelectionOrder::Forward);
        }
        if start_y > end_y {
            return Some(SelectionOrder::Reverse);
        }
        if start_x <= end_x {
            Some(SelectionOrder::Forward)
        } else {
            Some(SelectionOrder::Reverse)
        }
    }

    pub fn top_left(&self, screen: &Screen) -> Option<Pin> {
        let order = match self.order(screen) {
            Some(o) => o,
            None => return None,
        };
        match order {
            SelectionOrder::Forward => Some(self.start()),
            SelectionOrder::Reverse => Some(self.end_pin()),
            SelectionOrder::MirroredForward => {
                let mut p = self.start();
                p.x = self.end_pin().x;
                Some(p)
            }
            SelectionOrder::MirroredReverse => {
                let mut p = self.end_pin();
                p.x = self.start().x;
                Some(p)
            }
        }
    }

    pub fn bottom_right(&self, screen: &Screen) -> Option<Pin> {
        let order = match self.order(screen) {
            Some(o) => o,
            None => return None,
        };
        match order {
            SelectionOrder::Forward => Some(self.end_pin()),
            SelectionOrder::Reverse => Some(self.start()),
            SelectionOrder::MirroredForward => {
                let mut p = self.end_pin();
                p.x = self.start().x;
                Some(p)
            }
            SelectionOrder::MirroredReverse => {
                let mut p = self.start();
                p.x = self.end_pin().x;
                Some(p)
            }
        }
    }

    pub fn ordered(&self, screen: &Screen, desired: SelectionOrder) -> Option<Selection> {
        let order = match self.order(screen) {
            Some(o) => o,
            None => return None,
        };
        if order == desired {
            return Some(Selection::init(self.start(), self.end_pin(), self.rectangle));
        }

        let tl = match self.top_left(screen) {
            Some(p) => p,
            None => return None,
        };
        let br = match self.bottom_right(screen) {
            Some(p) => p,
            None => return None,
        };
        Some(match desired {
            SelectionOrder::Forward => Selection::init(tl, br, self.rectangle),
            SelectionOrder::Reverse => Selection::init(br, tl, self.rectangle),
            SelectionOrder::MirroredForward | SelectionOrder::MirroredReverse => {
                Selection::init(tl, br, self.rectangle)
            }
        })
    }

    pub fn contains(&self, screen: &Screen, pin: Pin) -> Option<bool> {
        let pages = match pages_from_screen(screen) {
            Some(p) => p,
            None => return None,
        };
        let tl_pin = match self.top_left(screen) {
            Some(p) => p,
            None => return None,
        };
        let br_pin = match self.bottom_right(screen) {
            Some(p) => p,
            None => return None,
        };

        let (tl_x, tl_y) = match pages.point_from_pin(PointTag::SCREEN, tl_pin) {
            Some(pt) => pt,
            None => return None,
        };
        let (br_x, br_y) = match pages.point_from_pin(PointTag::SCREEN, br_pin) {
            Some(pt) => pt,
            None => return None,
        };
        let (p_x, p_y) = match pages.point_from_pin(PointTag::SCREEN, pin) {
            Some(pt) => pt,
            None => return None,
        };

        if self.rectangle {
            return Some(p_y >= tl_y && p_y <= br_y && p_x >= tl_x && p_x <= br_x);
        }

        if tl_y == br_y {
            return Some(p_y == tl_y && p_x >= tl_x && p_x <= br_x);
        }

        if p_y == tl_y {
            return Some(p_x >= tl_x);
        }

        if p_y == br_y {
            return Some(p_x <= br_x);
        }

        Some(p_y > tl_y && p_y < br_y)
    }

    pub fn adjust(&mut self, screen: &Screen, adjustment: SelectionAdjustment) {
        let Some(pages) = pages_from_screen(screen) else {
            return;
        };
        let end_pin = self.end_ptr();
        unsafe {
            match adjustment {
                SelectionAdjustment::Up => {
                    if let Some(new_end) = (*end_pin).up(1) {
                        *end_pin = new_end;
                    } else {
                        self.adjust(screen, SelectionAdjustment::BeginningOfLine);
                    }
                }
                SelectionAdjustment::Down => {
                    let mut current = *end_pin;
                    loop {
                        let Some(next) = current.down(1) else {
                            self.adjust(screen, SelectionAdjustment::EndOfLine);
                            break;
                        };
                        let row = next.row_ptr();
                        let cells = unsafe {
                            let page = &(*next.node).data;
                            core::slice::from_raw_parts(
                                page.row_cells_ptr(row),
                                page.size.cols as usize,
                            )
                        };
                        if Cell::has_text_any(cells) {
                            *end_pin = next;
                            break;
                        }
                        current = next;
                    }
                }
                SelectionAdjustment::Left => {
                    let mut it =
                        cell_iterator_at_pin(*end_pin, PageListDirection::LeftUp, None);
                    let _ = it.next();
                    while let Some(next) = it.next() {
                        let (_row, cell) = next.row_and_cell_ptr();
                        if unsafe { (*cell).has_text() } {
                            *end_pin = next;
                            break;
                        }
                    }
                }
                SelectionAdjustment::Right => {
                    let mut it =
                        cell_iterator_at_pin(*end_pin, PageListDirection::RightDown, None);
                    let _ = it.next();
                    while let Some(next) = it.next() {
                        let (_row, cell) = next.row_and_cell_ptr();
                        if unsafe { (*cell).has_text() } {
                            *end_pin = next;
                            break;
                        }
                    }
                }
                SelectionAdjustment::PageUp => {
                    if let Some(new_end) = (*end_pin).up(pages.rows as usize) {
                        *end_pin = new_end;
                    } else {
                        self.adjust(screen, SelectionAdjustment::Home);
                    }
                }
                SelectionAdjustment::PageDown => {
                    if let Some(new_end) = (*end_pin).down(pages.rows as usize) {
                        *end_pin = new_end;
                    } else {
                        self.adjust(screen, SelectionAdjustment::End);
                    }
                }
                SelectionAdjustment::Home => {
                    if let Some(pin) = pages.pin(PointTag::SCREEN, 0, 0) {
                        *end_pin = pin;
                    }
                }
                SelectionAdjustment::End => {
                    let mut row_it = pages.row_iterator(
                        PageListDirection::LeftUp,
                        PointTag::SCREEN,
                        0,
                        0,
                        None,
                        None,
                        None,
                    );
                    while let Some(next) = row_it.next() {
                        let row = next.row_ptr();
                        let cells = unsafe {
                            let page = &(*next.node).data;
                            core::slice::from_raw_parts(
                                page.row_cells_ptr(row),
                                page.size.cols as usize,
                            )
                        };
                        if Cell::has_text_any(cells) {
                            *end_pin = next;
                            (*end_pin).x = (cells.len() - 1) as u16;
                            break;
                        }
                    }
                }
                SelectionAdjustment::BeginningOfLine => {
                    (*end_pin).x = 0;
                }
                SelectionAdjustment::EndOfLine => {
                    (*end_pin).x = unsafe { (*(*end_pin).node).data.size.cols } - 1;
                }
            }
        }
    }
}

pub(crate) unsafe fn terminal_owned_selection_adjust_impl(
    screen: *mut Screen,
    selection: *mut GhosttySelection,
    adjustment: c_int,
) -> c_int {
    unsafe {
        if screen.is_null() || selection.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let adjustment = match adjustment {
            0 => SelectionAdjustment::Left,
            1 => SelectionAdjustment::Right,
            2 => SelectionAdjustment::Up,
            3 => SelectionAdjustment::Down,
            4 => SelectionAdjustment::Home,
            5 => SelectionAdjustment::End,
            6 => SelectionAdjustment::PageUp,
            7 => SelectionAdjustment::PageDown,
            8 => SelectionAdjustment::BeginningOfLine,
            9 => SelectionAdjustment::EndOfLine,
            _ => return GHOSTTY_INVALID_VALUE,
        };

        let sel_c = ptr::read(selection);
        let Some(mut sel) = selection_from_ghostty(&sel_c) else {
            return GHOSTTY_INVALID_VALUE;
        };

        let screen_ref = &*screen;
        sel.adjust(screen_ref, adjustment);
        let out = selection_to_ghostty(&sel);
        copy_selection(selection, &out)
    }
}

pub(crate) unsafe fn terminal_owned_selection_order_impl(
    screen: *const Screen,
    selection: *const GhosttySelection,
    out_order: *mut c_int,
) -> c_int {
    unsafe {
        if screen.is_null() || selection.is_null() || out_order.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let sel_c = ptr::read(selection);
        let Some(sel) = selection_from_ghostty(&sel_c) else {
            return GHOSTTY_INVALID_VALUE;
        };

        let Some(order) = sel.order(&*screen) else {
            return GHOSTTY_INVALID_VALUE;
        };
        selection_write_order_impl(order as c_int, out_order)
    }
}

pub(crate) unsafe fn terminal_owned_selection_ordered_impl(
    screen: *const Screen,
    selection: *const GhosttySelection,
    desired: c_int,
    out: *mut GhosttySelection,
) -> c_int {
    unsafe {
        if screen.is_null() || selection.is_null() || out.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let desired_order = match desired {
            0 => SelectionOrder::Forward,
            1 => SelectionOrder::Reverse,
            2 => SelectionOrder::MirroredForward,
            3 => SelectionOrder::MirroredReverse,
            _ => return GHOSTTY_INVALID_VALUE,
        };

        let sel_c = ptr::read(selection);
        let Some(sel) = selection_from_ghostty(&sel_c) else {
            return GHOSTTY_INVALID_VALUE;
        };

        let Some(ordered) = sel.ordered(&*screen, desired_order) else {
            return GHOSTTY_INVALID_VALUE;
        };
        let ghostty = selection_to_ghostty(&ordered);
        copy_selection(out, &ghostty)
    }
}

pub(crate) unsafe fn terminal_owned_selection_contains_impl(
    screen: *const Screen,
    selection: *const GhosttySelection,
    point_tag: u8,
    x: u16,
    y: u32,
    out: *mut bool,
) -> c_int {
    unsafe {
        if screen.is_null() || selection.is_null() || out.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let sel_c = ptr::read(selection);
        let Some(sel) = selection_from_ghostty(&sel_c) else {
            return GHOSTTY_INVALID_VALUE;
        };

        let screen_ref = &*screen;
        let pages = match pages_from_screen(screen_ref) {
            Some(p) => p,
            None => return GHOSTTY_INVALID_VALUE,
        };

        let tag = PointTag::from_u8(point_tag);
        let Some(pin) = pages.pin(tag, x, y) else {
            return GHOSTTY_INVALID_VALUE;
        };

        let Some(value) = sel.contains(screen_ref, pin) else {
            return GHOSTTY_INVALID_VALUE;
        };
        selection_write_bool_impl(value, out)
    }
}

pub(crate) unsafe fn terminal_owned_selection_contains_from_point_impl(
    screen: *const Screen,
    selection: *const GhosttySelection,
    pt: PointC,
    out: *mut bool,
) -> c_int {
    unsafe {
        let coord = crate::point::Point::from_c(pt).coord();
        terminal_owned_selection_contains_impl(
            screen,
            selection,
            pt.tag as u8,
            coord.x,
            coord.y,
            out,
        )
    }
}
