use crate::early::*;
use crate::page_core::*;
use crate::page_types::*;
use crate::size_types::*;
use core::ptr;

const STYLE_DEFAULT_ID: u16 = 0;
const KITTY_GRAPHICS_UNICODE_PLACEHOLDER: u32 = 0xE0B6;

impl Page {
    fn get_cell_offset(&self, cell: *const Cell) -> OffsetInt {
        (cell as usize).wrapping_sub(self.memory as usize) as OffsetInt
    }

    pub unsafe fn move_cells(
        &mut self,
        src_row: *mut Row,
        src_left: usize,
        dst_row: *mut Row,
        dst_left: usize,
        len: usize,
    ) {
        unsafe {
            let cols = self.size.cols;
            let src_cells = self.row_cells_ptr(src_row);
            let dst_cells = self.row_cells_ptr(dst_row);

            self.clear_cells(dst_row, dst_left, dst_left + len);

            let src_ref = &*src_row;
            let has_managed = src_ref.managed_memory();

            if !has_managed {
                ptr::copy(src_cells.add(src_left), dst_cells.add(dst_left), len);
            } else {
                let mut i = 0usize;
                while i < len {
                    let s = src_cells.add(src_left + i);
                    let d = dst_cells.add(dst_left + i);
                    ptr::copy(s, d, 1);

                    if (*s).has_grapheme() {
                        (*d).set_content_tag(ContentTag::Codepoint);
                        self.move_grapheme(s, d);
                        (*s).set_content_tag(ContentTag::Codepoint);
                        (*d).set_content_tag(ContentTag::CodepointGrapheme);
                        (*dst_row).set_grapheme(true);
                    }
                    if (*s).hyperlink() {
                        (*d).set_hyperlink(false);
                        self.move_hyperlink(s, d);
                        (*d).set_hyperlink(true);
                        (*dst_row).set_hyperlink(true);
                    }
                    if (*s).codepoint() == KITTY_GRAPHICS_UNICODE_PLACEHOLDER {
                        (*dst_row).set_kitty_virtual_placeholder(true);
                    }
                    i += 1;
                }
            }

            if !(*dst_row).styled() {
                let dc = self.row_cells_ptr(dst_row);
                let mut j = 0usize;
                while j < len {
                    if (*dc.add(dst_left + j)).style_id() != STYLE_DEFAULT_ID {
                        (*dst_row).set_styled(true);
                        break;
                    }
                    j += 1;
                }
            }

            ptr::write_bytes(src_cells.add(src_left), 0, len);

            if len == cols as usize {
                (*src_row).set_grapheme(false);
                (*src_row).set_hyperlink(false);
                (*src_row).set_styled(false);
                (*src_row).set_kitty_virtual_placeholder(false);
            }
        }
        self.assert_integrity();
    }

    pub unsafe fn swap_cells(&mut self, src: *mut Cell, dst: *mut Cell) {
        unsafe {
            if (*src).has_grapheme() || (*dst).has_grapheme() {
                match ((*src).has_grapheme(), (*dst).has_grapheme()) {
                    (true, false) => self.move_grapheme(src, dst),
                    (false, true) => self.move_grapheme(dst, src),
                    (true, true) => {
                        // TODO: requires GraphemeMap port - swap values in map
                    }
                    _ => {}
                }
            }

            if (*src).hyperlink() || (*dst).hyperlink() {
                match ((*src).hyperlink(), (*dst).hyperlink()) {
                    (true, false) => self.move_hyperlink(src, dst),
                    (false, true) => self.move_hyperlink(dst, src),
                    (true, true) => {
                        // TODO: requires hyperlink.Map port - swap values in map
                    }
                    _ => {}
                }
            }

            let old_dst = *dst;
            *dst = *src;
            *src = old_dst;
        }
        self.assert_integrity();
    }

    pub unsafe fn clear_cells(&mut self, row: *mut Row, left: usize, end: usize) {
        unsafe {
            let cols = self.size.cols;
            let cells = self.row_cells_ptr(row);
            let count = end - left;

            if (*row).grapheme() {
                let mut i = 0usize;
                while i < count {
                    let c = cells.add(left + i);
                    if (*c).has_grapheme() {
                        self.clear_grapheme(c);
                    }
                    i += 1;
                }
                if count == cols as usize {
                    (*row).set_grapheme(false);
                } else {
                    self.update_row_grapheme_flag(row);
                }
            }

            if (*row).hyperlink() {
                let mut i = 0usize;
                while i < count {
                    let c = cells.add(left + i);
                    if (*c).hyperlink() {
                        self.clear_hyperlink(c);
                    }
                    i += 1;
                }
                if count == cols as usize {
                    (*row).set_hyperlink(false);
                } else {
                    self.update_row_hyperlink_flag(row);
                }
            }

            if (*row).styled() {
                let mut i = 0usize;
                while i < count {
                    let c = cells.add(left + i);
                    if (*c).has_styling() {
                        // TODO: requires StyleSet port - release(*c).style_id())
                    }
                    i += 1;
                }
                if count == cols as usize {
                    (*row).set_styled(false);
                } else {
                    self.update_row_styled_flag(row);
                }
            }

            if (*row).kitty_virtual_placeholder() && count == cols as usize {
                let mut found = false;
                let mut i = 0usize;
                while i < count {
                    if (*cells.add(left + i)).codepoint() == KITTY_GRAPHICS_UNICODE_PLACEHOLDER {
                        found = true;
                        break;
                    }
                    i += 1;
                }
                if !found {
                    (*row).set_kitty_virtual_placeholder(false);
                }
            }

            ptr::write_bytes(cells.add(left), 0, count);
        }
        self.assert_integrity();
    }

    pub unsafe fn lookup_hyperlink(&self, cell: *const Cell) -> Option<u16> {
        let _cell_offset = self.get_cell_offset(cell);
        // TODO: requires hyperlink.Map port
        None
    }

    pub unsafe fn clear_hyperlink(&mut self, cell: *mut Cell) {
        unsafe {
            let _cell_offset = self.get_cell_offset(cell);
            // TODO: requires hyperlink.Map + hyperlink.Set port
            (*cell).set_hyperlink(false);
        }
        self.assert_integrity();
    }

    pub unsafe fn update_row_hyperlink_flag(&mut self, row: *mut Row) {
        unsafe {
            let cols = self.size.cols as usize;
            let cells = self.row_cells_ptr(row);
            let mut i = 0usize;
            while i < cols {
                if (*cells.add(i)).hyperlink() {
                    return;
                }
                i += 1;
            }
            (*row).set_hyperlink(false);
        }
    }

    pub unsafe fn insert_hyperlink(
        &mut self,
        _uri: *const u8,
        _uri_len: usize,
    ) -> Result<u16, &'static str> {
        // TODO: requires hyperlink.Set + StringAlloc port
        Err("insertHyperlink not yet implemented")
    }

    pub unsafe fn set_hyperlink(
        &mut self,
        row: *mut Row,
        cell: *mut Cell,
        _id: u16,
    ) -> Result<(), &'static str> {
        unsafe {
            let _cell_offset = self.get_cell_offset(cell);
            // TODO: requires hyperlink.Map port
            (*cell).set_hyperlink(true);
            (*row).set_hyperlink(true);
            Ok(())
        }
    }

    pub fn hyperlink_count(&self) -> usize {
        0
    }

    pub fn hyperlink_capacity(&self) -> usize {
        0
    }

    pub unsafe fn set_graphemes(
        &mut self,
        row: *mut Row,
        cell: *mut Cell,
        _cps: *const u32,
        _cps_len: usize,
    ) -> Result<(), &'static str> {
        unsafe {
            debug_assert!((*cell).codepoint() > 0);
            debug_assert!((*cell).content_tag() == ContentTag::Codepoint);
            let _cell_offset = self.get_cell_offset(cell);
            // TODO: requires GraphemeMap port
            (*cell).set_content_tag(ContentTag::CodepointGrapheme);
            (*row).set_grapheme(true);
            Ok(())
        }
    }

    pub unsafe fn append_grapheme(
        &mut self,
        row: *mut Row,
        cell: *mut Cell,
        _cp: u32,
    ) -> Result<(), &'static str> {
        unsafe {
            let _cell_offset = self.get_cell_offset(cell);
            if (*cell).content_tag() != ContentTag::CodepointGrapheme {
                // TODO: requires GraphemeMap port
                (*cell).set_content_tag(ContentTag::CodepointGrapheme);
                (*row).set_grapheme(true);
            }
            Ok(())
        }
    }

    pub unsafe fn lookup_grapheme(&self, cell: *const Cell) -> Option<(*const u32, usize)> {
        let _cell_offset = self.get_cell_offset(cell);
        // TODO: requires GraphemeMap port
        None
    }

    pub unsafe fn move_grapheme(&mut self, src: *mut Cell, dst: *mut Cell) {
        let _src_offset = self.get_cell_offset(src);
        let _dst_offset = self.get_cell_offset(dst);
        // TODO: requires GraphemeMap port
    }

    pub unsafe fn clear_grapheme(&mut self, cell: *mut Cell) {
        let _cell_offset = self.get_cell_offset(cell);
        // TODO: requires GraphemeMap port
        unsafe { (*cell).set_content_tag(ContentTag::Codepoint); }
        self.assert_integrity();
    }

    pub unsafe fn update_row_grapheme_flag(&mut self, row: *mut Row) {
        unsafe {
            let cols = self.size.cols as usize;
            let cells = self.row_cells_ptr(row);
            let mut i = 0usize;
            while i < cols {
                if (*cells.add(i)).has_grapheme() {
                    return;
                }
                i += 1;
            }
            (*row).set_grapheme(false);
        }
    }

    pub fn grapheme_count(&self) -> usize {
        0
    }

    pub fn grapheme_capacity(&self) -> usize {
        0
    }

    pub unsafe fn update_row_styled_flag(&mut self, row: *mut Row) {
        unsafe {
            let cols = self.size.cols as usize;
            let cells = self.row_cells_ptr(row);
            let mut i = 0usize;
            while i < cols {
                if (*cells.add(i)).has_styling() {
                    return;
                }
                i += 1;
            }
            (*row).set_styled(false);
        }
    }

    unsafe fn move_hyperlink(&mut self, src: *mut Cell, dst: *mut Cell) {
        unsafe {
            debug_assert!((*src).hyperlink());
            debug_assert!(!(*dst).hyperlink());
            let _src_offset = self.get_cell_offset(src);
            let _dst_offset = self.get_cell_offset(dst);
            // TODO: requires hyperlink.Map port
        }
    }

    pub unsafe fn clone_page(&self) -> Result<Page, &'static str> {
        Err("clone not yet implemented: requires full page alloc")
    }

    pub unsafe fn clone_buf(&self, buf: *mut u8, buf_len: usize) -> Page {
        debug_assert!(buf_len >= self.memory_len);
        unsafe {
            ptr::copy_nonoverlapping(self.memory, buf, self.memory_len);
        }
        Page {
            memory: buf,
            memory_len: self.memory_len,
            rows: self.rows,
            cells: self.cells,
            dirty: self.dirty,
            string_alloc: self.string_alloc,
            grapheme_alloc: self.grapheme_alloc,
            grapheme_map: self.grapheme_map,
            styles: self.styles,
            hyperlink_map: self.hyperlink_map,
            hyperlink_set: self.hyperlink_set,
            size: self.size,
            capacity: self.capacity,
        }
    }

    pub fn exact_row_capacity(&self, y_start: usize, y_end: usize) -> PageCapacity {
        debug_assert!(y_start < y_end);
        debug_assert!(y_end <= self.size.rows as usize);
        PageCapacity {
            cols: self.size.cols,
            rows: (y_end - y_start) as CellCountInt,
            styles: 0,
            grapheme_bytes: 0,
            hyperlink_bytes: 0,
            string_bytes: 0,
        }
    }

    pub unsafe fn clone_from(
        &mut self,
        other: *const Page,
        y_start: usize,
        y_end: usize,
    ) -> Result<(), &'static str> {
        unsafe {
            let other_ref = &*other;
            debug_assert!(y_start <= y_end);
            debug_assert!(y_end <= other_ref.size.rows as usize);
            debug_assert!(y_end - y_start <= self.size.rows as usize);

            let row_count = y_end - y_start;
            let dst_rows = self.rows_ptr();
            let src_rows = other_ref.rows_ptr() as *const Row;

            let mut i = 0usize;
            while i < row_count {
                let dst_row = dst_rows.add(i);
                let src_row = src_rows.add(y_start + i);
                self.clone_row_from(other, dst_row, src_row)?;
                i += 1;
            }
        }
        self.assert_integrity();
        Ok(())
    }

    pub unsafe fn clone_row_from(
        &mut self,
        other: *const Page,
        dst_row: *mut Row,
        src_row: *const Row,
    ) -> Result<(), &'static str> {
        unsafe { self.clone_partial_row_from(other, dst_row, src_row, 0, self.size.cols as usize) }
    }

    pub unsafe fn clone_partial_row_from(
        &mut self,
        other: *const Page,
        dst_row: *mut Row,
        src_row: *const Row,
        x_start: usize,
        x_end_req: usize,
    ) -> Result<(), &'static str> {
        unsafe {
            let other_ref = &*other;
            let self_cols = self.size.cols as usize;
            let other_cols = other_ref.size.cols as usize;
            let cell_len = if self_cols < other_cols {
                self_cols
            } else {
                other_cols
            };
            let x_end = if x_end_req < cell_len { x_end_req } else { cell_len };
            debug_assert!(x_start <= x_end);
            let copy_len = x_end - x_start;

            let dst_cells = self.row_cells_ptr(dst_row);
            let src_cells = other_ref.row_cells_ptr(src_row as *mut Row) as *const Cell;

            if (*dst_row).managed_memory() {
                self.clear_cells(dst_row, x_start, x_end);
            }

            let saved_cells = (*dst_row).cells();
            let saved_wrap = (*dst_row).wrap();
            let saved_wrap_cont = (*dst_row).wrap_continuation();
            let saved_gr = (*dst_row).grapheme();
            let saved_hl = (*dst_row).hyperlink();
            let saved_st = (*dst_row).styled();
            let saved_dirty = (*dst_row).dirty();

            *dst_row = *src_row;

            if copy_len < self_cols {
                (*dst_row).set_wrap(saved_wrap);
                (*dst_row).set_wrap_continuation(saved_wrap_cont);
                (*dst_row).set_grapheme(saved_gr);
                (*dst_row).set_hyperlink(saved_hl);
                (*dst_row).set_styled(saved_st);
                if saved_dirty {
                    (*dst_row).set_dirty(true);
                }
            }

            (*dst_row).set_cells(saved_cells);

            if !(*src_row).managed_memory() {
                ptr::copy(src_cells.add(x_start), dst_cells.add(x_start), copy_len);
            } else {
                let mut i = 0usize;
                while i < copy_len {
                    let si = x_start + i;
                    let di = x_start + i;
                    let sc = src_cells.add(si) as *mut Cell;
                    let dc = dst_cells.add(di);

                    ptr::copy(sc, dc, 1);
                    (*dc).set_hyperlink(false);
                    (*dc).set_style_id(STYLE_DEFAULT_ID);
                    if (*dc).content_tag() == ContentTag::CodepointGrapheme {
                        (*dc).set_content_tag(ContentTag::Codepoint);
                    }

                    if (*sc).has_grapheme() {
                        let _cps = other_ref.lookup_grapheme(sc);
                        // TODO: requires GraphemeMap port
                    }

                    if (*sc).hyperlink() {
                        let _id = other_ref.lookup_hyperlink(sc);
                        // TODO: requires hyperlink.Set port
                    }

                    if (*sc).style_id() != STYLE_DEFAULT_ID {
                        (*dst_row).set_styled(true);
                        // TODO: requires StyleSet port
                    }

                    if (*sc).codepoint() == KITTY_GRAPHICS_UNICODE_PLACEHOLDER {
                        (*dst_row).set_kitty_virtual_placeholder(true);
                    }

                    i += 1;
                }
            }

            if self_cols > other_cols {
                let last = dst_cells.add(other_cols - 1);
                if (*last).wide() == Wide::SpacerHead {
                    (*last).set_wide(Wide::Narrow);
                }
            }
        }
        self.assert_integrity();
        Ok(())
    }
}
