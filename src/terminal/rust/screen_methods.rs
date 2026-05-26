#![allow(unused)]

use core::ffi::c_void;
use core::ptr;

use crate::early::*;
use crate::constants::*;
use crate::size_types::*;
use crate::page_types::*;
use crate::page_core::Page;
use crate::style_types::*;
use crate::ansi::*;
use crate::highlight::Pin;
use crate::kitty_key::KittyKeyFlagStack;
use crate::screen_types::*;
use crate::selection_types::*;

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

            if let Some(_old) = old_style {
                let old_node = (*self.cursor.page_pin).node;
                if !old_node.is_null() {
                    // TODO: release old style from old page:
                    // (*old_node).data.styles.release((*old_node).data.memory, self.cursor.style_id);
                }
                self.cursor.style = Style::default();
                self.cursor.style_id = DEFAULT_ID;
            }

            if !self.cursor.hyperlink.is_null() {
                let old_node = (*self.cursor.page_pin).node;
                if !old_node.is_null() {
                    // TODO: release old hyperlink from old page:
                    // (*old_node).data.hyperlink_set.release((*old_node).data.memory, self.cursor.hyperlink_id);
                }
            }

            ptr::write(self.cursor.page_pin, new);

            if let Some(old_s) = old_style {
                self.cursor.style = old_s;
                let _ = self.manual_style_update();
            }

            if !self.cursor.hyperlink.is_null() {
                // TODO: re-add hyperlink to new page
                // self.start_hyperlink(link.uri, ...)
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
            sel.deinit();
            self.dirty.selection = true;
        }
    }

    /// Scroll the viewport of the terminal grid.
    pub fn scroll(&mut self, behavior: ScreenScroll) {
        self.kitty_images_dirty_set();
        // TODO: delegate to self.pages.scroll(...)
        match behavior {
            ScreenScroll::Active => {}
            ScreenScroll::Top => {}
            ScreenScroll::Pin(_p) => {}
            ScreenScroll::Row(_v) => {}
            ScreenScroll::DeltaRow(_v) => {}
            ScreenScroll::DeltaPrompt(_v) => {}
        }
    }

    /// Returns true if the viewport is scrolled to the bottom of the screen.
    pub fn viewport_is_bottom(&self) -> bool {
        // TODO: check self.pages.viewport == .active
        // The viewport enum in Rust is PageListViewport. For now we can't
        // safely introspect the opaque pages pointer, so default to true
        // if pages is null, otherwise assume true (callers should verify).
        self.pages.is_null()
    }

    /// Assert that the screen is in a consistent state.
    pub fn assert_integrity(&self) {
        // TODO: in debug builds, verify cursor.x < pages.cols, cursor.y < pages.rows,
        // and that cursor x/y match the pin via pointFromPin(.active, cursor.page_pin.*).
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
                        // TODO: page.clear_grapheme(cell)
                    }
                    i += 1;
                }
                if len == page.size.cols as usize {
                    (*row).set_grapheme(false);
                }
                // TODO: else page.update_row_grapheme_flag(row)
            }

            if (*row).hyperlink() {
                let mut i = 0usize;
                while i < len {
                    let cell = cells.add(i);
                    if (*cell).hyperlink() {
                        // TODO: page.clear_hyperlink(cell)
                    }
                    i += 1;
                }
                if len == page.size.cols as usize {
                    (*row).set_hyperlink(false);
                }
                // TODO: else page.update_row_hyperlink_flag(row)
            }

            if (*row).styled() {
                let mut i = 0usize;
                while i < len {
                    let cell = cells.add(i);
                    if (*cell).style_id() != DEFAULT_ID {
                        // TODO: page.styles.release(page.memory, cell.style_id);
                    }
                    i += 1;
                }
                if len == page.size.cols as usize {
                    (*row).set_styled(false);
                }
                // TODO: else page.update_row_styled_flag(row)
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
            // TODO: iterate via self.pages.page_iterator(.right_down, tl, bl) and for each
            // row call clear_cells (or clear_unprotected_cells if protected is true).
        }
    }

    /// Erase the region specified by the bottom-left point, inclusive.
    /// This will physically erase the rows meaning the memory will be
    /// reclaimed and other rows will be shifted up.
    pub fn erase_history(&mut self, _bl: Option<(CellCountInt, CellCountInt)>) {
        // TODO: self.pages.erase_history(bl)
        unsafe { self.cursor_reload(); }
    }

    /// Erase active rows starting from y.
    pub fn erase_active(&mut self, _y: CellCountInt) {
        // TODO: self.pages.erase_active(y)
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
            // resize doesn't invalidate it.
            let had_hyperlink = !self.cursor.hyperlink.is_null();
            if self.cursor.hyperlink_id != 0 {
                let node = (*self.cursor.page_pin).node;
                if !node.is_null() {
                    // TODO: (*node).data.hyperlink_set.release((*node).data.memory, self.cursor.hyperlink_id);
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
                        // TODO: iterate from prompt start to end via prompt_iterator,
                        // clearing cells on each row.
                    }
                    PromptRedraw::False => {}
                }
            }

            // Perform the resize.
            // TODO: self.pages.resize(...)
            let _ = opts;

            if self.no_scrollback {
                // TODO: self.pages.erase_history(None);
            }

            self.cursor_reload();

            // TODO: if saved_cursor_pin was created, update saved_cursor x/y
            // and fix up pending_wrap if needed.

            // Restore the cursor style.
            self.cursor.style = cursor_style;
            let _ = self.manual_style_update();

            // TODO: fix up hyperlink if we had one.
            if had_hyperlink {
                // TODO: re-add hyperlink via self.start_hyperlink(...)
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
            let node = (*self.cursor.page_pin).node;
            if node.is_null() {
                return Ok(());
            }
            let page = &mut (*node).data;

            if self.cursor.style_id != DEFAULT_ID {
                // TODO: page.styles.release(page.memory, self.cursor.style_id)
            }

            if self.cursor.style.is_default() {
                self.cursor.style_id = DEFAULT_ID;
                return Ok(());
            }

            self.cursor.style_id = DEFAULT_ID;

            // TODO: page.styles.add(page.memory, self.cursor.style)
            // On error.OutOfMemory -> increase_capacity then retry
            // On error.OutOfSpace -> split_for_capacity then retry
            self.cursor.style_id = DEFAULT_ID;
        }
        Ok(())
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
                    // TODO: self.pages.erase_row(.{ .active = .{} })
                    ptr::write(self.cursor.page_pin, old_pin);
                    let row_cell = pin_row_and_cell(self.cursor.page_pin);
                    self.cursor.page_row = row_cell.0;
                    self.cursor.page_cell = row_cell.1;
                }
            } else {
                let old_pin = *self.cursor.page_pin;
                // TODO: self.pages.grow()
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
                // TODO: style newly created line per cursor.style.bg_cell
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

            let node = (*self.cursor.page_pin).node;
            let page_rows = if node.is_null() {
                return Ok(());
            } else {
                (*node).data.size.rows
            };

            if self.cursor.y == page_rows - 1 {
                return self.cursor_down_scroll();
            }

            assert!(self.cursor.y < page_rows - 1);

            let old_pin = *self.cursor.page_pin;
            // TODO: self.pages.grow()
            // TODO: if grew, call cursor_scroll_above_rotate()
            // else if on last page, do fast path with rotate
            // else cursor_scroll_above_rotate()

            // After scroll, style the newly created line.
            if self.cursor.style_id != DEFAULT_ID {
                // TODO: style newly created line per cursor.style.bg_cell
            }
        }
        Ok(())
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
            let _page = &mut (*node).data;
            // TODO: page.append_grapheme(self.cursor.page_row, cell, cp)
            // On OutOfMemory: increase_capacity for .grapheme_bytes, then retry
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
            // TODO: end any existing hyperlink, then allocate in the page's
            // hyperlink set. On StringsOutOfMemory/SetOutOfMemory/SetNeedsRehash
            // increase_capacity and retry.
        }
        Ok(())
    }

    /// End the hyperlink state so future cells aren't part of the current hyperlink.
    pub fn end_hyperlink(&mut self) {
        if self.cursor.hyperlink_id == 0 {
            return;
        }
        unsafe {
            // TODO: release from page.hyperlink_set and free cursor.hyperlink
            let node = if self.cursor.page_pin.is_null() {
                ptr::null_mut()
            } else {
                (*self.cursor.page_pin).node
            };
            if !node.is_null() {
                // (*node).data.hyperlink_set.release((*node).data.memory, self.cursor.hyperlink_id);
            }
            // TODO: also free the heap-allocated Hyperlink struct
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
            let _page = &mut (*node).data;
            // TODO: page.set_hyperlink(self.cursor.page_row, self.cursor.page_cell, self.cursor.hyperlink_id)
            // On HyperlinkMapOutOfMemory: increase_capacity for string_bytes and hyperlink_bytes, retry
            // On success: page.hyperlink_set.use(page.memory, self.cursor.hyperlink_id)
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
                    old.deinit();
                }
                // TODO: if untracked, convert to tracked via sel.track(self)
                self.selection = Some(new_sel);
                self.dirty.selection = true;
            }
        }
    }

    /// Reset the screen according to DEC RIS semantics.
    pub fn reset(&mut self) {
        // TODO: self.pages.reset()
        unsafe {
            // The above reset preserves tracked pins so we can still use
            // our cursor pin, which should be at the top-left.
            if !self.cursor.page_pin.is_null() {
                let cursor_rac = pin_row_and_cell(self.cursor.page_pin);
                // TODO: cursor.deinit(alloc) -- free hyperlink
                self.cursor.pending_wrap = false;
                self.cursor.protected = false;
                self.cursor.style = Style::default();
                self.cursor.style_id = DEFAULT_ID;
                self.cursor.hyperlink_id = 0;
                self.cursor.hyperlink = ptr::null_mut();
                self.cursor.semantic_content = SemanticContent::Output;
                self.cursor.semantic_content_clear_eol = false;
                self.cursor.page_row = cursor_rac.0;
                self.cursor.page_cell = cursor_rac.1;
            }
        }
        // TODO: kitty_images.deinit + reset to default
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

            // Copy the row metadata (preserving cells offset)
            let dst_cells_offset = (*dst_row).cells();
            let src_wrap = (*src_row).wrap();
            let src_wrap_cont = (*src_row).wrap_continuation();
            let src_semantic = (*src_row).semantic_prompt();
            (*dst_row).set_wrap(src_wrap);
            (*dst_row).set_wrap_continuation(src_wrap_cont);
            (*dst_row).set_semantic_prompt(src_semantic);

            // Simple memcpy of cells when no managed memory.
            if !(*src_row).managed_memory() {
                ptr::copy_nonoverlapping(src_cells, dst_cells, cols);
                return;
            }

            // When managed memory is involved, copy cell by cell and
            // migrate graphemes, hyperlinks, style refs.
            let mut i = 0usize;
            while i < cols {
                let s = src_cells.add(i);
                let d = dst_cells.add(i);
                ptr::copy_nonoverlapping(s, d, 1);

                if (*s).has_grapheme() {
                    (*d).set_content_tag(ContentTag::Codepoint);
                    // TODO: page.move_grapheme(s, d)
                    (*d).set_content_tag(ContentTag::CodepointGrapheme);
                    (*dst_row).set_grapheme(true);
                }
                if (*s).hyperlink() {
                    (*d).set_hyperlink(false);
                    // TODO: page.move_hyperlink(s, d)
                    (*d).set_hyperlink(true);
                    (*dst_row).set_hyperlink(true);
                }
                if (*s).style_id() != DEFAULT_ID {
                    // TODO: page.styles.use(page.memory, style_id)
                }
                i += 1;
            }
            let _ = dst_cells_offset;
        }
    }

    /// Insert `n` blank lines at the cursor's current row, shifting lines below down.
    /// Lines pushed past the bottom are lost. This is the IL (insert line) operation.
    ///
    /// Safety: cursor must be on a valid row; rows below the cursor must be shiftable.
    pub unsafe fn insert_lines(&mut self, _n: CellCountInt) {
        unsafe {
            // TODO: shift rows from cursor.y+1 downwards by n (rotate within page),
            // then clear n rows starting at cursor position using clear_cells.
            // This requires Page-level row rotation and clearing.
        }
    }

    /// Delete `n` lines at the cursor's current row, shifting lines below up.
    /// New blank lines are appended at the bottom. This is the DL (delete line) operation.
    ///
    /// Safety: cursor must be on a valid row; rows below the cursor must be shiftable.
    pub unsafe fn delete_lines(&mut self, _n: CellCountInt) {
        unsafe {
            // TODO: shift rows from cursor.y+n upwards by n (rotate within page),
            // then clear n rows at the bottom using clear_cells.
            // This requires Page-level row rotation and clearing.
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
                    // TODO: clear all rows below cursor via page iterator / clear_rows
                }
                1 => {
                    let cells = page.row_cells_ptr(self.cursor.page_row);
                    let x = self.cursor.x as usize;
                    if x > 0 {
                        self.clear_cells(page, self.cursor.page_row, cells, x);
                    }
                    let here = cells.add(x);
                    self.clear_cells(page, self.cursor.page_row, here, 1);
                    // TODO: clear all rows above cursor
                }
                2 => {
                    // TODO: clear all rows in active area
                }
                3 => {
                    // TODO: clear all rows + erase scrollback (erase_history(None))
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
                // TODO: insert a line at the top of the scrolling region via
                // page row rotation, then clear the newly created top row.
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
            // TODO: apply hyperlink if cursor.hyperlink_id != 0
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
        // TODO: delegate to self.pages.increase_capacity(node, adjustment).
        // If the node being modified is our cursor page, we need to re-add the
        // cursor style and hyperlink after the capacity change, and reload
        // the cursor information.
        unsafe {
            self.cursor_reload();
        }
        Ok(_node)
    }

    /// Cleanly shut down the screen, releasing pages and cursor resources.
    pub fn deinit(&mut self) {
        // TODO: free cursor.hyperlink via allocator
        if !self.cursor.hyperlink.is_null() {
            // self.alloc.destroy(self.cursor.hyperlink);
            self.cursor.hyperlink = ptr::null_mut();
        }
        // TODO: self.pages.deinit()
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

impl Screen {
    fn kitty_images_dirty_set(&mut self) {
        // The kitty_images field is an opaque pointer in the Rust port;
        // there is no dirty flag accessible here. This is intentionally
        // a no-op until the kitty image storage is fully ported.
    }

    #[inline]
    fn point_from_pin_active(&self, _pin: Pin) -> Option<(usize, usize)> {
        // TODO: delegate to self.pages.point_from_pin(.active, pin) which
        // returns the active-area coordinates of the pin. Without the
        // page list this cannot be computed.
        None
    }

    #[inline]
    fn pin_active_origin(&self) -> Pin {
        // TODO: self.pages.pin(.{ .active = .{} }) -- the top-left of the active area
        Pin::default()
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
