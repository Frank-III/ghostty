use core::ffi::c_void;
use core::ptr;

use crate::allocator::GhosttyAllocator;
use crate::ansi::{ProtectedMode, StatusDisplay};
use crate::csi::{EraseDisplay, EraseLine};
use crate::mode_def::ModeTag;
use crate::screen_set::ScreenKey;
use crate::screen_types::Screen;
use crate::size_types::CellCountInt;
use crate::terminal_types::{
    Terminal, TerminalFlags, TerminalOptions, TerminalScrollingRegion, TABSTOP_INTERVAL,
};

impl Terminal {
    pub fn active(&self) -> *mut Screen {
        self.screens.active as *mut Screen
    }

    pub fn active_key(&self) -> ScreenKey {
        self.screens.active_key
    }

    pub fn init(opts: &TerminalOptions) -> Self {
        let scrolling_region = if opts.rows > 0 && opts.cols > 0 {
            TerminalScrollingRegion {
                top: 0,
                bottom: opts.rows - 1,
                left: 0,
                right: opts.cols - 1,
            }
        } else {
            TerminalScrollingRegion::default()
        };
        Terminal {
            cols: opts.cols,
            rows: opts.rows,
            scrolling_region,
            colors: opts.colors,
            ..Terminal::default()
        }
    }

    pub fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !alloc.is_null() {
            unsafe {
                self.tabstops.deinit(alloc);
            }
        }
        self.screens.active = ptr::null_mut();
        self.screens.screens = [ptr::null_mut(); 2];
        self.pwd = ptr::null_mut();
        self.title = ptr::null_mut();
    }

    pub fn mode_get(&self, tag: ModeTag) -> bool {
        self.modes.get_by_tag(tag)
    }

    pub fn mode_set(&mut self, tag: ModeTag, value: bool) {
        self.modes.set_by_tag(tag, value);
    }

    pub fn cursor_set_cell(&mut self, row: usize, col: usize) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }

        let origin = self.modes.get_by_tag(ModeTag::from_u16(6));
        let (x_offset, y_offset, x_max, y_max) = if origin {
            (
                self.scrolling_region.left as usize,
                self.scrolling_region.top as usize,
                self.scrolling_region.right as usize + 1,
                self.scrolling_region.bottom as usize + 1,
            )
        } else {
            (0, 0, self.cols as usize, self.rows as usize)
        };

        let row = if row == 0 { 1 } else { row };
        let col = if col == 0 { 1 } else { col };
        let x = (col + x_offset).min(x_max).saturating_sub(1) as CellCountInt;
        let y = (row + y_offset).min(y_max).saturating_sub(1) as CellCountInt;

        unsafe {
            let s = &mut *screen;
            s.cursor.pending_wrap = false;
            s.cursor.x = x;
            s.cursor.y = y;
        }
    }

    pub fn scroll_down(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (old_x, old_y, old_wrap) = unsafe {
            let s = &*screen;
            (s.cursor.x, s.cursor.y, s.cursor.pending_wrap)
        };
        let _ = (old_x, old_y, old_wrap);
        // TODO: cursor_absolute(scrolling_region.left, scrolling_region.top)
        // TODO: insert_lines(count) within scroll region
        // TODO: restore cursor position and pending_wrap
    }

    pub fn scroll_up(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (old_x, old_y, old_wrap) = unsafe {
            let s = &*screen;
            (s.cursor.x, s.cursor.y, s.cursor.pending_wrap)
        };
        let _ = (old_x, old_y, old_wrap);
        // TODO: if region is full-width, move lines into scrollback
        // TODO: otherwise delete_lines(count) within scroll region
        // TODO: restore cursor position and pending_wrap
    }

    pub fn insert_lines(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (cy, cx) = unsafe {
            let s = &*screen;
            (s.cursor.y, s.cursor.x)
        };
        if cy < self.scrolling_region.top
            || cy > self.scrolling_region.bottom
            || cx < self.scrolling_region.left
            || cx > self.scrolling_region.right
        {
            return;
        }
        let region_height = (self.scrolling_region.bottom - cy + 1) as usize;
        let _adjusted = if count > region_height {
            region_height
        } else {
            count
        };
        // TODO: shift rows down within scroll region
        // TODO: clear vacated rows with current SGR state
        // TODO: cursor_absolute(scrolling_region.left, cy)
        unsafe {
            (*screen).cursor.pending_wrap = false;
        }
    }

    pub fn delete_lines(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (cy, cx) = unsafe {
            let s = &*screen;
            (s.cursor.y, s.cursor.x)
        };
        if cy < self.scrolling_region.top
            || cy > self.scrolling_region.bottom
            || cx < self.scrolling_region.left
            || cx > self.scrolling_region.right
        {
            return;
        }
        let region_height = (self.scrolling_region.bottom - cy + 1) as usize;
        let _adjusted = if count > region_height {
            region_height
        } else {
            count
        };
        // TODO: shift rows up within scroll region
        // TODO: clear vacated rows at bottom with current SGR state
        // TODO: cursor_absolute(scrolling_region.left, cy)
        unsafe {
            (*screen).cursor.pending_wrap = false;
        }
    }

    pub fn erase_line(&mut self, mode: EraseLine, protected_req: bool) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (cx, pm) = unsafe {
            let s = &*screen;
            (s.cursor.x, s.protected_mode)
        };
        let cols = self.cols as usize;
        let (_start, _end) = match mode {
            EraseLine::Right => (cx as usize, cols),
            EraseLine::Left => (0, cx as usize + 1),
            EraseLine::Complete => (0, cols),
            EraseLine::RightUnlessPendingWrap => (cx as usize, cols),
        };
        unsafe {
            (*screen).cursor.pending_wrap = false;
        }
        let _protected = pm == ProtectedMode::ISO || protected_req;
        // TODO: clear cells from _start to _end on current row
        // respecting protected attributes if _protected
    }

    pub fn erase_display(&mut self, mode: EraseDisplay, protected_req: bool) {
        match mode {
            EraseDisplay::Complete => {
                let screen = self.active();
                if !screen.is_null() {
                    // TODO: if primary at prompt, prefer scroll_complete
                    unsafe {
                        (*screen).cursor.pending_wrap = false;
                    }
                }
                // TODO: clear_rows(active area, protected)
                self.flags.dirty.clear = true;
            }
            EraseDisplay::Below => {
                self.erase_line(EraseLine::Right, protected_req);
                // TODO: clear_rows below cursor
            }
            EraseDisplay::Above => {
                self.erase_line(EraseLine::Left, protected_req);
                // TODO: clear_rows above cursor
            }
            EraseDisplay::Scrollback => {
                // TODO: erase scrollback history
            }
            EraseDisplay::ScrollComplete => {
                // TODO: try scroll_clear, fall back to complete
                self.erase_display(EraseDisplay::Complete, protected_req);
            }
        }
    }

    pub fn resize(
        &mut self,
        alloc: *const GhosttyAllocator,
        cols: CellCountInt,
        rows: CellCountInt,
    ) {
        if self.cols == cols && self.rows == rows {
            return;
        }
        if cols == 0 || rows == 0 {
            return;
        }
        if self.cols != cols && !alloc.is_null() {
            unsafe {
                self.tabstops.deinit(alloc);
            }
            // TODO: re-init tabstops with new cols via alloc
        }
        // TODO: resize primary screen (with reflow if wraparound mode)
        // TODO: resize alternate screen without reflow
        self.flags.dirty.clear = true;
        self.cols = cols;
        self.rows = rows;
        self.scrolling_region = TerminalScrollingRegion {
            top: 0,
            bottom: rows - 1,
            left: 0,
            right: cols - 1,
        };
    }

    pub fn full_reset(&mut self) {
        self.screens.switch_to(ScreenKey::Primary);
        let _ = self.screens.remove_screen(ScreenKey::Alternate);
        self.modes.reset();
        self.flags = TerminalFlags::default();
        self.tabstops.reset(TABSTOP_INTERVAL as usize);
        self.previous_char = 0;
        self.has_previous_char = false;
        self.mouse_shape = Default::default();
        self.status_display = StatusDisplay::MAIN;
        self.scrolling_region = if self.rows > 0 && self.cols > 0 {
            TerminalScrollingRegion {
                top: 0,
                bottom: self.rows - 1,
                left: 0,
                right: self.cols - 1,
            }
        } else {
            TerminalScrollingRegion::default()
        };
        // TODO: screen.reset(), pwd.clear(), title.clear()
        self.flags.dirty.clear = true;
    }

    pub fn print(&mut self, c: u32) {
        if self.status_display != StatusDisplay::MAIN {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        // TODO: compute right limit (cols vs scrolling_region.right+1)
        // TODO: grapheme clustering when mode 2027 set
        // TODO: wide character handling (double-width, spacer head/tail)
        // TODO: pending wrap handling, wraparound mode check
        // TODO: write codepoint into current cell, advance cursor
        self.previous_char = c;
        self.has_previous_char = true;
    }

    pub fn print_repeat(&mut self, count: usize) {
        if !self.has_previous_char {
            return;
        }
        let c = self.previous_char;
        let n = if count == 0 { 1 } else { count };
        for _ in 0..n {
            self.print(c);
        }
    }

    pub fn write(&mut self, _data: &[u8]) {
        // TODO: feed bytes through VT stream parser, dispatching print /
        // carriage_return / linefeed / CSI / OSC / ... per codepoint
    }

    pub fn carriage_return(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let left = self.scrolling_region.left;
        unsafe {
            let s = &mut *screen;
            s.cursor.x = left;
            s.cursor.pending_wrap = false;
        }
    }

    pub fn linefeed(&mut self) {
        // TODO: move cursor down; if at bottom of scroll region, scroll up
    }

    pub fn backspace(&mut self) {
        // TODO: move cursor left, respecting left margin and wraparound
    }

    pub fn cursor_up(&mut self, count: usize) {
        let screen = self.active();
        if screen.is_null() || count == 0 {
            return;
        }
        let top = if self.modes.get_by_tag(ModeTag::from_u16(6)) {
            self.scrolling_region.top
        } else {
            0
        };
        unsafe {
            let cy = (*screen).cursor.y;
            let max = cy.saturating_sub(top) as usize;
            let n = if count > max { max } else { count };
            (*screen).cursor.y = cy.saturating_sub(n as CellCountInt);
            (*screen).cursor.pending_wrap = false;
        }
    }

    pub fn cursor_down(&mut self, count: usize) {
        let screen = self.active();
        if screen.is_null() || count == 0 {
            return;
        }
        let bottom = if self.modes.get_by_tag(ModeTag::from_u16(6)) {
            self.scrolling_region.bottom
        } else {
            self.rows.saturating_sub(1)
        };
        unsafe {
            let cy = (*screen).cursor.y;
            let max = bottom.saturating_sub(cy) as usize;
            let n = if count > max { max } else { count };
            (*screen).cursor.y = cy.saturating_add(n as CellCountInt);
            (*screen).cursor.pending_wrap = false;
        }
    }

    pub fn cursor_right(&mut self, count: usize) {
        let screen = self.active();
        if screen.is_null() || count == 0 {
            return;
        }
        unsafe {
            let cx = (*screen).cursor.x;
            let right = self.scrolling_region.right;
            let max = right.saturating_sub(cx) as usize;
            let n = if count > max { max } else { count };
            (*screen).cursor.x = cx.saturating_add(n as CellCountInt);
            (*screen).cursor.pending_wrap = false;
        }
    }

    pub fn cursor_left(&mut self, count: usize) {
        let screen = self.active();
        if screen.is_null() || count == 0 {
            return;
        }
        unsafe {
            let cx = (*screen).cursor.x;
            let left = self.scrolling_region.left;
            let max = cx.saturating_sub(left) as usize;
            let n = if count > max { max } else { count };
            (*screen).cursor.x = cx.saturating_sub(n as CellCountInt);
            (*screen).cursor.pending_wrap = false;
        }
    }

    pub fn set_scrolling_region(
        &mut self,
        top: CellCountInt,
        bottom: CellCountInt,
        left: CellCountInt,
        right: CellCountInt,
    ) {
        if top >= bottom || left >= right {
            return;
        }
        if bottom >= self.rows || right >= self.cols {
            return;
        }
        self.scrolling_region = TerminalScrollingRegion {
            top,
            bottom,
            left,
            right,
        };
        self.cursor_set_cell(1, 1);
    }

    pub fn erase_chars(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let cx = unsafe { (*screen).cursor.x as usize };
        let cols = self.cols as usize;
        let end = if cx + count > cols { cols } else { cx + count };
        let _ = end;
        // TODO: clear cells [cx..end] on current row, no line wrap reset
    }
}
