use crate::early::*;
use crate::page_core::*;
use crate::page_types::*;
use crate::size_types::*;
use crate::bitmap_allocator::BitmapAllocator;
use crate::hash_map::{AutoContext, HashMapUnmanaged, OffsetHashMap};
use crate::hyperlink::{HyperlinkPageEntry, HyperlinkPageEntryId, HYPERLINK_DEFAULT_ID};
use crate::ref_counted_set::RefCountedSet;
use core::ptr;
use crate::style_types::Style;

const STYLE_DEFAULT_ID: u16 = 0;
const KITTY_GRAPHICS_UNICODE_PLACEHOLDER: u32 = 0xE0B6;

type HlHashMap = HashMapUnmanaged<OffsetInt, u16, AutoContext>;
type GrHashMap = HashMapUnmanaged<OffsetInt, OffsetSlice, AutoContext>;

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
                        let base = self.memory;
                        let gm_off = self.grapheme_map;
                        let src_offset = self.get_cell_offset(src);
                        let dst_offset = self.get_cell_offset(dst);
                        let mut map = self.grapheme_map.map(self.memory);
                        if let (Some(si), Some(di)) = (
                            map.get_index(&src_offset, AutoContext),
                            map.get_index(&dst_offset, AutoContext),
                        ) {
                            let vals = map.values_mut();
                            let sv = ptr::read(vals.add(si));
                            let dv = ptr::read(vals.add(di));
                            ptr::write(vals.add(si), dv);
                            ptr::write(vals.add(di), sv);
                        }
                    }
                    _ => {}
                }
            }

            if (*src).hyperlink() || (*dst).hyperlink() {
                match ((*src).hyperlink(), (*dst).hyperlink()) {
                    (true, false) => self.move_hyperlink(src, dst),
                    (false, true) => self.move_hyperlink(dst, src),
                    (true, true) => {
                        let base = self.memory;
                        let hm_off = self.hyperlink_map;
                        let src_offset = self.get_cell_offset(src);
                        let dst_offset = self.get_cell_offset(dst);
                        let mut map = self.hyperlink_map.map(self.memory);
                        if let (Some(si), Some(di)) = (
                            map.get_index(&src_offset, AutoContext),
                            map.get_index(&dst_offset, AutoContext),
                        ) {
                            let vals = map.values_mut();
                            let sv = ptr::read(vals.add(si));
                            let dv = ptr::read(vals.add(di));
                            ptr::write(vals.add(si), dv);
                            ptr::write(vals.add(di), sv);
                        }
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

            if (*row).styled() && self.styles.capacity() > 0 {
                let base = self.memory;
                let set: *mut RefCountedSet = &mut self.styles;
                let mut i = 0usize;
                while i < count {
                    let c = cells.add(left + i);
                    if (*c).has_styling() {
                        (*set).remove(base, (*c).style_id());
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
        if self.hyperlink_map.metadata.offset == 0 {
            return None;
        }
        unsafe {
            let cell_offset = self.get_cell_offset(cell);
            let map = self.hyperlink_map.map(self.memory);
            map.get(&cell_offset, AutoContext).copied()
        }
    }

    pub unsafe fn clear_hyperlink(&mut self, cell: *mut Cell) {
        unsafe {
            if self.hyperlink_map.metadata.offset == 0 || self.hyperlink_set.capacity() == 0 {
                (*cell).set_hyperlink(false);
                self.assert_integrity();
                return;
            }
            let base = self.memory;
            let cell_offset = self.get_cell_offset(cell);
            let mut map = self.hyperlink_map.map(self.memory);
            if let Some(kv) = map.fetch_remove(&cell_offset, AutoContext) {
                let set: *mut RefCountedSet = &mut self.hyperlink_set;
                (*set).remove(base, kv.value);
            }
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
        uri: *const u8,
        uri_len: usize,
    ) -> Result<u16, &'static str> {
        unsafe {
            if self.hyperlink_set.capacity() == 0 {
                return Err("HyperlinkSet uninitialized");
            }
            let base = self.memory;

            let uri_alloc = self
                .string_alloc
                .alloc::<u8>(base, uri_len)
                .ok_or("StringsOutOfMemory")?;
            ptr::copy_nonoverlapping(uri, uri_alloc.as_mut_ptr(), uri_len);
            let uri_off = get_offset(base as *const u8, uri_alloc.as_ptr() as *const u8);
            let uri_slice = OffsetSlice {
                offset: uri_off,
                len: uri_len,
            };

            let entry = HyperlinkPageEntry {
                id: HyperlinkPageEntryId::implicit(0),
                uri: uri_slice,
            };

            let set: *mut RefCountedSet = &mut self.hyperlink_set;
            let id = (*set)
                .next_id::<HyperlinkPageEntry>(base, entry)
                .ok_or("SetOutOfMemory")?;

            Ok(id)
        }
    }

    pub unsafe fn set_hyperlink(
        &mut self,
        row: *mut Row,
        cell: *mut Cell,
        id: u16,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.hyperlink_map.metadata.offset == 0 || self.hyperlink_set.capacity() == 0 {
                (*cell).set_hyperlink(true);
                (*row).set_hyperlink(true);
                return Ok(());
            }
            let base = self.memory;
            let cell_offset = self.get_cell_offset(cell);
            let mut map = self.hyperlink_map.map(self.memory);

            if map.count() >= map.capacity() {
                let existing = map.get_index(&cell_offset, AutoContext);
                if existing.is_none() {
                    return Err("HyperlinkMapOutOfMemory");
                }
            }

            let gop = map.get_or_put_assume_capacity(cell_offset, AutoContext);
            if gop.found_existing {
                let old_id = ptr::read(gop.value_ptr);
                let set: *mut RefCountedSet = &mut self.hyperlink_set;
                (*set).remove(base, old_id);
                if old_id == id {
                    (*cell).set_hyperlink(true);
                    return Ok(());
                }
            }
            ptr::write(gop.value_ptr, id);
            (*cell).set_hyperlink(true);
            (*row).set_hyperlink(true);
            Ok(())
        }
    }

    pub fn hyperlink_count(&self) -> usize {
        if self.hyperlink_map.metadata.offset == 0 {
            return 0;
        }
        unsafe {
            let map = self.hyperlink_map.map(self.memory);
            map.count() as usize
        }
    }

    pub fn hyperlink_capacity(&self) -> usize {
        if self.hyperlink_map.metadata.offset == 0 {
            return 0;
        }
        unsafe {
            let map = self.hyperlink_map.map(self.memory);
            map.capacity() as usize
        }
    }

    pub unsafe fn set_graphemes(
        &mut self,
        row: *mut Row,
        cell: *mut Cell,
        cps: *const u32,
        cps_len: usize,
    ) -> Result<(), &'static str> {
        unsafe {
            debug_assert!((*cell).codepoint() > 0);
            debug_assert!((*cell).content_tag() == ContentTag::Codepoint);

            if self.grapheme_map.metadata.offset == 0 {
                (*cell).set_content_tag(ContentTag::CodepointGrapheme);
                (*row).set_grapheme(true);
                return Ok(());
            }

            let base = self.memory;
            let gm_off = self.grapheme_map;

            let alloc = self
                .grapheme_alloc
                .alloc::<u32>(base, cps_len)
                .ok_or("GraphemeAllocOutOfMemory")?;
            ptr::copy_nonoverlapping(cps, alloc.as_mut_ptr(), cps_len);

            let cp_off = get_offset(base as *const u8, alloc.as_ptr() as *const u8);
            let slice = OffsetSlice {
                offset: cp_off,
                len: cps_len,
            };

            let cell_offset = self.get_cell_offset(cell);
            let mut map = self.grapheme_map.map(self.memory);

            if map.count() >= map.capacity() {
                return Err("GraphemeMapOutOfMemory");
            }
            map.put_assume_capacity_no_clobber(cell_offset, slice, AutoContext);

            (*cell).set_content_tag(ContentTag::CodepointGrapheme);
            (*row).set_grapheme(true);
            Ok(())
        }
    }

    pub unsafe fn append_grapheme(
        &mut self,
        row: *mut Row,
        cell: *mut Cell,
        cp: u32,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.grapheme_map.metadata.offset == 0 {
                if (*cell).content_tag() != ContentTag::CodepointGrapheme {
                    (*cell).set_content_tag(ContentTag::CodepointGrapheme);
                    (*row).set_grapheme(true);
                }
                return Ok(());
            }

            let base = self.memory;
            let gm_off = self.grapheme_map;
            let cell_offset = self.get_cell_offset(cell);

            if (*cell).content_tag() != ContentTag::CodepointGrapheme {
                let alloc = self
                    .grapheme_alloc
                    .alloc::<u32>(base, 1)
                    .ok_or("GraphemeAllocOutOfMemory")?;
                ptr::write(alloc.as_mut_ptr(), cp);

                let cp_off = get_offset(base as *const u8, alloc.as_ptr() as *const u8);
                let slice = OffsetSlice {
                    offset: cp_off,
                    len: 1,
                };

                let mut map = self.grapheme_map.map(self.memory);
                if map.count() >= map.capacity() {
                    return Err("GraphemeMapOutOfMemory");
                }
                map.put_assume_capacity_no_clobber(cell_offset, slice, AutoContext);

                (*cell).set_content_tag(ContentTag::CodepointGrapheme);
                (*row).set_grapheme(true);
                return Ok(());
            }

            debug_assert!((*row).grapheme());
            let mut map = self.grapheme_map.map(self.memory);
            let idx = map
                .get_index(&cell_offset, AutoContext)
                .ok_or("GraphemeNotFound")?;
            let vals = map.values_mut();
            let val_ptr = vals.add(idx);
            let mut slice = ptr::read(val_ptr);

            if slice.len % GRAPHEME_CHUNK_LEN != 0 {
                let cps_ptr: *mut u32 = slice.offset.ptr_mut(base);
                ptr::write(cps_ptr.add(slice.len), cp);
                slice.len += 1;
                ptr::write(val_ptr, slice);
                return Ok(());
            }

            let new_len = slice.len + 1;
            let new_alloc = self
                .grapheme_alloc
                .alloc::<u32>(base, new_len)
                .ok_or("GraphemeAllocOutOfMemory")?;
            let old_slice_data = slice.offset.ptr(base);
            ptr::copy_nonoverlapping(old_slice_data, new_alloc.as_mut_ptr(), slice.len);
            ptr::write(new_alloc.as_mut_ptr().add(slice.len), cp);

            let old_data: *const u32 = slice.offset.ptr(base);
            let old_len = slice.len;
            let old_raw = core::slice::from_raw_parts(old_data, old_len);
            self.grapheme_alloc.free::<u32>(base, old_raw);

            let cp_off = get_offset(base as *const u8, new_alloc.as_ptr() as *const u8);
            ptr::write(
                val_ptr,
                OffsetSlice {
                    offset: cp_off,
                    len: new_len,
                },
            );
            Ok(())
        }
    }

    pub unsafe fn lookup_grapheme(&self, cell: *const Cell) -> Option<(*const u32, usize)> {
        if self.grapheme_map.metadata.offset == 0 {
            return None;
        }
        unsafe {
            let cell_offset = self.get_cell_offset(cell);
            let map = self.grapheme_map.map(self.memory);
            let os = map.get(&cell_offset, AutoContext).copied()?;
            let ptr: *const u32 = os.offset.ptr(self.memory as *const u8);
            Some((ptr, os.len))
        }
    }

    pub unsafe fn move_grapheme(&mut self, src: *mut Cell, dst: *mut Cell) {
        if self.grapheme_map.metadata.offset == 0 {
            return;
        }
        let src_offset = self.get_cell_offset(src);
        let dst_offset = self.get_cell_offset(dst);
        unsafe {
            let base = self.memory;
            let gm_off = self.grapheme_map;
            let mut map = self.grapheme_map.map(self.memory);
            if let Some(kv) = map.fetch_remove(&src_offset, AutoContext) {
                map.put_assume_capacity(dst_offset, kv.value, AutoContext);
            }
        }
    }

    pub unsafe fn clear_grapheme(&mut self, cell: *mut Cell) {
        if self.grapheme_map.metadata.offset == 0 {
            unsafe {
                (*cell).set_content_tag(ContentTag::Codepoint);
            }
            self.assert_integrity();
            return;
        }
        unsafe {
            let cell_offset = self.get_cell_offset(cell);
            let base = self.memory;
            let gm_off = self.grapheme_map;
            let mut map = self.grapheme_map.map(self.memory);
            if let Some(kv) = map.fetch_remove(&cell_offset, AutoContext) {
                let os = kv.value;
                let data_ptr: *const u32 = os.offset.ptr(base as *const u8);
                let slice = core::slice::from_raw_parts(data_ptr, os.len);
                self.grapheme_alloc.free::<u32>(base, slice);
            }
            (*cell).set_content_tag(ContentTag::Codepoint);
        }
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
        if self.grapheme_map.metadata.offset == 0 {
            return 0;
        }
        unsafe {
            let map = self.grapheme_map.map(self.memory);
            map.count() as usize
        }
    }

    pub fn grapheme_capacity(&self) -> usize {
        if self.grapheme_map.metadata.offset == 0 {
            return 0;
        }
        unsafe {
            let map = self.grapheme_map.map(self.memory);
            map.capacity() as usize
        }
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
        if self.hyperlink_map.metadata.offset == 0 {
            return;
        }
        unsafe {
            debug_assert!((*src).hyperlink());
            debug_assert!(!(*dst).hyperlink());
            let src_offset = self.get_cell_offset(src);
            let dst_offset = self.get_cell_offset(dst);
            let base = self.memory;
            let hm_off = self.hyperlink_map;
            let mut map = self.hyperlink_map.map(self.memory);
            if let Some(kv) = map.fetch_remove(&src_offset, AutoContext) {
                map.put_assume_capacity(dst_offset, kv.value, AutoContext);
            }
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

        let cols = self.size.cols as usize;
        let base = self.memory;

        let mut id_bits = [0u64; 1024];
        let mut grapheme_bytes: usize = 0;

        unsafe {
            let rows_ptr: *const Row = self.rows.ptr(base);
            let mut y = y_start;
            while y < y_end {
                let row = &*rows_ptr.add(y);
                let cells_ptr: *const Cell = row.cells().ptr(base);
                let mut x = 0usize;
                while x < cols {
                    let cell = cells_ptr.add(x);
                    let sid = (*cell).style_id();
                    if sid != STYLE_DEFAULT_ID {
                        let idx = sid as usize;
                        *id_bits.get_unchecked_mut(idx / 64) |= 1u64 << (idx % 64);
                    }
                    if (*cell).has_grapheme() && self.grapheme_map.metadata.offset != 0 {
                        if let Some((_p, len)) = self.lookup_grapheme(cell) {
                            grapheme_bytes +=
                                BitmapAllocator::<GRAPHEME_CHUNK>::bytes_required::<u32>(len);
                        }
                    }
                    x += 1;
                }
                y += 1;
            }
        }

        let mut style_count: usize = 0;
        for w in id_bits.iter() {
            style_count += w.count_ones() as usize;
        }

        let mut i = 0usize;
        while i < 1024 {
            unsafe {
                *id_bits.get_unchecked_mut(i) = 0;
            }
            i += 1;
        }

        let mut string_bytes: usize = 0;
        let mut hyperlink_cells: usize = 0;

        unsafe {
            let rows_ptr: *const Row = self.rows.ptr(base);
            let mut y = y_start;
            while y < y_end {
                let row = &*rows_ptr.add(y);
                let cells_ptr: *const Cell = row.cells().ptr(base);
                let mut x = 0usize;
                while x < cols {
                    let cell = cells_ptr.add(x);
                    if (*cell).hyperlink() {
                        hyperlink_cells += 1;
                        if let Some(id) = self.lookup_hyperlink(cell) {
                            let idx = id as usize;
                            let word = *id_bits.get_unchecked(idx / 64);
                            let bit = 1u64 << (idx % 64);
                            if (word & bit) == 0 {
                                *id_bits.get_unchecked_mut(idx / 64) = word | bit;
                                if self.hyperlink_set.capacity() > 0 {
                                    let set: *const RefCountedSet = &self.hyperlink_set;
                                    let entry: HyperlinkPageEntry =
                                        (*set).get::<HyperlinkPageEntry>(base as *mut u8, id);
                                    string_bytes +=
                                        BitmapAllocator::<STRING_CHUNK>::bytes_required::<u8>(
                                            entry.uri.len,
                                        );
                                    if entry.id.is_explicit() {
                                        string_bytes +=
                                            BitmapAllocator::<STRING_CHUNK>::bytes_required::<u8>(
                                                entry.id.explicit.len,
                                            );
                                    }
                                }
                            }
                        }
                    }
                    x += 1;
                }
                y += 1;
            }
        }

        let mut hl_count: usize = 0;
        for w in id_bits.iter() {
            hl_count += w.count_ones() as usize;
        }

        let styles_cap: StyleCountInt = if style_count == 0 {
            0
        } else {
            let mut c = 1usize;
            while c < style_count {
                c <<= 1;
            }
            if c < 16 {
                c = 16;
            }
            c as StyleCountInt
        };

        let hl_set_cap: usize = if hl_count == 0 {
            0
        } else {
            let mut c = 1usize;
            while c < hl_count {
                c <<= 1;
            }
            if c < HYPERLINK_COUNT_DEFAULT as usize {
                c = HYPERLINK_COUNT_DEFAULT as usize;
            }
            c
        };

        let hl_map_min = if hyperlink_cells == 0 {
            0
        } else {
            (hyperlink_cells + HYPERLINK_CELL_MULTIPLIER - 1) / HYPERLINK_CELL_MULTIPLIER
        };

        let hl_cap = if hl_set_cap > hl_map_min {
            hl_set_cap
        } else {
            hl_map_min
        };

        let hl_bytes = hl_cap * core::mem::size_of::<HyperlinkPageEntry>();

        PageCapacity {
            cols: self.size.cols,
            rows: (y_end - y_start) as CellCountInt,
            styles: styles_cap,
            grapheme_bytes: grapheme_bytes as GraphemeBytesInt,
            hyperlink_bytes: hl_bytes as HyperlinkCountInt,
            string_bytes: string_bytes as StringBytesInt,
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

            let same_page = core::ptr::eq(other as *const Page, self as *const Page);

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
                        if let Some((cps_ptr, cps_len)) = other_ref.lookup_grapheme(sc) {
                            self.set_graphemes(dst_row, dc, cps_ptr, cps_len)?;
                        }
                    }

                    if (*sc).hyperlink() {
                        if let Some(id) = other_ref.lookup_hyperlink(sc) {
                            if same_page {
                                if self.hyperlink_set.capacity() > 0 {
                                    let base = self.memory;
                                    let set: *mut RefCountedSet = &mut self.hyperlink_set;
                                    (*set).add_id(base, id);
                                }
                                self.set_hyperlink(dst_row, dc, id)?;
                            } else {
                                if self.hyperlink_count() >= self.hyperlink_capacity() {
                                    return Err("HyperlinkMapOutOfMemory");
                                }

                                let dst_id = if self.hyperlink_set.capacity() > 0
                                    && other_ref.hyperlink_set.capacity() > 0
                                {
                                    let other_base = other_ref.memory;
                                    let other_set: *const RefCountedSet = &other_ref.hyperlink_set;
                                    let _other_entry: HyperlinkPageEntry =
                                        (*other_set).get::<HyperlinkPageEntry>(
                                            other_base as *mut u8,
                                            id,
                                        );

                                    let our_base = self.memory;
                                    let our_set: *mut RefCountedSet = &mut self.hyperlink_set;
                                    let rc =
                                        (*our_set).ref_count(our_base as *const u8, id);
                                    if rc > 0 {
                                        (*our_set).add_id(our_base, id);
                                        id
                                    } else {
                                        let entry: HyperlinkPageEntry = (*other_set)
                                            .get::<HyperlinkPageEntry>(
                                                other_base as *mut u8,
                                                id,
                                            );
                                        (*our_set)
                                            .next_id::<HyperlinkPageEntry>(our_base, entry)
                                            .ok_or("HyperlinkSetOutOfMemory")?
                                    }
                                } else {
                                    id
                                };

                                self.set_hyperlink(dst_row, dc, dst_id)?;
                            }
                        }
                    }

                    if (*sc).style_id() != STYLE_DEFAULT_ID {
                        (*dst_row).set_styled(true);

                        if same_page {
                            if self.styles.capacity() > 0 {
                                let base = self.memory;
                                let set: *mut RefCountedSet = &mut self.styles;
                                (*set).add_id(base, (*sc).style_id());
                            }
                            (*dc).set_style_id((*sc).style_id());
                        } else {
                            if self.styles.capacity() > 0 && other_ref.styles.capacity() > 0 {
                                let other_base = other_ref.memory;
                                let other_set: *const RefCountedSet = &other_ref.styles;
                                let style: Style = (*other_set)
                                    .get::<Style>(other_base as *mut u8, (*sc).style_id());

                                let our_base = self.memory;
                                let our_set: *mut RefCountedSet = &mut self.styles;
                                let rc = (*our_set)
                                    .ref_count(our_base as *const u8, (*sc).style_id());
                                if rc > 0 {
                                    (*our_set).add_id(our_base, (*sc).style_id());
                                    (*dc).set_style_id((*sc).style_id());
                                } else if let Some(new_id) =
                                    (*our_set).next_id::<Style>(our_base, style)
                                {
                                    (*dc).set_style_id(new_id);
                                } else {
                                    return Err("StyleSetOutOfMemory");
                                }
                            }
                        }
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
