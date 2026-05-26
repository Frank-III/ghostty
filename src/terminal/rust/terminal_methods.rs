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
use crate::Cell;
use crate::Wide;

const MODE_WRAPAROUND: u16 = 7;
const MODE_LINEFEED: u16 = 20 | (1 << 15);

#[inline]
fn codepoint_width(c: u32) -> usize {
    if c <= 0xFF {
        return 1;
    }
    if (c >= 0x1100 && c <= 0x115F)
        || (c >= 0x2E80 && c <= 0x303E)
        || (c >= 0x3041 && c <= 0x33BF)
        || (c >= 0x3400 && c <= 0x4DBF)
        || (c >= 0x4E00 && c <= 0x9FFF)
        || (c >= 0xAC00 && c <= 0xD7AF)
        || (c >= 0xF900 && c <= 0xFAFF)
        || (c >= 0xFE30 && c <= 0xFE4F)
        || (c >= 0xFF01 && c <= 0xFF60)
        || (c >= 0xFFE0 && c <= 0xFFE6)
        || (c >= 0x20000 && c <= 0x2FFFD)
        || (c >= 0x30000 && c <= 0x3FFFD)
    {
        return 2;
    }
    1
}

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

    fn cursor_resync(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            let s = &mut *screen;
            if s.cursor.page_pin.is_null() {
                return;
            }
            let node = (*s.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &(*node).data;
            let rows = page.rows_ptr();
            s.cursor.page_row = rows.add(s.cursor.y as usize);
            s.cursor.page_cell = page.row_cells_ptr(s.cursor.page_row).add(s.cursor.x as usize);
        }
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
        let top = self.scrolling_region.top;
        let left = self.scrolling_region.left;

        unsafe {
            let s = &mut *screen;
            s.cursor.x = left;
            s.cursor.y = top;
        }
        self.cursor_resync();

        self.insert_lines(count);

        unsafe {
            let s = &mut *screen;
            s.cursor.x = old_x;
            s.cursor.y = old_y;
            s.cursor.pending_wrap = old_wrap;
        }
        self.cursor_resync();
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
        let top = self.scrolling_region.top;
        let left = self.scrolling_region.left;

        unsafe {
            let s = &mut *screen;
            s.cursor.x = left;
            s.cursor.y = top;
        }
        self.cursor_resync();

        self.delete_lines(count);

        unsafe {
            let s = &mut *screen;
            s.cursor.x = old_x;
            s.cursor.y = old_y;
            s.cursor.pending_wrap = old_wrap;
        }
        self.cursor_resync();
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

        self.cursor_resync();

        let (cx, pm) = unsafe {
            let s = &*screen;
            (s.cursor.x, s.protected_mode)
        };
        let cols = self.cols as usize;
        let cx_usize = cx as usize;

        let pending = unsafe { (*screen).cursor.pending_wrap };

        let (start, end) = match mode {
            EraseLine::Right => (cx_usize, cols),
            EraseLine::Left => (0, cx_usize + 1),
            EraseLine::Complete => (0, cols),
            EraseLine::RightUnlessPendingWrap => {
                if pending {
                    (cx_usize, cx_usize)
                } else {
                    (cx_usize, cols)
                }
            }
        };

        unsafe {
            (*screen).cursor.pending_wrap = false;
        }

        let protected = pm == ProtectedMode::ISO || protected_req;

        unsafe {
            (*(*screen).cursor.page_row).set_dirty(true);
        }

        if start < end {
            let blank = Cell::default();
            unsafe {
                let page_cell = (*screen).cursor.page_cell;
                let cells_start = page_cell.sub(cx_usize);
                if protected {
                    for i in start..end {
                        let cell_ptr = cells_start.add(i);
                        let cell = ptr::read(cell_ptr);
                        if !cell.protected() {
                            ptr::write(cell_ptr, blank);
                        }
                    }
                } else {
                    for i in start..end {
                        ptr::write(cells_start.add(i), blank);
                    }
                }
            }
        }
    }

    pub fn erase_display(&mut self, mode: EraseDisplay, protected_req: bool) {
        match mode {
            EraseDisplay::Complete => {
                let screen = self.active();
                if !screen.is_null() {
                    unsafe {
                        (*screen).cursor.pending_wrap = false;
                    }
                }
                self.erase_line(EraseLine::Complete, protected_req);
                // TODO: clear_rows(active area, protected)
                self.flags.dirty.clear = true;
            }
            EraseDisplay::Below => {
                self.erase_line(EraseLine::Right, protected_req);
                let screen = self.active();
                if screen.is_null() {
                    return;
                }
                let cy = unsafe { (*screen).cursor.y };
                if cy + 1 < self.rows {
                    // TODO: clear_rows below cursor via screen methods
                }
            }
            EraseDisplay::Above => {
                self.erase_line(EraseLine::Left, protected_req);
                let screen = self.active();
                if screen.is_null() {
                    return;
                }
                let cy = unsafe { (*screen).cursor.y };
                if cy > 0 {
                    // TODO: clear_rows above cursor via screen methods
                }
            }
            EraseDisplay::Scrollback => {
                // TODO: erase scrollback history
            }
            EraseDisplay::ScrollComplete => {
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

        self.cursor_resync();

        let cols = self.cols as usize;
        let sr_right = self.scrolling_region.right as usize;
        let sr_left = self.scrolling_region.left;
        let wraparound = self.modes.get_by_tag(ModeTag::from_u16(MODE_WRAPAROUND));

        let width = codepoint_width(c);

        let pending = unsafe { (*screen).cursor.pending_wrap };

        if pending && wraparound {
            let x = unsafe { (*screen).cursor.x as usize };
            let at_edge = x == cols - 1;
            unsafe {
                if at_edge {
                    (*(*screen).cursor.page_row).set_wrap(true);
                }
                (*(*screen).cursor.page_row).set_dirty(true);
                (*screen).cursor.pending_wrap = false;
            }

            self.index();

            unsafe {
                (*screen).cursor.x = sr_left;
            }
            self.cursor_resync();

            unsafe {
                if at_edge {
                    (*(*screen).cursor.page_row).set_wrap_continuation(true);
                }
            }
        }

        let cx = unsafe { (*screen).cursor.x as usize };
        let right_limit = if cx > sr_right { cols } else { sr_right + 1 };

        unsafe {
            (*(*screen).cursor.page_row).set_dirty(true);
        }

        match width {
            1 => unsafe {
                let page_cell = (*screen).cursor.page_cell;
                ptr::write(page_cell, Cell::init(c));
            },

            2 => {
                if right_limit.saturating_sub(sr_left as usize) > 1 {
                    let cx_now = unsafe { (*screen).cursor.x as usize };
                    if cx_now == right_limit - 1 {
                        if wraparound {
                            unsafe {
                                let s = &mut *screen;
                                if right_limit == cols {
                                    (*s.cursor.page_row).set_wrap(true);
                                    let mut head = Cell::default();
                                    head.set_wide(Wide::SpacerHead);
                                    ptr::write(s.cursor.page_cell, head);
                                }
                                s.cursor.pending_wrap = false;
                            }
                            self.index();
                            unsafe {
                                (*screen).cursor.x = sr_left;
                            }
                            self.cursor_resync();
                            unsafe {
                                let s = &mut *screen;
                                (*s.cursor.page_row).set_dirty(true);
                                if right_limit == cols {
                                    (*s.cursor.page_row).set_wrap_continuation(true);
                                }
                            }
                        } else {
                            self.previous_char = c;
                            self.has_previous_char = true;
                            return;
                        }
                    }

                    unsafe {
                        let s = &mut *screen;
                        (*s.cursor.page_row).set_dirty(true);
                        let pc = s.cursor.page_cell;
                        let mut wide_cell = Cell::init(c);
                        wide_cell.set_wide(Wide::Wide);
                        ptr::write(pc, wide_cell);

                        s.cursor.x += 1;
                        s.cursor.page_cell = pc.add(1);

                        let mut tail = Cell::default();
                        tail.set_wide(Wide::SpacerTail);
                        ptr::write(s.cursor.page_cell, tail);
                    }
                } else {
                    unsafe {
                        (*(*screen).cursor.page_row).set_dirty(true);
                        let pc = (*screen).cursor.page_cell;
                        ptr::write(pc, Cell::default());
                    }
                }
            }

            _ => {}
        }

        let cx_final = unsafe { (*screen).cursor.x as usize };
        if cx_final == right_limit.saturating_sub(1) {
            unsafe {
                (*screen).cursor.pending_wrap = true;
            }
        } else {
            unsafe {
                let s = &mut *screen;
                s.cursor.x += 1;
                s.cursor.page_cell = s.cursor.page_cell.add(1);
            }
        }

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
        self.index();
        if self.modes.get_by_tag(ModeTag::from_u16(MODE_LINEFEED)) {
            self.carriage_return();
        }
    }

    pub fn index(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }

        unsafe {
            (*screen).cursor.pending_wrap = false;
        }

        let cy = unsafe { (*screen).cursor.y };
        let top = self.scrolling_region.top;
        let bottom = self.scrolling_region.bottom;
        let left_margin = self.scrolling_region.left;
        let right_margin = self.scrolling_region.right;
        let cx = unsafe { (*screen).cursor.x };
        let rows = self.rows;

        if cy < top || cy > bottom {
            if cy < rows.saturating_sub(1) {
                self.cursor_down(1);
            }
            return;
        }

        if cy == bottom && cx >= left_margin && cx <= right_margin {
            self.scroll_up(1);
        } else if cy < bottom {
            self.cursor_down(1);
        }
    }

    pub fn backspace(&mut self) {
        self.cursor_left(1);
    }

    pub fn delete_chars(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let screen = self.active();
        if screen.is_null() {
            return;
        }

        self.cursor_resync();

        let cx = unsafe { (*screen).cursor.x as usize };
        let left = self.scrolling_region.left as usize;
        let right = self.scrolling_region.right as usize;

        if cx < left || cx > right {
            return;
        }

        let rem = right + 1 - cx;
        let n = if count > rem { rem } else { count };
        let scroll_amount = rem - n;

        unsafe {
            let page_cell = (*screen).cursor.page_cell;
            let blank = Cell::default();

            if scroll_amount > 0 {
                for i in 0..scroll_amount {
                    let src = page_cell.add(i + n);
                    let dst = page_cell.add(i);
                    let cell = ptr::read(src);
                    ptr::write(dst, cell);
                }
            }

            for i in scroll_amount..rem {
                ptr::write(page_cell.add(i), blank);
            }

            (*(*screen).cursor.page_row).set_dirty(true);
            (*screen).cursor.pending_wrap = false;
        }
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
