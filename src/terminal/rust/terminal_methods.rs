use core::ffi::c_void;
use core::mem;
use core::ptr;

use crate::allocator::{alloc_free_impl, GhosttyAllocator};
use crate::ansi::{ProtectedMode, StatusDisplay};
use crate::csi::{EraseDisplay, EraseLine};
use crate::mode_def::ModeTag;
use crate::page_list_types::PageListNode;
use crate::point::PointTag;
use crate::screen_methods::pin_row_and_cell;
use crate::screen_set::ScreenKey;
use crate::screen_types::Screen;
use crate::size_types::CellCountInt;
#[cfg(ghostty_vt_terminal_owned)]
use crate::terminal_byte_list::{byte_list_from_void, ByteList};
use crate::terminal_types::{
    SwitchScreenMode, Terminal, TerminalFlags, TerminalOptions, TerminalScrollingRegion,
    TABSTOP_INTERVAL,
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

    #[cfg(ghostty_vt_terminal_owned)]
    pub unsafe fn set_title_slice(&mut self, alloc: *const GhosttyAllocator, title: &[u8]) -> bool {
        if self.title.is_null() {
            return false;
        }
        unsafe { ByteList::set_slice(alloc, byte_list_from_void(self.title), title) }
    }

    #[cfg(ghostty_vt_terminal_owned)]
    pub unsafe fn set_pwd_slice(&mut self, alloc: *const GhosttyAllocator, pwd: &[u8]) -> bool {
        if self.pwd.is_null() {
            return false;
        }
        unsafe { ByteList::set_slice(alloc, byte_list_from_void(self.pwd), pwd) }
    }

    #[cfg(ghostty_vt_terminal_owned)]
    pub unsafe fn get_title_slice(&self) -> Option<&[u8]> {
        if self.title.is_null() {
            return None;
        }
        unsafe { ByteList::as_cstr_slice(byte_list_from_void(self.title)) }
    }

    #[cfg(ghostty_vt_terminal_owned)]
    pub unsafe fn get_pwd_slice(&self) -> Option<&[u8]> {
        if self.pwd.is_null() {
            return None;
        }
        unsafe { ByteList::as_cstr_slice(byte_list_from_void(self.pwd)) }
    }

    fn cursor_resync(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            let s = &mut *screen;
            if s.cursor.page_pin.is_null() || s.pages.is_null() {
                return;
            }
            let pages = &*s.pages;
            let Some(target) = pages.pin(PointTag::ACTIVE, s.cursor.x, s.cursor.y as u32) else {
                return;
            };
            let cur = *s.cursor.page_pin;
            if cur.node != target.node || cur.y != target.y {
                s.cursor_change_pin(target);
            } else {
                (*s.cursor.page_pin).x = s.cursor.x;
            }
            let (row, cell) = pin_row_and_cell(s.cursor.page_pin);
            s.cursor.page_row = row;
            s.cursor.page_cell = cell;
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
            s.cursor_absolute(left, top);
        }

        self.insert_lines(count);

        unsafe {
            let s = &mut *screen;
            s.cursor.pending_wrap = old_wrap;
            s.cursor_absolute(old_x, old_y);
        }
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

        if self.scrolling_region.top == 0
            && self.scrolling_region.left == 0
            && self.scrolling_region.right == self.cols - 1
        {
            let region_height = (self.scrolling_region.bottom + 1) as usize;
            let adjusted = count.min(region_height);

            unsafe {
                let s = &mut *screen;
                s.cursor.x = 0;
                s.cursor.y = self.scrolling_region.bottom;
            }
            self.cursor_resync();

            for _ in 0..adjusted {
                unsafe {
                    let _ = (*screen).cursor_scroll_above();
                }
            }

            unsafe {
                let s = &mut *screen;
                s.cursor.pending_wrap = old_wrap;
                s.cursor_absolute(old_x, old_y);
            }
            return;
        }

        let top = self.scrolling_region.top;
        let left = self.scrolling_region.left;

        unsafe {
            let s = &mut *screen;
            s.cursor_absolute(left, top);
        }

        self.delete_lines(count);

        unsafe {
            let s = &mut *screen;
            s.cursor.pending_wrap = old_wrap;
            s.cursor_absolute(old_x, old_y);
        }
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
        let adjusted = if count > region_height {
            region_height
        } else {
            count
        };
        self.cursor_resync();
        unsafe {
            let sref = &*screen;
            if sref.cursor.page_pin.is_null() {
                return;
            }
            let node = (*sref.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page: &mut crate::page_core::Page = &mut (*node).data;
            let page_row_count = page.size.rows as usize;
            if (cy as usize) >= page_row_count
                || (self.scrolling_region.bottom as usize) >= page_row_count
            {
                return;
            }
            let rows_base = page.rows_ptr();
            let sr_left = self.scrolling_region.left as usize;
            let width = (self.scrolling_region.right as usize + 1) - sr_left;
            let mut y = self.scrolling_region.bottom as usize;
            while y >= (cy as usize) + adjusted {
                let src_row = rows_base.add(y - adjusted);
                let dst_row = rows_base.add(y);
                page.move_cells(src_row, sr_left, dst_row, sr_left, width);
                (*dst_row).set_wrap(false);
                (*dst_row).set_wrap_continuation(false);
                (*src_row).set_wrap(false);
                (*src_row).set_wrap_continuation(false);
                (*dst_row).set_dirty(true);
                if y == 0 {
                    break;
                }
                y -= 1;
            }
            let mut y2 = cy as usize;
            let y_end = (cy as usize) + adjusted;
            while y2 < y_end {
                let row = rows_base.add(y2);
                page.clear_cells(row, sr_left, sr_left + width);
                (*row).set_wrap(false);
                (*row).set_wrap_continuation(false);
                (*row).set_dirty(true);
                y2 += 1;
            }
            let s_mut = &mut *screen;
            s_mut.cursor.x = self.scrolling_region.left;
            s_mut.cursor.pending_wrap = false;
        }
        self.cursor_resync();
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
        let adjusted = if count > region_height {
            region_height
        } else {
            count
        };
        self.cursor_resync();
        unsafe {
            let sref = &*screen;
            if sref.cursor.page_pin.is_null() {
                return;
            }
            let node = (*sref.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page: &mut crate::page_core::Page = &mut (*node).data;
            let page_row_count = page.size.rows as usize;
            if (cy as usize) >= page_row_count
                || (self.scrolling_region.bottom as usize) >= page_row_count
            {
                return;
            }
            let rows_base = page.rows_ptr();
            let sr_left = self.scrolling_region.left as usize;
            let width = (self.scrolling_region.right as usize + 1) - sr_left;
            let cy_u = cy as usize;
            let bottom_u = self.scrolling_region.bottom as usize;
            let mut y = cy_u;
            while y + adjusted <= bottom_u {
                let src_row = rows_base.add(y + adjusted);
                let dst_row = rows_base.add(y);
                page.move_cells(src_row, sr_left, dst_row, sr_left, width);
                (*dst_row).set_wrap(false);
                (*dst_row).set_wrap_continuation(false);
                (*src_row).set_wrap(false);
                (*src_row).set_wrap_continuation(false);
                (*dst_row).set_dirty(true);
                y += 1;
            }
            let clear_start = if bottom_u + 1 > adjusted {
                bottom_u + 1 - adjusted
            } else {
                0
            };
            let mut y2 = clear_start;
            while y2 <= bottom_u {
                let row = rows_base.add(y2);
                page.clear_cells(row, sr_left, sr_left + width);
                (*row).set_wrap(false);
                (*row).set_wrap_continuation(false);
                (*row).set_dirty(true);
                y2 += 1;
            }
            let s_mut = &mut *screen;
            s_mut.cursor.x = self.scrolling_region.left;
            s_mut.cursor.pending_wrap = false;
        }
        self.cursor_resync();
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
            unsafe {
                let page_cell = (*screen).cursor.page_cell;
                let cells_start = page_cell.sub(cx_usize);
                let row = (*screen).cursor.page_row;
                let node = (*screen).cursor.page_pin;
                if node.is_null() || row.is_null() {
                    return;
                }
                let node = (*node).node;
                if node.is_null() {
                    return;
                }
                let page = &mut (*node).data;
                let cells = cells_start.add(start);
                if protected {
                    (*screen).clear_unprotected_cells(page, row, cells, end - start);
                } else {
                    (*screen).clear_cells(page, row, cells, end - start);
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
                let screen = self.active();
                if !screen.is_null() {
                    let pm = unsafe { (*screen).protected_mode };
                    let protected = pm == ProtectedMode::ISO || protected_req;
                    unsafe {
                        (*screen).clear_rows(0, None, protected);
                    }
                }
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
                    self.cursor_resync();
                    let blank = unsafe { (*screen).blank_cell() };
                    let pm = unsafe { (*screen).protected_mode };
                    let protected = pm == ProtectedMode::ISO || protected_req;
                    let cols = self.cols as usize;
                    unsafe {
                        let sref = &*screen;
                        if sref.cursor.page_pin.is_null() {
                            return;
                        }
                        let node = (*sref.cursor.page_pin).node;
                        if node.is_null() {
                            return;
                        }
                        let page: &mut crate::page_core::Page = &mut (*node).data;
                        let page_row_count = page.size.rows as usize;
                        let mut y = (cy as usize) + 1;
                        while y < self.rows as usize && y < page_row_count {
                            let row = page.rows_ptr().add(y);
                            let cells = page.row_cells_ptr(row);
                            if protected {
                                let mut i = 0usize;
                                while i < cols {
                                    let c = cells.add(i);
                                    if !(*c).protected() {
                                        ptr::write(c, blank);
                                    }
                                    i += 1;
                                }
                            } else {
                                page.clear_cells(row, 0, cols);
                            }
                            (*row).set_dirty(true);
                            y += 1;
                        }
                    }
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
                    self.cursor_resync();
                    let blank = unsafe { (*screen).blank_cell() };
                    let pm = unsafe { (*screen).protected_mode };
                    let protected = pm == ProtectedMode::ISO || protected_req;
                    let cols = self.cols as usize;
                    unsafe {
                        let sref = &*screen;
                        if sref.cursor.page_pin.is_null() {
                            return;
                        }
                        let node = (*sref.cursor.page_pin).node;
                        if node.is_null() {
                            return;
                        }
                        let page: &mut crate::page_core::Page = &mut (*node).data;
                        let mut y = 0usize;
                        while y < cy as usize {
                            let row = page.rows_ptr().add(y);
                            let cells = page.row_cells_ptr(row);
                            if protected {
                                let mut i = 0usize;
                                while i < cols {
                                    let c = cells.add(i);
                                    if !(*c).protected() {
                                        ptr::write(c, blank);
                                    }
                                    i += 1;
                                }
                            } else {
                                page.clear_cells(row, 0, cols);
                            }
                            (*row).set_dirty(true);
                            y += 1;
                        }
                    }
                }
            }
            EraseDisplay::Scrollback => {
                let screen = self.active();
                if screen.is_null() {
                    return;
                }
                unsafe {
                    (*screen).erase_history(None);
                }
            }
            EraseDisplay::ScrollComplete => {
                self.erase_display(EraseDisplay::Complete, protected_req);
                let screen = self.active();
                if !screen.is_null() {
                    unsafe {
                        (*screen).erase_history(None);
                    }
                }
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
        let old_rows = self.rows;
        if self.cols != cols && !alloc.is_null() {
            unsafe {
                self.tabstops.deinit(alloc);
            }
            if let Some(ts) = unsafe {
                crate::tabstops::Tabstops::init(alloc, cols as usize, TABSTOP_INTERVAL as usize)
            } {
                self.tabstops = ts;
            }
        }
        let wraparound = self.modes.get_by_tag(ModeTag::from_u16(MODE_WRAPAROUND));
        let prompt_redraw = if old_rows != rows {
            crate::screen_types::PromptRedraw::True
        } else {
            crate::screen_types::PromptRedraw::False
        };
        let primary = self.screens.get(ScreenKey::Primary);
        if !primary.is_null() {
            let opts = crate::screen_types::ScreenResize {
                cols,
                rows,
                reflow: wraparound,
                prompt_redraw,
            };
            unsafe {
                let _ = (*primary.cast::<Screen>()).resize(opts);
            }
        }
        let alternate = self.screens.get(ScreenKey::Alternate);
        if !alternate.is_null() {
            let alt_opts = crate::screen_types::ScreenResize {
                cols,
                rows,
                reflow: false,
                prompt_redraw: crate::screen_types::PromptRedraw::False,
            };
            unsafe {
                let _ = (*alternate.cast::<Screen>()).resize(alt_opts);
            }
        }
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

    #[cfg(ghostty_vt_terminal_owned)]
    pub fn switch_screen(&mut self, key: ScreenKey) -> Option<*mut Screen> {
        if self.screens.active_key == key {
            return None;
        }
        let alloc = self.bootstrap_alloc;
        if alloc.is_null() {
            return None;
        }

        let old = self.active();
        if !old.is_null() {
            unsafe {
                (*old).end_hyperlink();
            }
        }

        let primary = self.screens.get(ScreenKey::Primary);
        let primary_scrollback = if primary.is_null() {
            0
        } else {
            unsafe {
                let pages = (*primary.cast::<Screen>()).pages;
                if pages.is_null() {
                    0
                } else {
                    (*pages).explicit_max_size
                }
            }
        };

        let new = unsafe {
            self.screens
                .get_or_init_screen(alloc, key, self.cols, self.rows, primary_scrollback)?
        };

        unsafe {
            (*new).clear_selection();
            if !old.is_null() {
                let old_charset = ptr::read(&(*old).charset);
                (*new).charset = old_charset.clone_charset_state();
            }
        }

        self.flags.dirty.clear = true;
        self.screens.switch_to(key);
        Some(old)
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    pub fn switch_screen(&mut self, key: ScreenKey) -> Option<*mut Screen> {
        if self.screens.active_key == key {
            return None;
        }
        let new = self.screens.get(key);
        if new.is_null() {
            return None;
        }
        let old = self.active();
        self.flags.dirty.clear = true;
        self.screens.switch_to(key);
        Some(old)
    }

    pub fn switch_screen_mode(&mut self, mode: SwitchScreenMode, enabled: bool) {
        match mode {
            SwitchScreenMode::Mode47 => {}
            SwitchScreenMode::Mode1047 => {
                if !enabled && self.screens.active_key == ScreenKey::Alternate {
                    self.erase_display(EraseDisplay::Complete, false);
                }
            }
            SwitchScreenMode::Mode1049 => {
                if enabled {
                    self.save_cursor();
                }
            }
        }

        let to = if enabled {
            ScreenKey::Alternate
        } else {
            ScreenKey::Primary
        };
        let old = self.switch_screen(to);

        match mode {
            SwitchScreenMode::Mode47 | SwitchScreenMode::Mode1047 => {
                if let Some(old_screen) = old {
                    if !old_screen.is_null() {
                        let active = self.active();
                        if !active.is_null() {
                            unsafe {
                                let other_cursor = ptr::read(&(*old_screen).cursor);
                                let _ = (*active).cursor_copy(other_cursor, false);
                            }
                        }
                    }
                }
            }
            SwitchScreenMode::Mode1049 => {
                if enabled {
                    debug_assert!(self.screens.active_key == ScreenKey::Alternate);
                    self.erase_display(EraseDisplay::Complete, false);
                    if let Some(old_screen) = old {
                        if !old_screen.is_null() {
                            let active = self.active();
                            if !active.is_null() {
                                unsafe {
                                    let other_cursor = ptr::read(&(*old_screen).cursor);
                                    let _ = (*active).cursor_copy(other_cursor, false);
                                }
                            }
                        }
                    }
                } else {
                    debug_assert!(self.screens.active_key == ScreenKey::Primary);
                    self.restore_cursor();
                }
            }
        }
    }

    pub fn save_cursor(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            (*screen).cursor_save();
        }
    }

    pub fn restore_cursor(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            (*screen).cursor_restore();
        }
    }

    #[cfg(ghostty_vt_terminal_owned)]
    pub fn full_reset(&mut self) {
        self.screens.switch_to(ScreenKey::Primary);
        let alloc = self.bootstrap_alloc;
        let alt = self.screens.get(ScreenKey::Alternate);
        if !alt.is_null() && !alloc.is_null() {
            unsafe {
                (*alt.cast::<Screen>()).bootstrap_deinit(alloc);
                alloc_free_impl(alloc, alt as *mut u8, mem::size_of::<Screen>());
            }
            self.screens.remove_screen(ScreenKey::Alternate);
        }
        self.full_reset_common();
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    pub fn full_reset(&mut self) {
        self.screens.switch_to(ScreenKey::Primary);
        self.full_reset_common();
    }

    fn full_reset_common(&mut self) {
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
        // Clear pwd and title buffers on the Rust-owned bootstrap path.
        #[cfg(ghostty_vt_terminal_owned)]
        unsafe {
            if !self.pwd.is_null() {
                ByteList::clear_retaining_capacity(byte_list_from_void(self.pwd));
            }
            if !self.title.is_null() {
                ByteList::clear_retaining_capacity(byte_list_from_void(self.title));
            }
        }
        #[cfg(not(ghostty_vt_terminal_owned))]
        {
            #[repr(C)]
            struct ArrayListLayout {
                _items_ptr: *mut u8,
                items_len: usize,
            }
            if !self.pwd.is_null() {
                unsafe {
                    let list = &mut *self.pwd.cast::<ArrayListLayout>();
                    list.items_len = 0;
                }
            }
            if !self.title.is_null() {
                unsafe {
                    let list = &mut *self.title.cast::<ArrayListLayout>();
                    list.items_len = 0;
                }
            }
        }
        let primary = self.screens.get(ScreenKey::Primary);
        if !primary.is_null() {
            unsafe {
                (*primary.cast::<Screen>()).reset();
            }
            self.screens.bump_generation(ScreenKey::Primary);
        }
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
                (*screen).write_cell(c);
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

    pub fn write(&mut self, data: &[u8]) {
        use crate::stream_core::Stream;
        use crate::stream_terminal::StreamTerminal;

        let terminal_ptr = self as *mut Terminal as *mut core::ffi::c_void;
        let handler = StreamTerminal::new(terminal_ptr);
        let mut stream = Stream::new(handler);
        let len = data.len();
        let mut i = 0;
        unsafe {
            while i < len {
                stream.next(*data.get_unchecked(i));
                i += 1;
            }
        }
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
            if self.scrolling_region.top == 0
                && self.scrolling_region.left == 0
                && self.scrolling_region.right == self.cols - 1
            {
                unsafe {
                    let _ = (*screen).cursor_scroll_above();
                }
            } else {
                self.scroll_up(1);
            }
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
        self.cursor_resync();
        let cx = unsafe { (*screen).cursor.x as usize };
        let cols = self.cols as usize;
        let count_eff = if count == 0 { 1 } else { count };
        let end = if cx + count_eff > cols {
            cols
        } else {
            cx + count_eff
        };
        if cx >= end {
            return;
        }
        let len = end - cx;
        self.cursor_resync();
        unsafe {
            let sref = &*screen;
            if sref.cursor.page_pin.is_null() {
                return;
            }
            let node = (*sref.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page: &mut crate::page_core::Page = &mut (*node).data;
            let row = sref.cursor.page_row;
            let cells = page.row_cells_ptr(row).add(cx);
            let pm = sref.protected_mode;
            let protected = pm == ProtectedMode::ISO;
            let blank = sref.blank_cell();
            if protected {
                let mut i = 0usize;
                while i < len {
                    let c = cells.add(i);
                    if !(*c).protected() {
                        ptr::write(c, blank);
                    }
                    i += 1;
                }
            } else {
                page.clear_cells(row, cx, cx + len);
            }
            (*row).set_dirty(true);
        }
    }
}
