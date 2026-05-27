#![allow(unused)]

use core::ffi::c_void;
use core::ptr;

use crate::early::*;
use crate::constants::*;
use crate::size_types::*;
use crate::page_types::*;
use crate::page_core::Page;
use crate::page_list_types::{PageList, PageListNode, PageListViewport, PageListDirection};
use crate::page_list_methods::{PageListScroll, PromptIterator, RowIterator};
use crate::point::PointTag;
use crate::style_types::*;
use crate::ansi::*;
use crate::highlight::Pin;
use crate::kitty_key::KittyKeyFlagStack;
use crate::ref_counted_set::{AddError, RefCountedSetContext};
use crate::sgr_attribute::{Attribute, Name};
use crate::screen_types::*;
use crate::selection_types::*;
use crate::allocator::{GhosttyAllocator, alloc_free_impl};
use crate::hyperlink::Hyperlink;
use core::mem;

pub struct StyleContext;

impl RefCountedSetContext<Style> for StyleContext {
    #[inline]
    fn hash(value: &Style) -> u64 {
        value.hash()
    }
    #[inline]
    fn eql(a: &Style, b: &Style) -> bool {
        a.eql(b)
    }
}

/// Errors that can occur when resizing the screen.
pub enum ScreenResizeError {
    OutOfMemory,
}

/// Errors that can occur when increasing page capacity from the screen.
pub enum ScreenIncreaseCapacityError {
    OutOfMemory,
    OutOfSpace,
}

/// Errors that can occur when setting an attribute that requires style updates.
pub enum ScreenStyleError {
    OutOfMemory,
    OutOfSpace,
}

impl Screen {
    #[inline]
    fn ghostty_alloc(&self) -> *const GhosttyAllocator {
        &self.alloc
    }

    unsafe fn hyperlink_deinit(&mut self, link: *mut Hyperlink) {
        if link.is_null() {
            return;
        }
        let alloc = self.ghostty_alloc();
        if alloc.is_null() {
            return;
        }
        unsafe {
            let uri_ptr = (*link).uri_ptr;
            let uri_len = (*link).uri_len;
            if !uri_ptr.is_null() && uri_len > 0 {
                alloc_free_impl(alloc, uri_ptr as *mut u8, uri_len);
            }
            alloc_free_impl(alloc, link as *mut u8, mem::size_of::<Hyperlink>());
        }
    }

    fn page_cols(&self) -> CellCountInt {
        if self.pages.is_null() {
            return 0;
        }
        unsafe {
            let node = self.cursor_page_node();
            if node.is_null() {
                return 0;
            }
            (*node).data.size.cols
        }
    }

    fn page_rows(&self) -> CellCountInt {
        if self.pages.is_null() {
            return 0;
        }
        unsafe {
            let node = self.cursor_page_node();
            if node.is_null() {
                return 0;
            }
            (*node).data.size.rows
        }
    }

    /// Returns the page node backing the cursor pin.
    fn cursor_page_node(&self) -> *mut crate::page_list_types::PageListNode {
        if self.cursor.page_pin.is_null() {
            return ptr::null_mut();
        }
        unsafe { (*self.cursor.page_pin).node }
    }

    #[inline]
    pub fn cursor_mark_dirty(&mut self) {
        if !self.cursor.page_row.is_null() {
            unsafe {
                (*self.cursor.page_row).set_dirty(true);
            }
        }
    }

    /// Returns the blank cell to use when doing terminal operations that
    /// require preserving the bg color.
    #[inline]
    pub fn blank_cell(&self) -> Cell {
        if self.cursor.style_id == DEFAULT_ID {
            return Cell::default();
        }
        match self.cursor.style.bg_color {
            Color::None => Cell::default(),
            Color::Palette(idx) => {
                let mut c = Cell::default();
                c.set_content_tag(ContentTag::BgColorPalette);
                c.set_content_codepoint(idx as u32);
                c
            }
            Color::Rgb(rgb) => {
                let mut c = Cell::default();
                c.set_content_tag(ContentTag::BgColorRgb);
                c.set_content_codepoint(((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32));
                c
            }
        }
    }

    /// Move the cursor right. This is a specialized function that is very fast
    /// if the caller can guarantee we have space to move right (no wrapping).
    ///
    /// Safety: cursor.page_cell must point into a valid cell array, and
    /// cursor.x + n must be less than the page column count.
    pub unsafe fn cursor_right(&mut self, n: CellCountInt) {
        unsafe {
            let cols = (*self.cursor.page_pin).node;
            let page_cols = if cols.is_null() { 0 } else { (*cols).data.size.cols };
            assert!(self.cursor.x + n < page_cols);

            let cell = self.cursor.page_cell;
            self.cursor.page_cell = cell.add(n as usize);
            (*self.cursor.page_pin).x += n;
            self.cursor.x += n;
        }
    }

    /// Move the cursor left.
    ///
    /// Safety: cursor.page_cell must point into a valid cell array, and
    /// cursor.x must be >= n.
    pub unsafe fn cursor_left(&mut self, n: CellCountInt) {
        unsafe {
            assert!(self.cursor.x >= n);

            let cell = self.cursor.page_cell;
            self.cursor.page_cell = cell.sub(n as usize);
            (*self.cursor.page_pin).x -= n;
            self.cursor.x -= n;
        }
    }

    /// Returns a pointer to the cell n positions to the right of the cursor.
    ///
    /// Safety: cursor.page_cell must be valid and cursor.x + n < page.cols.
    pub unsafe fn cursor_cell_right(&self, n: CellCountInt) -> *mut Cell {
        unsafe {
            let cols_node = (*self.cursor.page_pin).node;
            let page_cols = if cols_node.is_null() { 0 } else { (*cols_node).data.size.cols };
            assert!(self.cursor.x + n < page_cols);
            self.cursor.page_cell.add(n as usize)
        }
    }

    /// Returns a pointer to the cell n positions to the left of the cursor.
    ///
    /// Safety: cursor.page_cell must be valid and cursor.x >= n.
    pub unsafe fn cursor_cell_left(&self, n: CellCountInt) -> *mut Cell {
        unsafe {
            assert!(self.cursor.x >= n);
            self.cursor.page_cell.sub(n as usize)
        }
    }

    /// Move the cursor up by n rows.
    ///
    /// Precondition: the cursor is not at the top of the screen, i.e. cursor.y >= n.
    ///
    /// Safety: pin.up() must return a valid pin.
    pub unsafe fn cursor_up(&mut self, n: CellCountInt) {
        unsafe {
            assert!(self.cursor.y >= n);

            self.cursor.y -= n;
            let old_pin = *self.cursor.page_pin;
            let new_pin = pin_up(old_pin, n);
            self.cursor_change_pin(new_pin);
            let row_cell = pin_row_and_cell(self.cursor.page_pin);
            self.cursor.page_row = row_cell.0;
            self.cursor.page_cell = row_cell.1;
        }
    }

    /// Move the cursor down by n rows.
    ///
    /// Precondition: the cursor is not at the bottom of the screen, i.e.
    /// cursor.y + n < pages.rows.
    ///
    /// Safety: pin.down() must return a valid pin.
    pub unsafe fn cursor_down(&mut self, n: CellCountInt) {
        unsafe {
            let rows_node = (*self.cursor.page_pin).node;
            let page_rows = if rows_node.is_null() { 0 } else { (*rows_node).data.size.rows };
            assert!(self.cursor.y + n < page_rows);

            self.cursor.y += n;
            let old_pin = *self.cursor.page_pin;
            let new_pin = pin_down(old_pin, n);
            self.cursor_change_pin(new_pin);
            let row_cell = pin_row_and_cell(self.cursor.page_pin);
            self.cursor.page_row = row_cell.0;
            self.cursor.page_cell = row_cell.1;
        }
    }

    /// Returns a pointer to the row `n` rows above the cursor.
    ///
    /// Safety: pin.up() must return a valid pin.
    pub unsafe fn cursor_row_up(&self, n: CellCountInt) -> *mut Row {
        unsafe {
            assert!(self.cursor.y >= n);
            let old_pin = *self.cursor.page_pin;
            let new_pin = pin_up(old_pin, n);
            let row_cell = pin_row_and_cell_raw(new_pin);
            row_cell.0
        }
    }

    /// Move the cursor to some absolute horizontal position.
    ///
    /// Safety: x must be less than pages.cols.
    pub unsafe fn cursor_horizontal_absolute(&mut self, x: CellCountInt) {
        unsafe {
            let cols_node = (*self.cursor.page_pin).node;
            let page_cols = if cols_node.is_null() { 0 } else { (*cols_node).data.size.cols };
            assert!(x < page_cols);

            (*self.cursor.page_pin).x = x;
            let row_cell = pin_row_and_cell(self.cursor.page_pin);
            self.cursor.page_cell = row_cell.1;
            self.cursor.x = x;
        }
    }

    /// Move the cursor to some absolute (x, y) position.
    ///
    /// Safety: x must be less than pages.cols, y must be less than pages.rows.
    pub unsafe fn cursor_absolute(&mut self, x: CellCountInt, y: CellCountInt) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            let page = if node.is_null() {
                return;
            } else {
                &(*node).data
            };
            assert!(x < page.size.cols);
            assert!(y < page.size.rows);

            let mut new_pin = if y < self.cursor.y {
                pin_up(*self.cursor.page_pin, self.cursor.y - y)
            } else if y > self.cursor.y {
                pin_down(*self.cursor.page_pin, y - self.cursor.y)
            } else {
                *self.cursor.page_pin
            };
            new_pin.x = x;
            self.cursor.x = x;
            self.cursor.y = y;
            self.cursor_change_pin(new_pin);
            let row_cell = pin_row_and_cell(self.cursor.page_pin);
            self.cursor.page_row = row_cell.0;
            self.cursor.page_cell = row_cell.1;
        }
    }

    /// Always use this to write to cursor.page_pin.*.
    ///
    /// This handles the case when the new pin is on a different page than
    /// the old AND we have a style or hyperlink set: in that case we must
    /// release the old one and insert the new one, since styles are stored
    /// per-page.
    ///
    /// Safety: the new pin must be a valid tracked pin.
    pub unsafe fn cursor_change_pin(&mut self, new: Pin) {
        unsafe {
            if !self.cursor.page_pin.is_null() && !ptr::read(self.cursor.page_pin).eql(new) {
                self.cursor_mark_dirty();
                if !new.node.is_null() {
                    (*new.node).data.dirty = true;
                }
            }

            if self.cursor.page_pin.is_null() {
                let pin_ptr = self.cursor.page_pin;
                ptr::write(pin_ptr, new);
                return;
            }

            if (*self.cursor.page_pin).node == new.node {
                ptr::write(self.cursor.page_pin, new);
                return;
            }

            let old_style: Option<Style> = if self.cursor.style_id == DEFAULT_ID {
                None
            } else {
                Some(self.cursor.style)
            };

            if old_style.is_some() {
                self.cursor.style = Style::default();
                self.cursor.style_id = DEFAULT_ID;
            }

            ptr::write(self.cursor.page_pin, new);

            if let Some(old_s) = old_style {
                self.cursor.style = old_s;
                let _ = self.manual_style_update();
            }
        }
    }

    fn cursor_page_pin_eql(&self, other: Pin) -> bool {
        if self.cursor.page_pin.is_null() {
            return false;
        }
        unsafe { (*self.cursor.page_pin).eql(other) }
    }

    /// Reloads the cursor pointer information into the screen. This is expensive
    /// so it should only be done in cases where the pointers are invalidated
    /// in such a way that it's difficult to recover otherwise.
    ///
    /// Safety: screen must be fully initialized with valid pages and cursor.
    pub unsafe fn cursor_reload(&mut self) {
        unsafe {
            let pt = self.point_from_pin_active(*self.cursor.page_pin);
            let (x, y) = match pt {
                Some(xy) => xy,
                None => {
                    let pin = self.pin_active_origin();
                    ptr::write(self.cursor.page_pin, pin);
                    self.point_from_pin_active(pin).unwrap_or((0, 0))
                }
            };

            self.cursor.x = x as CellCountInt;
            self.cursor.y = y as CellCountInt;
            let row_cell = pin_row_and_cell(self.cursor.page_pin);
            self.cursor.page_row = row_cell.0;
            self.cursor.page_cell = row_cell.1;

            if self.cursor.style_id != DEFAULT_ID {
                let _ = self.manual_style_update();
            }
        }
    }

    /// Reset the cursor row's soft-wrap state and the cursor's pending wrap.
    ///
    /// Safety: cursor.page_row and cursor.page_pin must be valid.
    pub unsafe fn cursor_reset_wrap(&mut self) {
        unsafe {
            self.cursor.pending_wrap = false;

            let page_row = self.cursor.page_row;
            if page_row.is_null() {
                return;
            }

            if !(*page_row).wrap() {
                return;
            }

            (*page_row).set_wrap(false);

            if let Some(next_pin) = pin_down_safe(*self.cursor.page_pin, 1) {
                let row_cell = pin_row_and_cell_raw(next_pin);
                (*row_cell.0).set_wrap_continuation(false);
            }

            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let cols = (*node).data.size.cols;
            let cells = (*node).data.row_cells_ptr(page_row);
            let last_cell = cells.add((cols as usize) - 1);
            if (*last_cell).wide() == Wide::SpacerHead {
                self.clear_cells(&mut (*node).data, page_row, last_cell, 1);
            }
        }
    }

    /// Modify the semantic content type of the cursor. This should be
    /// preferred over setting it manually since it handles all the proper
    /// accounting.
    pub fn cursor_set_semantic_content_prompt(&mut self, is_continuation: bool) {
        self.semantic_prompt.seen = true;
        self.cursor.semantic_content = SemanticContent::Prompt;
        self.cursor.semantic_content_clear_eol = false;
        if !self.cursor.page_row.is_null() {
            unsafe {
                (*self.cursor.page_row).set_semantic_prompt(if is_continuation {
                    SemanticPrompt::PromptContinuation
                } else {
                    SemanticPrompt::Prompt
                });
            }
        }
    }

    pub fn cursor_set_semantic_content_input(&mut self, clear_eol: bool) {
        self.cursor.semantic_content = SemanticContent::Input;
        self.cursor.semantic_content_clear_eol = clear_eol;
    }

    pub fn cursor_set_semantic_content_output(&mut self) {
        self.cursor.semantic_content = SemanticContent::Output;
        self.cursor.semantic_content_clear_eol = false;
    }

    /// Clear the selection, if any, and mark the dirty flag.
    pub fn clear_selection(&mut self) {
        if let Some(sel) = self.selection.take() {
            sel.deinit(self.pages);
            self.dirty.selection = true;
        }
    }

    /// Scroll the viewport of the terminal grid.
    pub fn scroll(&mut self, behavior: ScreenScroll) {
        self.kitty_images_dirty_set();
        if self.pages.is_null() {
            return;
        }
        unsafe {
            let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
            let pl_beh = match behavior {
                ScreenScroll::Active => PageListScroll::Active,
                ScreenScroll::Top => PageListScroll::Top,
                ScreenScroll::Pin(p) => PageListScroll::Pin(p),
                ScreenScroll::Row(v) => PageListScroll::Row(v),
                ScreenScroll::DeltaRow(v) => PageListScroll::DeltaRow(v),
                ScreenScroll::DeltaPrompt(v) => PageListScroll::DeltaPrompt(v),
            };
            pl.scroll(pl_beh);
        }
    }

    /// Returns true if the viewport is scrolled to the bottom of the screen.
    pub fn viewport_is_bottom(&self) -> bool {
        if self.pages.is_null() {
            return true;
        }
        unsafe {
            let pl: &PageList = &*self.pages.cast::<PageList>();
            pl.viewport == PageListViewport::Active
        }
    }

    /// Assert that the screen is in a consistent state.
    pub fn assert_integrity(&self) {
        if cfg!(debug_assertions) && !self.pages.is_null() && !self.cursor.page_pin.is_null() {
            unsafe {
                let pl: &PageList = &*self.pages.cast::<PageList>();
                let pin = *self.cursor.page_pin;
                if let Some((_x, y)) = pl.point_from_pin(PointTag::ACTIVE, pin) {
                    debug_assert!((self.cursor.x as u32) < (pl.cols as u32));
                    debug_assert!(y < (pl.rows as u32));
                }
            }
        }
    }

    /// Clear cells in a row respecting grapheme, hyperlink and style cleanup.
    ///
    /// `cells` is a pointer to the first cell in a contiguous array of `len`
    /// cells within the given `row`.
    ///
    /// Safety: page, row and cells must be valid pointers with consistent layout.
    pub unsafe fn clear_cells(
        &mut self,
        page: &mut Page,
        row: *mut Row,
        cells: *mut Cell,
        len: usize,
    ) {
        unsafe {
            page.pause_integrity_checks(true);

            if (*row).grapheme() {
                let mut i = 0usize;
                while i < len {
                    let cell = cells.add(i);
                    if (*cell).has_grapheme() {
                        page.clear_grapheme(cell);
                    }
                    i += 1;
                }
                if len == page.size.cols as usize {
                    (*row).set_grapheme(false);
                } else {
                    page.update_row_grapheme_flag(row);
                }
            }

            if (*row).hyperlink() {
                let mut i = 0usize;
                while i < len {
                    let cell = cells.add(i);
                    if (*cell).hyperlink() {
                        page.clear_hyperlink(cell);
                    }
                    i += 1;
                }
                if len == page.size.cols as usize {
                    (*row).set_hyperlink(false);
                } else {
                    page.update_row_hyperlink_flag(row);
                }
            }

            if (*row).styled() {
                if len == page.size.cols as usize {
                    (*row).set_styled(false);
                } else {
                    page.update_row_styled_flag(row);
                }
            }

            let blank = self.blank_cell();
            let mut i = 0usize;
            while i < len {
                ptr::write(cells.add(i), blank);
                i += 1;
            }

            page.pause_integrity_checks(false);
            page.assert_integrity();
            self.assert_integrity();
        }
    }

    /// Clear cells but only if they are not protected.
    ///
    /// Safety: page, row and cells must be valid pointers.
    pub unsafe fn clear_unprotected_cells(
        &mut self,
        page: &mut Page,
        row: *mut Row,
        cells: *mut Cell,
        len: usize,
    ) {
        unsafe {
            let mut x0: usize = 0;
            loop {
                while x0 < len && (*cells.add(x0)).protected() {
                    x0 += 1;
                }
                if x0 >= len {
                    break;
                }
                let mut x1 = x0 + 1;
                while x1 < len && !(*cells.add(x1)).protected() {
                    x1 += 1;
                }
                self.clear_cells(page, row, cells.add(x0), x1 - x0);
                x0 = x1;
            }
            page.assert_integrity();
            self.assert_integrity();
        }
    }

    /// Clear the rows in the given range (tl..=bl).
    ///
    /// `tl_y` is the top-left active y, `bl_y` is the bottom-left y (or None for end).
    /// If `protected` is true only unprotected cells are cleared.
    ///
    /// Safety: pages must be initialized; coordinates within bounds.
    pub unsafe fn clear_rows(
        &mut self,
        tl_y: CellCountInt,
        bl_y: Option<CellCountInt>,
        protected: bool,
    ) {
        unsafe {
            if self.pages.is_null() {
                return;
            }
            let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
            let cols = pl.cols;

            let (bl_tag, bl_x, bl_y_val): (Option<PointTag>, Option<CellCountInt>, Option<u32>) =
                match bl_y {
                    Some(y) => (Some(PointTag::ACTIVE), Some(0), Some(y as u32)),
                    None => (None, None, None),
                };

            let mut it = pl.page_iterator(
                PageListDirection::RightDown,
                PointTag::ACTIVE,
                0,
                tl_y as u32,
                bl_tag,
                bl_x,
                bl_y_val,
            );

            while let Some(chunk) = it.next() {
                let node = chunk.node;
                if node.is_null() {
                    continue;
                }
                let page = &mut (*node).data;
                let rows_base = page.rows_ptr();

                let start = chunk.start as usize;
                let end = chunk.end as usize;
                let mut r = start;
                while r < end {
                    let row = rows_base.add(r);
                    if protected {
                        let cells = page.row_cells_ptr(row);
                        self.clear_unprotected_cells(page, row, cells, cols as usize);
                        (*row).set_dirty(true);
                    } else {
                        let cells = page.row_cells_ptr(row);
                        self.clear_cells(page, row, cells, cols as usize);
                        (*row).set_dirty(true);
                    }
                    r += 1;
                }
            }
        }
    }

    /// Erase the region specified by the bottom-left point, inclusive.
    /// This will physically erase the rows meaning the memory will be
    /// reclaimed and other rows will be shifted up.
    pub fn erase_history(&mut self, _bl: Option<(CellCountInt, CellCountInt)>) {
        if !self.pages.is_null() {
            unsafe {
                let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
                let active_tl = pl.get_top_left(PointTag::ACTIVE);
                let history_br = match _bl {
                    Some((x, y)) => Pin {
                        node: active_tl.node,
                        y,
                        x,
                        garbage: false,
                    },
                    None => match pl.get_bottom_right(PointTag::HISTORY) {
                        Some(p) => p,
                        None => {
                            self.cursor_reload();
                            return;
                        }
                    },
                };

                let remove_count = {
                    let mut count: usize = 0;
                    let mut cur = active_tl.node;
                    while !cur.is_null() && cur != history_br.node {
                        count += unsafe { (*cur).data.size.rows as usize };
                        cur = unsafe { (*cur).next };
                    }
                    if !cur.is_null() && cur == history_br.node {
                        count += (history_br.y as usize) + 1;
                    }
                    count
                };

                let mut removed: usize = 0;
                let mut cur = pl.pages.first;
                while !cur.is_null() && removed < remove_count {
                    let node = cur;
                    let next = unsafe { (*node).next };
                    let rows = unsafe { (*node).data.size.rows as usize };

                    if removed + rows <= remove_count {
                        pl.pages.remove(node);
                        pl.total_rows = pl.total_rows.saturating_sub(rows);
                        removed += rows;
                    } else {
                        let keep = (removed + rows) - remove_count;
                        let page = unsafe { &mut (*node).data };
                        let rows_base = page.rows_ptr();
                        let shift = rows - keep;
                        let mut i: usize = 0;
                        while i < keep {
                            unsafe {
                                *rows_base.add(i) = *rows_base.add(i + shift);
                            }
                            i += 1;
                        }
                        page.size.rows = keep as CellCountInt;
                        removed = remove_count;
                    }
                    cur = next;
                }
                pl.fixup_viewport(removed);
            }
        }
        unsafe { self.cursor_reload(); }
    }

    /// Erase active rows starting from y.
    pub fn erase_active(&mut self, y: CellCountInt) {
        if self.pages.is_null() {
            unsafe { self.cursor_reload(); }
            return;
        }
        unsafe {
            let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
            let total_active = pl.rows as usize;
            if (y as usize) >= total_active {
                self.cursor_reload();
                return;
            }
            let remove_count = total_active - (y as usize);

            let start_pin = match pl.pin(PointTag::ACTIVE, 0, y as u32) {
                Some(p) => p,
                None => {
                    self.cursor_reload();
                    return;
                }
            };

            let mut removed: usize = 0;
            let mut cur = start_pin.node;
            let first_y = start_pin.y as usize;

            while !cur.is_null() && removed < remove_count {
                let node = cur;
                let next = (*node).next;
                let rows = (*node).data.size.rows as usize;
                let y_start = if node == start_pin.node { first_y } else { 0 };
                let avail = rows - y_start;

                if removed + avail <= remove_count {
                    if y_start == 0 {
                        pl.pages.remove(node);
                        pl.total_rows = pl.total_rows.saturating_sub(rows);
                        removed += rows;
                    } else {
                        (*node).data.size.rows = y_start as CellCountInt;
                        removed += avail;
                    }
                } else {
                    let keep_from_end = remove_count - removed;
                    let page = &mut (*node).data;
                    let rows_base = page.rows_ptr();
                    let keep_y = y_start + keep_from_end;
                    let leftover = rows - keep_y;
                    let mut i: usize = 0;
                    while i < leftover {
                        *rows_base.add(y_start + i) = *rows_base.add(keep_y + i);
                        i += 1;
                    }
                    page.size.rows = (y_start + leftover) as CellCountInt;
                    removed = remove_count;
                }
                cur = next;
            }
            pl.fixup_viewport(removed);
        }
        unsafe { self.cursor_reload(); }
    }

    /// Resize the screen.
    ///
    /// Safety: the screen must be fully initialized.
    pub unsafe fn resize(&mut self, opts: ScreenResize) -> Result<(), ScreenResizeError> {
        unsafe {
            // Release the cursor style while resizing in case the cursor
            // ends up on a different page.
            let cursor_style = self.cursor.style;
            self.cursor.style = Style::default();
            let _ = self.manual_style_update();

            // If we have a hyperlink, release it from the old page so the
            // resize doesn't invalidate it. Keep the heap-allocated
            // Hyperlink alive so we can re-establish the link on the new page.
            let hyperlink_saved: Option<*mut Hyperlink> = if self.cursor.hyperlink.is_null() {
                None
            } else {
                Some(self.cursor.hyperlink)
            };
            if self.cursor.hyperlink_id != 0 {
                let node = (*self.cursor.page_pin).node;
                if !node.is_null() {
                    let set: *mut crate::ref_counted_set::RefCountedSet =
                        &mut (*node).data.hyperlink_set;
                    (*set).release((*node).data.memory, self.cursor.hyperlink_id);
                }
                self.cursor.hyperlink_id = 0;
                self.cursor.hyperlink = ptr::null_mut();
            }

            // Handle prompt redraw if requested.
            if opts.prompt_redraw != PromptRedraw::False
                && self.cursor.semantic_content != SemanticContent::Output
            {
                match opts.prompt_redraw {
                    PromptRedraw::Last => {
                        let node = (*self.cursor.page_pin).node;
                        if !node.is_null() {
                            let page = &mut (*node).data;
                            let row = self.cursor.page_row;
                            let cells = page.row_cells_ptr(row);
                            self.clear_cells(page, row, cells, page.size.cols as usize);
                        }
                    }
                    PromptRedraw::True => {
                        let cursor_pin = self.cursor.page_pin;
                        if cursor_pin.is_null() {
                            // Cannot do prompt redraw without cursor pin.
                        } else {
                            let cp = unsafe { *cursor_pin };
                            let mut pit = PromptIterator::new(
                                Some(cp),
                                None,
                                PageListDirection::LeftUp,
                            );
                            let start_pin = match pit.next() {
                                Some(p) => p,
                                None => {
                                    // Prompt iterator found no prompt; skip redraw.
                                    cp
                                }
                            };
                            let mut row_it = RowIterator::new_from_pin(
                                start_pin,
                                PageListDirection::RightDown,
                            );
                            while let Some(pin) = row_it.next() {
                                let node = pin.node;
                                if node.is_null() {
                                    break;
                                }
                                unsafe {
                                    let page = &mut (*node).data;
                                    let row = page.get_row(pin.y as usize);
                                    let cells = page.row_cells_ptr(row);
                                    self.clear_cells(page, row, cells, page.size.cols as usize);
                                }
                            }
                        }
                    }
                    PromptRedraw::False => {}
                }
            }

            // Track a pin at the saved cursor position so it survives reflow.
            // track_pin returns null when the underlying pool/tracked-pins
            // infrastructure is not yet available; in that case the saved
            // cursor coordinates stay unchanged after resize.
            let saved_cursor_pin: *mut Pin = if let Some(sc) = self.saved_cursor {
                if !self.pages.is_null() {
                    unsafe {
                        let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
                        let origin = pl.get_top_left(PointTag::ACTIVE);
                        pl.track_pin(Pin {
                            node: origin.node,
                            y: sc.y,
                            x: sc.x,
                            garbage: false,
                        })
                    }
                } else {
                    ptr::null_mut()
                }
            } else {
                ptr::null_mut()
            };

            {
                let pl: &mut PageList = unsafe { &mut *self.pages.cast::<PageList>() };
                let mut pl_resize = crate::page_list_types::PageListResize::default();
                pl_resize.cols = opts.cols;
                pl_resize.rows = opts.rows;
                pl_resize.reflow = opts.reflow;
                pl_resize.cursor_x = self.cursor.x;
                pl_resize.cursor_y = self.cursor.y;
                pl_resize.cursor_pin = self.cursor.page_pin.cast();
                pl_resize.has_cursor = !self.cursor.page_pin.is_null();
                pl.resize(&pl_resize);
            }

            if self.no_scrollback {
                self.erase_history(None);
            }

            self.cursor_reload();

            // Update saved cursor coordinates using the tracked pin.
            if !saved_cursor_pin.is_null() {
                let tracked = unsafe { *saved_cursor_pin };
                let result = self.point_from_pin_active(tracked);

                if let Some(ref mut sc) = self.saved_cursor {
                    if let Some((nx, ny)) = result {
                        sc.x = nx as CellCountInt;
                        sc.y = ny as CellCountInt;
                        if sc.pending_wrap && opts.cols > 0 && sc.x != opts.cols - 1 {
                            sc.pending_wrap = false;
                            sc.x += 1;
                        }
                    } else {
                        sc.x = 0;
                        sc.y = 0;
                        sc.pending_wrap = false;
                    }
                }
                if !self.pages.is_null() {
                    unsafe {
                        let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
                        pl.untrack_pin(saved_cursor_pin);
                    }
                }
            }

            // Restore the cursor style.
            self.cursor.style = cursor_style;
            let _ = self.manual_style_update();

            // Re-add our hyperlink if we had one before resize.
            if let Some(link) = hyperlink_saved {
                if !link.is_null() {
                    let uri_ptr = (*link).uri_ptr;
                    let uri_len = (*link).uri_len;
                    if !uri_ptr.is_null() && uri_len > 0 {
                        let uri_slice = core::slice::from_raw_parts(uri_ptr, uri_len);
                        let _ = self.start_hyperlink(uri_slice, None);
                    }
                    self.hyperlink_deinit(link);
                }
            }
        }
        Ok(())
    }

    /// Update the style stored on the page to match the cursor's current style.
    ///
    /// This function can NOT fail if the cursor style is changing to default.
    ///
    /// Safety: cursor.page_pin must be valid.
    pub unsafe fn manual_style_update(&mut self) -> Result<(), ScreenStyleError> {
        unsafe {
            let pin = self.cursor.page_pin;
            if pin.is_null() {
                return Ok(());
            }
            let node = (*pin).node;
            if node.is_null() {
                return Ok(());
            }
            let page = &mut (*node).data;
            let memory = page.memory;

            if self.cursor.style_id != DEFAULT_ID {
                page.styles.release(memory, self.cursor.style_id);
            }

            if self.cursor.style.is_default() {
                self.cursor.style_id = DEFAULT_ID;
                return Ok(());
            }

            self.cursor.style_id = DEFAULT_ID;

            match page.styles.add::<Style, StyleContext>(memory, self.cursor.style) {
                Ok(id) => {
                    self.cursor.style_id = id;
                    Ok(())
                }
                Err(AddError::OutOfMemory) => {
                    match self.increase_capacity_styles(node) {
                        Ok(()) => {
                            let page = &mut (*node).data;
                            match page.styles.add::<Style, StyleContext>(
                                page.memory,
                                self.cursor.style,
                            ) {
                                Ok(id) => {
                                    self.cursor.style_id = id;
                                    Ok(())
                                }
                                Err(_) => Ok(()),
                            }
                        }
                        Err(_) => Ok(()),
                    }
                }
                Err(AddError::NeedsRehash) => {
                    let page = &mut (*node).data;
                    page.styles.rehash::<Style, StyleContext>(page.memory);
                    self.cursor.style_id = DEFAULT_ID;
                    match page.styles.add::<Style, StyleContext>(
                        page.memory,
                        self.cursor.style,
                    ) {
                        Ok(id) => {
                            self.cursor.style_id = id;
                            Ok(())
                        }
                        Err(_) => Ok(()),
                    }
                }
            }
        }
    }

    /// Try to free style slots on the page by rehashing the style set.
    ///
    /// Safety: `node` must be a valid `PageListNode` pointer.
    unsafe fn increase_capacity_styles(
        &mut self,
        node: *mut PageListNode,
    ) -> Result<(), ()> {
        unsafe {
            let page = &mut (*node).data;
            page.styles.rehash::<Style, StyleContext>(page.memory);
        }
        Ok(())
    }

    /// Apply an SGR attribute to the cursor style, then synchronize it with
    /// the page's style set via `manual_style_update`.
    ///
    /// This is called by stream_terminal when processing CSI `m` sequences.
    ///
    /// Safety: cursor.page_pin must be valid.
    pub unsafe fn set_attribute(
        &mut self,
        attr: Attribute,
    ) -> Result<(), ScreenStyleError> {
        match attr {
            Attribute::Unset => {
                self.cursor.style.flags = Flags(0);
                self.cursor.style.fg_color = Color::None;
                self.cursor.style.bg_color = Color::None;
                self.cursor.style.underline_color = Color::None;
                self.cursor.style_id = DEFAULT_ID;
            }
            Attribute::Unknown(_) => return Ok(()),

            Attribute::Bold => {
                self.cursor.style.flags.set_bold(true);
            }
            Attribute::ResetBold => {
                self.cursor.style.flags.set_bold(false);
            }
            Attribute::Faint => {
                self.cursor.style.flags.set_faint(true);
            }
            Attribute::Italic => {
                self.cursor.style.flags.set_italic(true);
            }
            Attribute::ResetItalic => {
                self.cursor.style.flags.set_italic(false);
            }

            Attribute::Underline(val) => {
                self.cursor.style.flags.set_underline(val != Underline::None);
            }
            Attribute::UnderlineColor(rgb) => {
                self.cursor.style.underline_color = Color::Rgb(rgb);
            }
            Attribute::UnderlineColor256(n) => {
                self.cursor.style.underline_color = Color::Palette(n);
            }
            Attribute::ResetUnderlineColor => {
                self.cursor.style.underline_color = Color::None;
            }

            Attribute::Overline => {
                self.cursor.style.flags.set_overline(true);
            }
            Attribute::ResetOverline => {
                self.cursor.style.flags.set_overline(false);
            }

            Attribute::Blink => {
                self.cursor.style.flags.set_blink(true);
            }
            Attribute::ResetBlink => {
                self.cursor.style.flags.set_blink(false);
            }

            Attribute::Inverse => {
                self.cursor.style.flags.set_inverse(true);
            }
            Attribute::ResetInverse => {
                self.cursor.style.flags.set_inverse(false);
            }

            Attribute::Invisible => {
                self.cursor.style.flags.set_invisible(true);
            }
            Attribute::ResetInvisible => {
                self.cursor.style.flags.set_invisible(false);
            }

            Attribute::Strikethrough => {
                self.cursor.style.flags.set_strikethrough(true);
            }
            Attribute::ResetStrikethrough => {
                self.cursor.style.flags.set_strikethrough(false);
            }

            Attribute::DirectColorFg(rgb) => {
                self.cursor.style.fg_color = Color::Rgb(rgb);
            }
            Attribute::DirectColorBg(rgb) => {
                self.cursor.style.bg_color = Color::Rgb(rgb);
            }

            Attribute::Color8Fg(name) => {
                self.cursor.style.fg_color = Color::Palette(name as u8);
            }
            Attribute::Color8Bg(name) => {
                self.cursor.style.bg_color = Color::Palette(name as u8);
            }
            Attribute::Color8BrightFg(name) => {
                self.cursor.style.fg_color = Color::Palette((name as u8) + 8);
            }
            Attribute::Color8BrightBg(name) => {
                self.cursor.style.bg_color = Color::Palette((name as u8) + 8);
            }

            Attribute::ResetFg => {
                self.cursor.style.fg_color = Color::None;
            }
            Attribute::ResetBg => {
                self.cursor.style.bg_color = Color::None;
            }

            Attribute::Palette256Fg(n) => {
                self.cursor.style.fg_color = Color::Palette(n);
            }
            Attribute::Palette256Bg(n) => {
                self.cursor.style.bg_color = Color::Palette(n);
            }
        }

        unsafe { self.manual_style_update() }
    }

    /// Scroll the active area and keep the cursor at the bottom of the screen.
    ///
    /// Safety: cursor.y == pages.rows - 1.
    pub unsafe fn cursor_down_scroll(&mut self) -> Result<(), ScreenResizeError> {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            let page_rows = if node.is_null() {
                return Ok(());
            } else {
                (*node).data.size.rows
            };
            assert!(self.cursor.y == page_rows - 1);

            self.kitty_images_dirty_set();

            if self.no_scrollback {
                if page_rows == 1 {
                    let page = &mut (*node).data;
                    let row = self.cursor.page_row;
                    let cells = page.row_cells_ptr(row);
                    self.clear_cells(page, row, cells, page.size.cols as usize);
                    self.cursor_mark_dirty();
                } else {
                    let old_pin = *self.cursor.page_pin;
                    let row = self.cursor.page_row;
                    if !row.is_null() {
                        let page = &mut (*node).data;
                        let cells = page.row_cells_ptr(row);
                        self.clear_cells(page, row, cells, page.size.cols as usize);
                    }
                    ptr::write(self.cursor.page_pin, old_pin);
                    let row_cell = pin_row_and_cell(self.cursor.page_pin);
                    self.cursor.page_row = row_cell.0;
                    self.cursor.page_cell = row_cell.1;
                }
            } else {
                let old_pin = *self.cursor.page_pin;
                let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
                let _ = pl.grow();
                ptr::write(self.cursor.page_pin, pin_down(old_pin, 1));
                let row_cell = pin_row_and_cell(self.cursor.page_pin);
                self.cursor.page_row = row_cell.0;
                self.cursor.page_cell = row_cell.1;
                self.cursor_mark_dirty();

                if self.cursor.style.bg_color != Color::None {
                    let page = &mut (*node).data;
                    let row = self.cursor.page_row;
                    let cells = page.row_cells_ptr(row);
                    self.clear_cells(page, row, cells, page.size.cols as usize);
                }
            }

            if self.cursor.style_id != DEFAULT_ID {
                let blank = self.blank_cell();
                if blank.0 != Cell::default().0 {
                    let page = &mut (*node).data;
                    let row = self.cursor.page_row;
                    let cells = page.row_cells_ptr(row);
                    let cols = page.size.cols as usize;
                    let mut i = 0usize;
                    while i < cols {
                        ptr::write(cells.add(i), blank);
                        i += 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// Scroll the active area at and above the cursor; the lines below the
    /// cursor are not scrolled.
    ///
    /// Safety: screen and cursor must be fully initialized.
    pub unsafe fn cursor_scroll_above(&mut self) -> Result<(), ScreenResizeError> {
        unsafe {
            self.cursor_mark_dirty();

            if self.pages.is_null() {
                return Ok(());
            }
            let pl: &mut PageList = &mut *self.pages.cast::<PageList>();

            let node = (*self.cursor.page_pin).node;
            let page_rows = if node.is_null() {
                return Ok(());
            } else {
                (*node).data.size.rows
            };

            if self.cursor.y == page_rows - 1 {
                return self.cursor_down_scroll();
            }

            let _old_pin = *self.cursor.page_pin;
            let grew_node = pl.grow();

            if !grew_node.is_null() {
                self.cursor_scroll_above_rotate();
            } else {
                let last_node = pl.pages.last;
                let cursor_node = (*self.cursor.page_pin).node;
                if cursor_node == last_node && !last_node.is_null() {
                    let new_pin = match (*self.cursor.page_pin).down(1) {
                        Some(p) => p,
                        None => return Ok(()),
                    };
                    ptr::write(self.cursor.page_pin, new_pin);

                    let page = &mut (*last_node).data;
                    let rows_base = page.rows_ptr();
                    let pin_y = new_pin.y as usize;
                    let total = page.size.rows as usize;
                    if total > pin_y {
                        rotate_rows_right_once(rows_base.add(pin_y), total - pin_y);
                    }
                    page.dirty = true;

                    let (row, cell) = pin_row_and_cell(self.cursor.page_pin);
                    self.cursor.page_row = row;
                    self.cursor.page_cell = cell;
                } else {
                    self.cursor_scroll_above_rotate();
                }
            }

            if self.cursor.style_id != DEFAULT_ID {
                let blank = self.blank_cell();
                if blank != Cell::default() {
                    let node = (*self.cursor.page_pin).node;
                    if !node.is_null() {
                        let page = &(*node).data;
                        let cols = page.size.cols as usize;
                        let base = self.cursor.page_cell;
                        let cells = base.sub(self.cursor.x as usize);
                        let mut i = 0usize;
                        while i < cols {
                            ptr::write(cells.add(i), blank);
                            i += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper for cursor_scroll_above: rotate rows across multiple pages so
    /// the newly created blank row lands at the cursor's position.
    ///
    /// Safety: screen and cursor must be fully initialized; cursor pin must be
    /// valid and within a page that grew by one row.
    unsafe fn cursor_scroll_above_rotate(&mut self) {
        unsafe {
            if self.pages.is_null() || self.cursor.page_pin.is_null() {
                return;
            }
            let pl: &PageList = &*self.pages.cast::<PageList>();

            let new_pin = match (*self.cursor.page_pin).down(1) {
                Some(p) => p,
                None => return,
            };
            self.cursor_change_pin(new_pin);

            let cursor_node = (*self.cursor.page_pin).node;
            let mut current = pl.pages.last;

            while !current.is_null() && current != cursor_node {
                let prev = (*current).prev;
                if prev.is_null() {
                    break;
                }
                let prev_page = &(*prev).data;
                let cur_page = &mut (*current).data;
                let prev_rows_base = prev_page.rows_ptr();
                let cur_rows_base = cur_page.rows_ptr();
                let prev_rows = prev_page.size.rows as usize;
                let cur_rows = cur_page.size.rows as usize;

                if cur_rows > 0 {
                    rotate_rows_right_once(cur_rows_base, cur_rows);
                }

                let src_row = prev_rows_base.add(prev_rows.saturating_sub(1));
                let dst_row = cur_rows_base;
                let _ = cur_page.clone_row_from(prev_page, dst_row, src_row);

                cur_page.dirty = true;

                current = prev;
            }

            if !cursor_node.is_null() {
                let cur_page = &mut (*cursor_node).data;
                let cur_rows_base = cur_page.rows_ptr();
                let pin_y = (*self.cursor.page_pin).y as usize;
                let total = cur_page.size.rows as usize;
                if total > pin_y {
                    rotate_rows_right_once(cur_rows_base.add(pin_y), total - pin_y);
                }

                let target_row = cur_rows_base.add(pin_y);
                let cells = cur_page.row_cells_ptr(target_row);
                self.clear_cells(cur_page, target_row, cells, cur_page.size.cols as usize);

                cur_page.dirty = true;

                let (row, cell) = pin_row_and_cell(self.cursor.page_pin);
                self.cursor.page_row = row;
                self.cursor.page_cell = cell;
            }
        }
    }

    /// Append a grapheme to the given cell within the current cursor row.
    ///
    /// Safety: cell must be a valid pointer into the cursor row.
    pub unsafe fn append_grapheme(
        &mut self,
        cell: *mut Cell,
        _cp: u32,
    ) -> Result<(), ScreenStyleError> {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return Ok(());
            }
            let page = &mut (*node).data;
            match page.append_grapheme(self.cursor.page_row, cell, _cp) {
                Ok(()) => {}
                Err(_) => {
                    let _ = self.increase_capacity(node, Some(PageCapacityAdjustment::GraphemeBytes));
                    let _ = page.append_grapheme(self.cursor.page_row, cell, _cp);
                }
            }
        }
        Ok(())
    }

    /// Start a hyperlink. Future cells will be marked as hyperlinks with this state.
    ///
    /// Safety: the screen must be initialized with a valid cursor.
    pub unsafe fn start_hyperlink(
        &mut self,
        _uri: &[u8],
        _id: Option<&[u8]>,
    ) -> Result<(), ScreenStyleError> {
        unsafe {
            if self.cursor.hyperlink_id != 0 {
                self.end_hyperlink();
            }
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return Ok(());
            }
            let page = &mut (*node).data;
            match page.insert_hyperlink(_uri.as_ptr(), _uri.len()) {
                Ok(id) => {
                    self.cursor.hyperlink_id = id;
                    let set: *mut crate::ref_counted_set::RefCountedSet =
                        &mut (*node).data.hyperlink_set;
                    (*set).use_id((*node).data.memory, id);
                }
                Err(_) => {
                    let _ = self.increase_capacity(
                        node,
                        Some(PageCapacityAdjustment::StringBytes),
                    );
                    let _ = self.increase_capacity(
                        node,
                        Some(PageCapacityAdjustment::HyperlinkBytes),
                    );
                    let page = &mut (*node).data;
                    match page.insert_hyperlink(_uri.as_ptr(), _uri.len()) {
                        Ok(id) => {
                            self.cursor.hyperlink_id = id;
                            let set: *mut crate::ref_counted_set::RefCountedSet =
                                &mut (*node).data.hyperlink_set;
                            (*set).use_id((*node).data.memory, id);
                        }
                        Err(_) => return Err(ScreenStyleError::OutOfMemory),
                    }
                }
            }
        }
        Ok(())
    }

    /// End the hyperlink state so future cells aren't part of the current hyperlink.
    pub fn end_hyperlink(&mut self) {
        if self.cursor.hyperlink_id == 0 {
            return;
        }
        unsafe {
            let node = if self.cursor.page_pin.is_null() {
                ptr::null_mut()
            } else {
                (*self.cursor.page_pin).node
            };
            if !node.is_null() {
                let set: *mut crate::ref_counted_set::RefCountedSet =
                    &mut (*node).data.hyperlink_set;
                (*set).release((*node).data.memory, self.cursor.hyperlink_id);
            }
            let link = self.cursor.hyperlink;
            if !link.is_null() {
                self.hyperlink_deinit(link);
            }
            self.cursor.hyperlink_id = 0;
            self.cursor.hyperlink = ptr::null_mut();
        }
    }

    /// Set the current hyperlink state on the current cell.
    ///
    /// Safety: hyperlink_id must be non-zero and cursor must be valid.
    pub unsafe fn cursor_set_hyperlink(&mut self) -> Result<(), ScreenStyleError> {
        unsafe {
            assert!(self.cursor.hyperlink_id != 0);
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return Ok(());
            }
            let page = &mut (*node).data;
            match page.set_hyperlink(
                self.cursor.page_row,
                self.cursor.page_cell,
                self.cursor.hyperlink_id,
            ) {
                Ok(()) => {
                    let set: *mut crate::ref_counted_set::RefCountedSet =
                        &mut (*node).data.hyperlink_set;
                    (*set).use_id((*node).data.memory, self.cursor.hyperlink_id);
                }
                Err(_) => {
                    let _ = self.increase_capacity(
                        node,
                        Some(PageCapacityAdjustment::HyperlinkBytes),
                    );
                    let page = &mut (*node).data;
                    let _ = page.set_hyperlink(
                        self.cursor.page_row,
                        self.cursor.page_cell,
                        self.cursor.hyperlink_id,
                    );
                    let set: *mut crate::ref_counted_set::RefCountedSet =
                        &mut (*node).data.hyperlink_set;
                    (*set).use_id((*node).data.memory, self.cursor.hyperlink_id);
                }
            }
        }
        Ok(())
    }

    /// Set the selection to the given value (or clear if None).
    ///
    /// This always marks the dirty flag.
    pub fn select(&mut self, sel: Option<Selection>) {
        match sel {
            None => {
                self.clear_selection();
            }
            Some(new_sel) => {
                // Untrack prior selection.
                if let Some(old) = self.selection.take() {
                    old.deinit(self.pages);
                }
                // If the incoming selection is untracked, convert to tracked
                // so its pins survive scrollback/resize operations. When
                // track_pin is unavailable (stub) we fall back to storing
                // the untracked selection as-is.
                let final_sel = if new_sel.is_tracked() {
                    Some(new_sel)
                } else {
                    match new_sel.track(self.pages) {
                        Some(tracked) => Some(tracked),
                        None => Some(new_sel),
                    }
                };
                self.selection = final_sel;
                self.dirty.selection = true;
            }
        }
    }

    /// Reset the screen according to DEC RIS semantics.
    pub fn reset(&mut self) {
        if !self.pages.is_null() {
            unsafe {
                let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
                pl.reset();
            }
        }
        unsafe {
            // The above reset preserves tracked pins so we can still use
            // our cursor pin, which should be at the top-left.
            if !self.cursor.page_pin.is_null() {
                let cursor_rac = pin_row_and_cell(self.cursor.page_pin);
                // Release cursor hyperlink: end_hyperlink frees the
                // heap-allocated Hyperlink struct and URI bytes (Zig:
                // Cursor.deinit -> Hyperlink.deinit + alloc.destroy),
                // and releases the reference against the page set.
                if self.cursor.hyperlink_id != 0 {
                    self.end_hyperlink();
                } else if !self.cursor.hyperlink.is_null() {
                    self.hyperlink_deinit(self.cursor.hyperlink);
                    self.cursor.hyperlink = ptr::null_mut();
                }
                self.cursor.pending_wrap = false;
                self.cursor.protected = false;
                self.cursor.style = Style::default();
                self.cursor.style_id = DEFAULT_ID;
                self.cursor.semantic_content = SemanticContent::Output;
                self.cursor.semantic_content_clear_eol = false;
                self.cursor.page_row = cursor_rac.0;
                self.cursor.page_cell = cursor_rac.1;
            }
        }
        // Reset kitty graphics storage.
        // Zig (Screen.zig:396-400):
        //   self.kitty_images.deinit(self.alloc, self);
        //   self.kitty_images = .{ .dirty = true };
        //
        // kitty_images is conditionally compiled (build_options.kitty_graphics)
        // and is either kitty.graphics.ImageStorage or an empty struct. When
        // enabled it requires:
        //   (a) a Zig FFI wrapper such as
        //       `ghostty_screen_kitty_images_reset(screen_ptr)` in
        //       c/kitty_graphics.zig that calls
        //       `screen.kitty_images.deinit(screen.alloc, &screen)` then
        //       `screen.kitty_images = .{ .dirty = true }`, and
        //   (b) the corresponding `@export` in lib_vt.zig.
        //
        // The Rust-side `kitty_images_dirty_set()` is currently a no-op
        // placeholder for part (b); once the Zig wrapper is added, call
        // it here via `extern "C"`.
        self.kitty_images_dirty_set();
        self.saved_cursor = None;
        self.charset = ScreenCharsetState::default();
        self.kitty_keyboard = KittyKeyFlagStack::default();
        self.protected_mode = ProtectedMode::OFF;
        self.semantic_prompt = ScreenSemanticPrompt::DISABLED;
        self.clear_selection();
    }

    /// Copy a row's cells from one row pointer to another within the same page.
    ///
    /// Safety: src_row and dst_row must be valid rows on `page`, and the page
    /// must have sufficient cells allocated.
    pub unsafe fn copy_row(
        &mut self,
        page: &mut Page,
        src_row: *mut Row,
        dst_row: *mut Row,
    ) {
        unsafe {
            let cols = page.size.cols as usize;
            let src_cells = page.row_cells_ptr(src_row);
            let dst_cells = page.row_cells_ptr(dst_row);

            let dst_cells_offset = (*dst_row).cells();
            let src_wrap = (*src_row).wrap();
            let src_wrap_cont = (*src_row).wrap_continuation();
            let src_semantic = (*src_row).semantic_prompt();
            let src_dirty = (*src_row).dirty();
            (*dst_row).set_wrap(src_wrap);
            (*dst_row).set_wrap_continuation(src_wrap_cont);
            (*dst_row).set_semantic_prompt(src_semantic);
            if src_dirty {
                (*dst_row).set_dirty(true);
            }

            if !(*src_row).managed_memory() {
                ptr::copy_nonoverlapping(src_cells, dst_cells, cols);
                return;
            }

            let mut i = 0usize;
            while i < cols {
                let s = src_cells.add(i);
                let d = dst_cells.add(i);
                ptr::copy_nonoverlapping(s, d, 1);

                if (*s).has_grapheme() {
                    (*d).set_content_tag(ContentTag::Codepoint);
                    page.move_grapheme(s, d);
                    (*d).set_content_tag(ContentTag::CodepointGrapheme);
                    (*dst_row).set_grapheme(true);
                }
                if (*s).hyperlink() {
                    (*d).set_hyperlink(false);
                    (*d).set_hyperlink(true);
                    (*dst_row).set_hyperlink(true);
                }
                i += 1;
            }
            let _ = dst_cells_offset;
        }
    }

    /// Insert `n` blank lines at the cursor's current row, shifting lines below down.
    /// Lines pushed past the bottom are lost. This is the IL (insert line) operation.
    ///
    /// Operates on the full scroll region (top=0, bottom=rows-1, left=0, right=cols-1).
    ///
    /// Safety: cursor must be on a valid row; rows below the cursor must be shiftable.
    pub unsafe fn insert_lines(&mut self, n: CellCountInt) {
        unsafe {
            if n == 0 || self.pages.is_null() || self.cursor.page_pin.is_null() {
                return;
            }
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &mut (*node).data;
            let page_rows = page.size.rows as usize;
            let page_cols = page.size.cols as usize;
            let cursor_y = (*self.cursor.page_pin).y as usize;
            if cursor_y >= page_rows {
                return;
            }

            let rem = page_rows - cursor_y;
            let adjusted = if (n as usize) < rem { n as usize } else { rem };

            let rows_base = page.rows_ptr();

            let mut y = page_rows;
            while y > cursor_y + adjusted {
                y -= 1;
                let src = y - adjusted;
                let dst_row = rows_base.add(y);
                let src_row = rows_base.add(src);
                *dst_row = *src_row;
            }

            let mut y = cursor_y;
            while y < cursor_y + adjusted {
                let row = rows_base.add(y);
                let cells_offset_saved = (*row).cells();
                let cells = page.row_cells_ptr(row);
                self.clear_cells(page, row, cells, page_cols);
                *row = Row(0);
                (*row).set_cells(cells_offset_saved);
                y += 1;
            }
            page.dirty = true;

            let start_y = (*self.cursor.page_pin).y;
            self.cursor_absolute(0, start_y);
            self.cursor.pending_wrap = false;
        }
    }

    /// Delete `n` lines at the cursor's current row, shifting lines below up.
    /// New blank lines are appended at the bottom. This is the DL (delete line) operation.
    ///
    /// Operates on the full scroll region (top=0, bottom=rows-1, left=0, right=cols-1).
    ///
    /// Safety: cursor must be on a valid row; rows below the cursor must be shiftable.
    pub unsafe fn delete_lines(&mut self, n: CellCountInt) {
        unsafe {
            if n == 0 || self.pages.is_null() || self.cursor.page_pin.is_null() {
                return;
            }
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &mut (*node).data;
            let page_rows = page.size.rows as usize;
            let page_cols = page.size.cols as usize;
            let cursor_y = (*self.cursor.page_pin).y as usize;
            if cursor_y >= page_rows {
                return;
            }

            let rem = page_rows - cursor_y;
            let adjusted = if (n as usize) < rem { n as usize } else { rem };

            let rows_base = page.rows_ptr();

            let mut y = cursor_y;
            while y + adjusted < page_rows {
                let src = rows_base.add(y + adjusted);
                let dst = rows_base.add(y);
                *dst = *src;
                y += 1;
            }

            let clear_start = page_rows - adjusted;
            let mut y = clear_start;
            while y < page_rows {
                let row = rows_base.add(y);
                let cells_offset_saved = (*row).cells();
                let cells = page.row_cells_ptr(row);
                self.clear_cells(page, row, cells, page_cols);
                *row = Row(0);
                (*row).set_cells(cells_offset_saved);
                y += 1;
            }
            page.dirty = true;

            let start_y = (*self.cursor.page_pin).y;
            self.cursor_absolute(0, start_y);
            self.cursor.pending_wrap = false;
        }
    }

    /// Erase the current cursor line (EL: Erase Line). Mode determines the range:
    /// - 0: from cursor to end of line
    /// - 1: from start of line to cursor
    /// - 2: entire line
    ///
    /// Safety: cursor must be on a valid row.
    pub unsafe fn erase_line(&mut self, mode: u8) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &mut (*node).data;
            let row = self.cursor.page_row;
            let cells = page.row_cells_ptr(row);
            let cols = page.size.cols as usize;
            let x = self.cursor.x as usize;

            match mode {
                0 => {
                    if x < cols {
                        self.clear_cells(page, row, cells.add(x), cols - x);
                    }
                }
                1 => {
                    if x > 0 {
                        self.clear_cells(page, row, cells, x);
                    }
                    let here = cells.add(x);
                    self.clear_cells(page, row, here, 1);
                }
                2 => {
                    self.clear_cells(page, row, cells, cols);
                }
                _ => {}
            }
        }
    }

    /// Erase the display (ED: Erase Display). Mode determines the region:
    /// - 0: from cursor to end of display
    /// - 1: from start of display to cursor
    /// - 2: entire display (preserve scrollback)
    /// - 3: entire display + scrollback
    ///
    /// Safety: cursor must be on a valid row; pages must be initialized.
    pub unsafe fn erase_display(&mut self, mode: u8) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &mut (*node).data;
            let cols = page.size.cols as usize;

            match mode {
                0 => {
                    let cells = page.row_cells_ptr(self.cursor.page_row);
                    let x = self.cursor.x as usize;
                    if x < cols {
                        self.clear_cells(page, self.cursor.page_row, cells.add(x), cols - x);
                    }
                    let pl: &PageList = &*self.pages.cast::<PageList>();
                    let total_rows = pl.rows as usize;
                    let below = (self.cursor.y as usize) + 1;
                    if below < total_rows {
                        self.clear_rows(
                            below as CellCountInt,
                            Some((total_rows - 1) as CellCountInt),
                            false,
                        );
                    }
                }
                1 => {
                    let cells = page.row_cells_ptr(self.cursor.page_row);
                    let x = self.cursor.x as usize;
                    if x > 0 {
                        self.clear_cells(page, self.cursor.page_row, cells, x);
                    }
                    let here = cells.add(x);
                    self.clear_cells(page, self.cursor.page_row, here, 1);
                    let cursor_y = self.cursor.y as usize;
                    if cursor_y > 0 {
                        self.clear_rows(0, Some((cursor_y - 1) as CellCountInt), false);
                    }
                }
                2 => {
                    let pl: &PageList = &*self.pages.cast::<PageList>();
                    let total_rows = pl.rows as usize;
                    if total_rows > 0 {
                        self.clear_rows(0, Some((total_rows - 1) as CellCountInt), false);
                    }
                }
                3 => {
                    let pl: &PageList = &*self.pages.cast::<PageList>();
                    let total_rows = pl.rows as usize;
                    if total_rows > 0 {
                        self.clear_rows(0, Some((total_rows - 1) as CellCountInt), false);
                    }
                    self.erase_history(None);
                }
                _ => {}
            }
        }
    }

    /// Index (IND): move the cursor down one line, or scroll if at the bottom.
    ///
    /// Safety: screen must be fully initialized.
    pub unsafe fn index(&mut self) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page_rows = (*node).data.size.rows;
            if self.cursor.y + 1 < page_rows {
                self.cursor_down(1);
            } else {
                let _ = self.cursor_down_scroll();
            }
        }
    }

    /// Reverse Index (RI): move the cursor up one line, or reverse-scroll
    /// if at the top.
    ///
    /// Safety: screen must be fully initialized.
    pub unsafe fn reverse_index(&mut self) {
        unsafe {
            if self.cursor.y > 0 {
                self.cursor_up(1);
            } else {
                if self.pages.is_null() || self.cursor.page_pin.is_null() {
                    return;
                }
                let node = (*self.cursor.page_pin).node;
                if node.is_null() {
                    return;
                }
                let page = &mut (*node).data;
                let rows_base = page.rows_ptr();
                let total = page.size.rows as usize;
                if total == 0 {
                    return;
                }
                rotate_rows_right_once(rows_base, total);
                let top_row = rows_base;
                let cells = page.row_cells_ptr(top_row);
                self.clear_cells(page, top_row, cells, page.size.cols as usize);
                page.dirty = true;
                let (row, cell) = pin_row_and_cell(self.cursor.page_pin);
                self.cursor.page_row = row;
                self.cursor.page_cell = cell;
            }
        }
    }

    /// Move the cursor to the cell at the given (x, y) position and write
    /// a codepoint into it.
    ///
    /// This is a simplified write path: it sets the cell's content to the
    /// given codepoint and applies the cursor's current style and hyperlink.
    /// It does NOT handle: graphemes, wide characters, scrolling, or wrapping.
    ///
    /// Safety: cursor must be at a valid position.
    pub unsafe fn write_cell(&mut self, cp: u32) {
        unsafe {
            if self.cursor.page_cell.is_null() {
                return;
            }
            let cell = self.cursor.page_cell;
            (*cell).set_content_tag(ContentTag::Codepoint);
            (*cell).set_content_codepoint(cp);
            (*cell).set_style_id(self.cursor.style_id);
            (*cell).set_protected(self.cursor.protected);
            (*cell).set_semantic_content(self.cursor.semantic_content);
            if self.cursor.hyperlink_id != 0 {
                let _ = self.cursor_set_hyperlink();
            }
        }
    }

    /// Move the cursor to (x, y), updating cached pointers.
    ///
    /// Safety: x, y within bounds.
    pub unsafe fn move_cell(&mut self, x: CellCountInt, y: CellCountInt) {
        unsafe {
            self.cursor_absolute(x, y);
        }
    }

    /// Set the cursor position to the given cell coordinates, clamping to bounds.
    ///
    /// Safety: screen must be initialized.
    pub unsafe fn cursor_set_cell(&mut self, x: CellCountInt, y: CellCountInt) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &(*node).data;
            let cx = if x >= page.size.cols { page.size.cols - 1 } else { x };
            let cy = if y >= page.size.rows { page.size.rows - 1 } else { y };
            self.cursor_absolute(cx, cy);
        }
    }

    /// Set only the cursor X coordinate, clamping to bounds.
    ///
    /// Safety: screen must be initialized.
    pub unsafe fn cursor_set_x(&mut self, x: CellCountInt) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &(*node).data;
            let cx = if x >= page.size.cols { page.size.cols - 1 } else { x };
            self.cursor_horizontal_absolute(cx);
        }
    }

    /// Set only the cursor Y coordinate, clamping to bounds.
    ///
    /// Safety: screen must be initialized.
    pub unsafe fn cursor_set_y(&mut self, y: CellCountInt) {
        unsafe {
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &(*node).data;
            let cy = if y >= page.size.rows { page.size.rows - 1 } else { y };
            self.cursor_absolute(self.cursor.x, cy);
        }
    }

    /// Save the current cursor state (position, style, charset, etc.).
    pub fn cursor_save(&mut self) {
        let x = self.cursor.x;
        let y = self.cursor.y;
        self.saved_cursor = Some(ScreenSavedCursor {
            x,
            y,
            style: self.cursor.style,
            protected: self.cursor.protected,
            pending_wrap: self.cursor.pending_wrap,
            origin: false,
            charset: self.charset.clone_charset_state(),
        });
    }

    /// Restore a previously saved cursor state.
    ///
    /// Safety: the saved cursor coordinates must be within current bounds.
    pub unsafe fn cursor_restore(&mut self) {
        unsafe {
            let sc = match self.saved_cursor {
                Some(ref s) => *s,
                None => return,
            };
            self.cursor.style = sc.style;
            self.cursor.protected = sc.protected;
            self.cursor.pending_wrap = sc.pending_wrap;
            self.charset = sc.charset;
            let _ = self.manual_style_update();

            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return;
            }
            let page = &(*node).data;
            let cx = if sc.x >= page.size.cols { page.size.cols - 1 } else { sc.x };
            let cy = if sc.y >= page.size.rows { page.size.rows - 1 } else { sc.y };
            self.cursor_absolute(cx, cy);
        }
    }

    /// Increase the capacity of the given page list node.
    ///
    /// Safety: node must be a valid page node in self.pages.
    pub unsafe fn increase_capacity(
        &mut self,
        _node: *mut crate::page_list_types::PageListNode,
        _adjustment: Option<PageCapacityAdjustment>,
    ) -> Result<*mut crate::page_list_types::PageListNode, ScreenIncreaseCapacityError> {
        // The full implementation delegates to PageList.increase_capacity which
        // allocates a new page with larger subsystem capacity and clones rows.
        // As a simplified approach, we rehash the relevant RefCountedSet to
        // reclaim dead slots; this can free capacity without allocation.
        unsafe {
            if !_node.is_null() {
                let page = &mut (*_node).data;
                let memory = page.memory;
                match _adjustment {
                    Some(PageCapacityAdjustment::Styles)
                    | Some(PageCapacityAdjustment::Rehash) => {
                        page.styles.rehash::<Style, StyleContext>(memory);
                    }
                    Some(PageCapacityAdjustment::HyperlinkBytes)
                    | Some(PageCapacityAdjustment::StringBytes) => {
                        page.hyperlink_set.rehash::<Style, StyleContext>(memory);
                    }
                    Some(PageCapacityAdjustment::GraphemeBytes) | None => {}
                }
            }
            // If the node being modified is our cursor page, re-add the
            // cursor style and hyperlink after the capacity change.
            self.cursor_reload();
        }
        Ok(_node)
    }

    /// Cleanly shut down the screen, releasing pages and cursor resources.
    pub fn deinit(&mut self) {
        if !self.cursor.hyperlink.is_null() {
            unsafe { self.hyperlink_deinit(self.cursor.hyperlink); }
            self.cursor.hyperlink = ptr::null_mut();
        }
        if !self.pages.is_null() {
            unsafe {
                let pl: &mut PageList = &mut *self.pages.cast::<PageList>();
                pl.deinit_pages();
            }
        }
        self.pages = ptr::null_mut();
    }
}

#[derive(Clone, Copy)]
pub enum PageCapacityAdjustment {
    Styles,
    StringBytes,
    GraphemeBytes,
    HyperlinkBytes,
    Rehash,
}

impl ScreenCharsetState {
    fn clone_charset_state(&self) -> ScreenCharsetState {
        ScreenCharsetState {
            charsets: ScreenCharsetArray {
                g0: self.charsets.g0,
                g1: self.charsets.g1,
                g2: self.charsets.g2,
                g3: self.charsets.g3,
            },
            gl: self.gl,
            gr: self.gr,
            single_shift: self.single_shift,
        }
    }
}

/// Move the last element to the first position (fastmem.rotateOnceR equivalent).
/// e.g. `[0 1 2 3]` becomes `[3 0 1 2]`.
#[inline]
unsafe fn rotate_rows_right_once(base: *mut Row, count: usize) {
    unsafe {
        if count <= 1 {
            return;
        }
        let last: Row = *base.add(count - 1);
        ptr::copy(base, base.add(1), count - 1);
        *base = last;
    }
}

/// Move the first element to the last position (fastmem.rotateOnce equivalent).
/// e.g. `[0 1 2 3]` becomes `[1 2 3 0]`.
#[inline]
unsafe fn rotate_rows_left_once(base: *mut Row, count: usize) {
    unsafe {
        if count <= 1 {
            return;
        }
        let first: Row = *base;
        ptr::copy(base.add(1), base, count - 1);
        *base.add(count - 1) = first;
    }
}

impl Screen {
    fn kitty_images_dirty_set(&mut self) {
        // The kitty_images field is an opaque pointer in the Rust port;
        // there is no dirty flag accessible here. This is intentionally
        // a no-op until the kitty image storage is fully ported.
    }

    #[inline]
    fn point_from_pin_active(&self, pin: Pin) -> Option<(usize, usize)> {
        if self.pages.is_null() {
            return None;
        }
        unsafe {
            let pl: &PageList = &*self.pages.cast::<PageList>();
            let (x, y) = pl.point_from_pin(PointTag::ACTIVE, pin)?;
            Some((x as usize, y as usize))
        }
    }

    #[inline]
    fn pin_active_origin(&self) -> Pin {
        if self.pages.is_null() {
            return Pin::default();
        }
        unsafe {
            let pl: &PageList = &*self.pages.cast::<PageList>();
            pl.get_top_left(PointTag::ACTIVE)
        }
    }
}

/// Helper: compute Pin.up(n) by walking the linked list backwards.
/// Returns the original pin if no upward movement is possible.
unsafe fn pin_up(pin: Pin, n: CellCountInt) -> Pin {
    if pin.node.is_null() {
        return pin;
    }
    unsafe {
        let mut result = pin;
        let mut remaining = n as usize;
        while remaining > 0 {
            let cur_y = result.y as usize;
            if cur_y >= remaining {
                result.y = (cur_y - remaining) as CellCountInt;
                return result;
            }
            remaining -= cur_y + 1;
            let prev = (*result.node).prev;
            if prev.is_null() {
                result.y = 0;
                return result;
            }
            result.node = prev;
            result.y = (*prev).data.size.rows - 1;
        }
        result
    }
}

/// Helper: compute Pin.down(n) by walking the linked list forwards.
unsafe fn pin_down(pin: Pin, n: CellCountInt) -> Pin {
    if pin.node.is_null() {
        return pin;
    }
    unsafe {
        let mut result = pin;
        let mut remaining = n as usize;
        while remaining > 0 {
            let rows = (*result.node).data.size.rows as usize;
            let space = rows - 1 - result.y as usize;
            if space >= remaining {
                result.y = (result.y as usize + remaining) as CellCountInt;
                return result;
            }
            remaining -= space + 1;
            let next = (*result.node).next;
            if next.is_null() {
                result.y = (rows - 1) as CellCountInt;
                return result;
            }
            result.node = next;
            result.y = 0;
        }
        result
    }
}

unsafe fn pin_down_safe(pin: Pin, n: CellCountInt) -> Option<Pin> {
    if pin.node.is_null() {
        return None;
    }
    unsafe {
        let cur_node = pin.node;
        let rows = (*cur_node).data.size.rows as usize;
        if (pin.y as usize) + (n as usize) < rows {
            let mut out = pin;
            out.y += n;
            return Some(out);
        }
        let next = (*cur_node).next;
        if next.is_null() {
            return None;
        }
        Some(pin_down(pin, n))
    }
}

/// Helper: get (row ptr, cell ptr) from a pin.
unsafe fn pin_row_and_cell(pin: *const Pin) -> (*mut Row, *mut Cell) {
    unsafe {
        let p = &*pin;
        let node = p.node;
        if node.is_null() {
            return (ptr::null_mut(), ptr::null_mut());
        }
        let page = &(*node).data;
        let rows_ptr = page.rows_ptr();
        let row = rows_ptr.add(p.y as usize);
        let cells = page.row_cells_ptr(row);
        let cell = cells.add(p.x as usize);
        (row, cell)
    }
}

/// Same as pin_row_and_cell but takes Pin by value.
unsafe fn pin_row_and_cell_raw(pin: Pin) -> (*mut Row, *mut Cell) {
    let mut p = pin;
    unsafe { pin_row_and_cell(&p) }
}
