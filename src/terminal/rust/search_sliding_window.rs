use core::ffi::c_void;
use core::ptr;

use crate::allocator::{GhosttyAllocator, alloc_alloc_impl, alloc_free_impl};
use crate::cell::cell_codepoint;
use crate::page_list_types::{PageList, PageListNode};
use crate::page_types::{Cell, Row};
use crate::point::Coordinate;
use crate::size_types::CellCountInt;
use crate::highlight::{FlattenedChunk, HighlightFlattened};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward = 0,
    Reverse = 1,
}

pub struct Meta {
    pub node: *mut PageListNode,
    pub serial: u64,
    pub cell_map_ptr: *mut Coordinate,
    pub cell_map_len: usize,
    pub cell_map_cap: usize,
}

impl Meta {
    pub fn cell_map(&self) -> &[Coordinate] {
        if self.cell_map_ptr.is_null() || self.cell_map_len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.cell_map_ptr, self.cell_map_len) }
    }

    pub fn cell_map_mut(&mut self) -> &mut [Coordinate] {
        if self.cell_map_ptr.is_null() || self.cell_map_len == 0 {
            return &mut [];
        }
        unsafe { core::slice::from_raw_parts_mut(self.cell_map_ptr, self.cell_map_len) }
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.cell_map_ptr.is_null() && self.cell_map_cap > 0 {
            unsafe {
                alloc_free_impl(
                    alloc,
                    self.cell_map_ptr as *mut u8,
                    self.cell_map_cap * core::mem::size_of::<Coordinate>(),
                );
            }
            self.cell_map_ptr = ptr::null_mut();
            self.cell_map_len = 0;
            self.cell_map_cap = 0;
        }
    }
}

pub struct CircBuf {
    pub buf_ptr: *mut u8,
    pub buf_cap: usize,
    pub head: usize,
    pub len_val: usize,
}

impl CircBuf {
    pub fn empty() -> Self {
        Self {
            buf_ptr: ptr::null_mut(),
            buf_cap: 0,
            head: 0,
            len_val: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len_val
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.len_val = 0;
    }

    pub unsafe fn ensure_unused_capacity(
        &mut self,
        alloc: *const GhosttyAllocator,
        additional: usize,
    ) -> bool {
        let needed = self.len_val + additional;
        if needed <= self.buf_cap {
            return true;
        }
        let new_cap = if self.buf_cap == 0 {
            needed
        } else {
            let mut c = self.buf_cap;
            while c < needed {
                c = c * 2;
            }
            c
        };
        unsafe {
            let new_buf = alloc_alloc_impl(alloc, new_cap);
            if new_buf.is_null() {
                return false;
            }
            if self.len_val > 0 && !self.buf_ptr.is_null() {
                let off = self.head;
                let first_len = if off + self.len_val <= self.buf_cap {
                    self.len_val
                } else {
                    self.buf_cap - off
                };
                ptr::copy_nonoverlapping(self.buf_ptr.add(off), new_buf, first_len);
                if first_len < self.len_val {
                    ptr::copy_nonoverlapping(
                        self.buf_ptr,
                        new_buf.add(first_len),
                        self.len_val - first_len,
                    );
                }
                alloc_free_impl(alloc, self.buf_ptr, self.buf_cap);
            }
            self.buf_ptr = new_buf;
            self.buf_cap = new_cap;
            self.head = 0;
        }
        true
    }

    pub fn get_slices(&self) -> (*const u8, usize, *const u8, usize) {
        if self.len_val == 0 || self.buf_ptr.is_null() {
            return (ptr::null(), 0, ptr::null(), 0);
        }
        let off = self.head;
        let first_len = if off + self.len_val <= self.buf_cap {
            self.len_val
        } else {
            self.buf_cap - off
        };
        let second_len = self.len_val - first_len;
        unsafe {
            let s0 = self.buf_ptr.add(off);
            let s1 = self.buf_ptr;
            (s0, first_len, s1, second_len)
        }
    }

    pub unsafe fn append_slice(&mut self, data: &[u8]) {
        let n = data.len();
        if n == 0 {
            return;
        }
        let write_pos = (self.head + self.len_val) % self.buf_cap;
        unsafe {
            if write_pos + n <= self.buf_cap {
                ptr::copy_nonoverlapping(data.as_ptr(), self.buf_ptr.add(write_pos), n);
            } else {
                let first = self.buf_cap - write_pos;
                ptr::copy_nonoverlapping(data.as_ptr(), self.buf_ptr.add(write_pos), first);
                ptr::copy_nonoverlapping(data.as_ptr().add(first), self.buf_ptr, n - first);
            }
        }
        self.len_val += n;
    }

    pub unsafe fn delete_oldest(&mut self, count: usize) {
        if count == 0 || count > self.len_val {
            return;
        }
        self.head = (self.head + count) % self.buf_cap;
        self.len_val -= count;
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.buf_ptr.is_null() && self.buf_cap > 0 {
            unsafe {
                alloc_free_impl(alloc, self.buf_ptr, self.buf_cap);
            }
            self.buf_ptr = ptr::null_mut();
            self.buf_cap = 0;
            self.head = 0;
            self.len_val = 0;
        }
    }
}

pub struct MetaBuf {
    pub buf_ptr: *mut Meta,
    pub buf_cap: usize,
    pub head: usize,
    pub len_val: usize,
}

impl MetaBuf {
    pub fn empty() -> Self {
        Self {
            buf_ptr: ptr::null_mut(),
            buf_cap: 0,
            head: 0,
            len_val: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len_val
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.len_val = 0;
    }

    pub unsafe fn ensure_unused_capacity(
        &mut self,
        alloc: *const GhosttyAllocator,
        additional: usize,
    ) -> bool {
        let needed = self.len_val + additional;
        if needed <= self.buf_cap {
            return true;
        }
        let new_cap = if self.buf_cap == 0 {
            needed
        } else {
            let mut c = self.buf_cap;
            while c < needed {
                c = c * 2;
            }
            c
        };
        unsafe {
            let byte_size = new_cap * core::mem::size_of::<Meta>();
            let new_buf = alloc_alloc_impl(alloc, byte_size) as *mut Meta;
            if new_buf.is_null() {
                return false;
            }
            if self.len_val > 0 && !self.buf_ptr.is_null() {
                let off = self.head;
                let first_len = if off + self.len_val <= self.buf_cap {
                    self.len_val
                } else {
                    self.buf_cap - off
                };
                ptr::copy_nonoverlapping(self.buf_ptr.add(off), new_buf, first_len);
                if first_len < self.len_val {
                    ptr::copy_nonoverlapping(
                        self.buf_ptr,
                        new_buf.add(first_len),
                        self.len_val - first_len,
                    );
                }
                let old_byte_size = self.buf_cap * core::mem::size_of::<Meta>();
                alloc_free_impl(alloc, self.buf_ptr as *mut u8, old_byte_size);
            }
            self.buf_ptr = new_buf;
            self.buf_cap = new_cap;
            self.head = 0;
        }
        true
    }

    pub unsafe fn get(&self, idx: usize) -> *mut Meta {
        let actual = (self.head + idx) % self.buf_cap;
        unsafe { self.buf_ptr.add(actual) }
    }

    pub unsafe fn append(&mut self, meta: Meta) {
        let write_pos = (self.head + self.len_val) % self.buf_cap;
        unsafe {
            ptr::write(self.buf_ptr.add(write_pos), meta);
        }
        self.len_val += 1;
    }

    pub unsafe fn delete_oldest(&mut self, count: usize) {
        if count == 0 || count > self.len_val {
            return;
        }
        self.head = (self.head + count) % self.buf_cap;
        self.len_val -= count;
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.buf_ptr.is_null() && self.buf_cap > 0 {
            let byte_size = self.buf_cap * core::mem::size_of::<Meta>();
            unsafe {
                alloc_free_impl(alloc, self.buf_ptr as *mut u8, byte_size);
            }
            self.buf_ptr = ptr::null_mut();
            self.buf_cap = 0;
            self.head = 0;
            self.len_val = 0;
        }
    }
}

pub struct ChunkBuf {
    pub ptr: *mut FlattenedChunk,
    pub len_val: usize,
    pub cap: usize,
}

impl ChunkBuf {
    pub fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len_val: 0,
            cap: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len_val
    }

    pub fn clear(&mut self) {
        self.len_val = 0;
    }

    pub fn chunks(&self) -> &[FlattenedChunk] {
        if self.ptr.is_null() || self.len_val == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.ptr, self.len_val) }
    }

    pub fn chunks_mut(&mut self) -> &mut [FlattenedChunk] {
        if self.ptr.is_null() || self.len_val == 0 {
            return &mut [];
        }
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len_val) }
    }

    pub unsafe fn ensure_total_capacity(
        &mut self,
        alloc: *const GhosttyAllocator,
        needed: usize,
    ) -> bool {
        if needed <= self.cap {
            return true;
        }
        let new_cap = if self.cap == 0 {
            needed
        } else {
            let mut c = self.cap;
            while c < needed {
                c *= 2;
            }
            c
        };
        let byte_size = new_cap * core::mem::size_of::<FlattenedChunk>();
        unsafe {
            let new_ptr = alloc_alloc_impl(alloc, byte_size) as *mut FlattenedChunk;
            if new_ptr.is_null() {
                return false;
            }
            if self.len_val > 0 && !self.ptr.is_null() {
                ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len_val);
                let old_size = self.cap * core::mem::size_of::<FlattenedChunk>();
                alloc_free_impl(alloc, self.ptr as *mut u8, old_size);
            }
            self.ptr = new_ptr;
            self.cap = new_cap;
        }
        true
    }

    pub unsafe fn push(&mut self, chunk: FlattenedChunk) {
        if self.len_val < self.cap {
            unsafe {
                ptr::write(self.ptr.add(self.len_val), chunk);
            }
            self.len_val += 1;
        }
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.ptr.is_null() && self.cap > 0 {
            let byte_size = self.cap * core::mem::size_of::<FlattenedChunk>();
            unsafe {
                alloc_free_impl(alloc, self.ptr as *mut u8, byte_size);
            }
            self.ptr = ptr::null_mut();
            self.len_val = 0;
            self.cap = 0;
        }
    }
}

pub struct SlidingWindow {
    pub alloc: *const GhosttyAllocator,
    pub data: CircBuf,
    pub meta: MetaBuf,
    pub chunk_buf: ChunkBuf,
    pub data_offset: usize,
    pub needle_ptr: *const u8,
    pub needle_len: usize,
    pub direction: Direction,
    pub overlap_buf: *mut u8,
    pub overlap_buf_len: usize,
}

impl SlidingWindow {
    pub unsafe fn init(
        alloc: *const GhosttyAllocator,
        direction: Direction,
        needle: &[u8],
    ) -> *mut SlidingWindow {
        let n = needle.len();
        let sw_size = core::mem::size_of::<SlidingWindow>();
        unsafe {
            let sw_ptr = alloc_alloc_impl(alloc, sw_size) as *mut SlidingWindow;
            if sw_ptr.is_null() {
                return ptr::null_mut();
            }

            let needle_copy = if n > 0 {
                let p = alloc_alloc_impl(alloc, n);
                if p.is_null() {
                    alloc_free_impl(alloc, sw_ptr as *mut u8, sw_size);
                    return ptr::null_mut();
                }
                ptr::copy_nonoverlapping(needle.as_ptr(), p, n);
                if direction == Direction::Reverse {
                    reverse_bytes(p, n);
                }
                p
            } else {
                ptr::null_mut()
            };

            let overlap_len = n * 2;
            let overlap_buf = if overlap_len > 0 {
                let p = alloc_alloc_impl(alloc, overlap_len);
                if p.is_null() {
                    if !needle_copy.is_null() {
                        alloc_free_impl(alloc, needle_copy, n);
                    }
                    alloc_free_impl(alloc, sw_ptr as *mut u8, sw_size);
                    return ptr::null_mut();
                }
                p
            } else {
                ptr::null_mut()
            };

            ptr::write(sw_ptr, SlidingWindow {
                alloc,
                data: CircBuf::empty(),
                meta: MetaBuf::empty(),
                chunk_buf: ChunkBuf::empty(),
                data_offset: 0,
                needle_ptr: needle_copy,
                needle_len: n,
                direction,
                overlap_buf,
                overlap_buf_len: overlap_len,
            });

            sw_ptr
        }
    }

    pub unsafe fn deinit(sw: *mut SlidingWindow) {
        if sw.is_null() {
            return;
        }
        unsafe {
            let alloc = (*sw).alloc;

            if !(*sw).overlap_buf.is_null() && (*sw).overlap_buf_len > 0 {
                alloc_free_impl(alloc, (*sw).overlap_buf, (*sw).overlap_buf_len);
            }

            if !(*sw).needle_ptr.is_null() && (*sw).needle_len > 0 {
                alloc_free_impl(alloc, (*sw).needle_ptr as *mut u8, (*sw).needle_len);
            }

            (*sw).chunk_buf.deinit(alloc);

            let meta_len = (*sw).meta.len();
            for i in 0..meta_len {
                let m = (*sw).meta.get(i);
                (*m).deinit(alloc);
            }
            (*sw).meta.deinit(alloc);
            (*sw).data.deinit(alloc);

            let sw_size = core::mem::size_of::<SlidingWindow>();
            alloc_free_impl(alloc, sw as *mut u8, sw_size);
        }
    }

    pub unsafe fn clear_and_retain_capacity(sw: *mut SlidingWindow) {
        if sw.is_null() {
            return;
        }
        unsafe {
            let alloc = (*sw).alloc;
            let meta_len = (*sw).meta.len();
            for i in 0..meta_len {
                let m = (*sw).meta.get(i);
                (*m).deinit(alloc);
            }
            (*sw).meta.clear();
            (*sw).data.clear();
            (*sw).data_offset = 0;
        }
    }

    pub unsafe fn needle_slice(sw: *const SlidingWindow) -> &'static [u8] {
        unsafe {
            if (*sw).needle_ptr.is_null() || (*sw).needle_len == 0 {
                return &[];
            }
            core::slice::from_raw_parts((*sw).needle_ptr, (*sw).needle_len)
        }
    }

    pub unsafe fn next(sw: *mut SlidingWindow) -> HighlightFlattened {
        let empty = HighlightFlattened::empty();
        if sw.is_null() {
            return empty;
        }
        unsafe {
            let needle = Self::needle_slice(sw);
            let needle_len = needle.len();
            let data_len = (*sw).data.len();

            if data_len < needle_len {
                return empty;
            }

            let avail = data_len - (*sw).data_offset;
            if avail < needle_len {
                return empty;
            }

            let (s0_ptr, s0_len, s1_ptr, s1_len) = (*sw).data.get_slices();

            let off = (*sw).data_offset;
            let (search0_ptr, search0_len, search1_ptr, search1_len) = if off == 0 {
                (s0_ptr, s0_len, s1_ptr, s1_len)
            } else if off < s0_len {
                let rem0 = s0_len - off;
                (s0_ptr.add(off), rem0, s1_ptr, s1_len)
            } else {
                let skip1 = off - s0_len;
                if skip1 < s1_len {
                    (s1_ptr.add(skip1), s1_len - skip1, ptr::null(), 0)
                } else {
                    (ptr::null(), 0, ptr::null(), 0)
                }
            };

            if search0_len > 0 {
                if let Some(idx) = ascii_index_of_ignore_case(
                    core::slice::from_raw_parts(search0_ptr, search0_len),
                    needle,
                ) {
                    return Self::highlight(sw, idx, needle_len);
                }
            }

            if search0_len > 0 && search1_len > 0 && needle_len > 1 {
                let prefix_len = min_usize(search0_len, needle_len - 1);
                let suffix_len = min_usize(search1_len, needle_len - 1);
                let overlap_len = prefix_len + suffix_len;

                if overlap_len <= (*sw).overlap_buf_len {
                    let prefix_start = search0_ptr.add(search0_len - prefix_len);
                    ptr::copy_nonoverlapping(prefix_start, (*sw).overlap_buf, prefix_len);
                    ptr::copy_nonoverlapping(
                        search1_ptr,
                        (*sw).overlap_buf.add(prefix_len),
                        suffix_len,
                    );

                    let overlap_slice = core::slice::from_raw_parts(
                        (*sw).overlap_buf,
                        overlap_len,
                    );
                    if let Some(idx) = ascii_index_of_ignore_case(overlap_slice, needle) {
                        let abs_idx = search0_len - prefix_len + idx;
                        return Self::highlight(sw, abs_idx, needle_len);
                    }
                }
            }

            if search1_len > 0 {
                if let Some(idx) = ascii_index_of_ignore_case(
                    core::slice::from_raw_parts(search1_ptr, search1_len),
                    needle,
                ) {
                    return Self::highlight(sw, search0_len + idx, needle_len);
                }
            }

            if needle_len == 1 {
                Self::clear_and_retain_capacity(sw);
                return empty;
            }

            {
                let meta_len = (*sw).meta.len();
                let mut saved: usize = 0;
                let mut found_meta_idx: usize = meta_len;
                let mut _found_data_off: usize = 0;

                for ri in 0..meta_len {
                    let i = meta_len - 1 - ri;
                    let m = (*sw).meta.get(i);
                    let cm_len = (*m).cell_map_len;
                    let needed = if needle_len - 1 > saved {
                        needle_len - 1 - saved
                    } else {
                        0
                    };
                    if needed == 0 {
                        break;
                    }
                    if cm_len >= needed {
                        found_meta_idx = i;
                        _found_data_off = cm_len - needed;
                        break;
                    }
                    saved += cm_len;
                }

                if found_meta_idx < meta_len {
                    let prune_count = found_meta_idx;
                    if prune_count > 0 {
                        let mut prune_data_len: usize = 0;
                        for j in 0..prune_count {
                            let m = (*sw).meta.get(j);
                            prune_data_len += (*m).cell_map_len;
                            (*m).deinit((*sw).alloc);
                        }
                        (*sw).meta.delete_oldest(prune_count);
                        (*sw).data.delete_oldest(prune_data_len);
                    }
                }
            }

            let data_len_after = (*sw).data.len();
            if data_len_after >= needle_len {
                (*sw).data_offset = data_len_after - needle_len + 1;
            } else {
                (*sw).data_offset = 0;
            }

            empty
        }
    }

    unsafe fn highlight(
        sw: *mut SlidingWindow,
        start_offset: usize,
        len: usize,
    ) -> HighlightFlattened {
        unsafe {
            let start = start_offset + (*sw).data_offset;
            let end = start + len - 1;

            (*sw).chunk_buf.clear();
            let mut result = HighlightFlattened::empty();

            let meta_len = (*sw).meta.len();
            let mut meta_consumed: usize = 0;
            let mut prune_meta: usize = 0;
            let mut prune_data: usize = 0;
            let mut found_start = false;
            let mut need_end = false;
            let mut br_meta_consumed: usize = 0;
            let mut br_start_idx: usize = 0;

            for mi in 0..meta_len {
                let m = (*sw).meta.get(mi);
                let prior_consumed = meta_consumed;
                meta_consumed += (*m).cell_map_len;

                let meta_i = if start >= prior_consumed {
                    start - prior_consumed
                } else {
                    continue;
                };

                if meta_i >= (*m).cell_map_len {
                    continue;
                }

                let end_i = if end >= prior_consumed {
                    end - prior_consumed
                } else {
                    0
                };

                if end_i < (*m).cell_map_len {
                    let cm = (*m).cell_map();
                    let start_map = *cm.get_unchecked(meta_i);
                    let end_map = *cm.get_unchecked(end_i);
                    result.top_x = start_map.x;
                    result.bot_x = end_map.x;

                    let chunk = FlattenedChunk {
                        node: (*m).node,
                        serial: (*m).serial,
                        start: start_map.y as CellCountInt,
                        end: (end_map.y + 1) as CellCountInt,
                    };
                    (*sw).chunk_buf.push(chunk);

                    prune_meta = mi;
                    prune_data = prior_consumed;
                    found_start = true;
                    break;
                } else {
                    let cm = (*m).cell_map();
                    let map = *cm.get_unchecked(meta_i);
                    result.top_x = map.x;

                    let node = (*m).node;
                    let node_rows = if !node.is_null() {
                        (*node).data.size.rows
                    } else {
                        0
                    };

                    let chunk = FlattenedChunk {
                        node: (*m).node,
                        serial: (*m).serial,
                        start: map.y as CellCountInt,
                        end: node_rows,
                    };
                    (*sw).chunk_buf.push(chunk);

                    prune_meta = mi;
                    prune_data = prior_consumed;
                    need_end = true;
                    br_meta_consumed = meta_consumed;
                    br_start_idx = mi + 1;
                    found_start = true;
                    break;
                }
            }

            if !found_start {
                return result;
            }

            if need_end {
                for mi in br_start_idx..meta_len {
                    let m = (*sw).meta.get(mi);
                    let meta_i = if end >= br_meta_consumed {
                        end - br_meta_consumed
                    } else {
                        0
                    };

                    if meta_i >= (*m).cell_map_len {
                        let node = (*m).node;
                        let node_rows = if !node.is_null() {
                            (*node).data.size.rows
                        } else {
                            0
                        };
                        let chunk = FlattenedChunk {
                            node: (*m).node,
                            serial: (*m).serial,
                            start: 0,
                            end: node_rows,
                        };
                        (*sw).chunk_buf.push(chunk);
                        br_meta_consumed += (*m).cell_map_len;
                        continue;
                    }

                    let cm = (*m).cell_map();
                    let map = *cm.get_unchecked(meta_i);
                    result.bot_x = map.x;
                    let chunk = FlattenedChunk {
                        node: (*m).node,
                        serial: (*m).serial,
                        start: 0,
                        end: (map.y + 1) as CellCountInt,
                    };
                    (*sw).chunk_buf.push(chunk);
                    break;
                }
            }

            (*sw).data_offset = start - prune_data + 1;

            if prune_meta > 0 {
                for j in 0..prune_meta {
                    let m = (*sw).meta.get(j);
                    (*m).deinit((*sw).alloc);
                }
                (*sw).meta.delete_oldest(prune_meta);
                if prune_data > 0 {
                    (*sw).data.delete_oldest(prune_data);
                }
            }

            if (*sw).direction == Direction::Reverse {
                Self::reverse_chunks(sw, &mut result);
            }

            result.chunks_ptr = (*sw).chunk_buf.ptr;
            result.chunks_len = (*sw).chunk_buf.len_val;
            result.chunks_cap = (*sw).chunk_buf.cap;

            result
        }
    }

    unsafe fn reverse_chunks(sw: *mut SlidingWindow, result: &mut HighlightFlattened) {
        unsafe {
            let chunks = (*sw).chunk_buf.chunks_mut();
            let n = chunks.len();
            if n == 0 {
                return;
            }

            if n > 1 {
                let mut i = 0;
                let mut j = n - 1;
                while i < j {
                    let tmp = *chunks.get_unchecked(i);
                    let tmp2 = *chunks.get_unchecked(j);
                    *chunks.get_unchecked_mut(i) = tmp2;
                    *chunks.get_unchecked_mut(j) = tmp;
                    i += 1;
                    j -= 1;
                }

                let first_node = chunks.get_unchecked(0).node;
                let first_rows = if !first_node.is_null() {
                    (*first_node).data.size.rows
                } else {
                    0
                };
                let first_end = chunks.get_unchecked(0).end;
                let first_start = first_end - 1;
                chunks.get_unchecked_mut(0).start = first_start;
                chunks.get_unchecked_mut(0).end = first_rows;

                let last_start = chunks.get_unchecked(n - 1).start;
                chunks.get_unchecked_mut(n - 1).end = last_start + 1;
                chunks.get_unchecked_mut(n - 1).start = 0;
            } else {
                let start_y = chunks.get_unchecked(0).start;
                let end_y = chunks.get_unchecked(0).end;
                chunks.get_unchecked_mut(0).start = end_y - 1;
                chunks.get_unchecked_mut(0).end = start_y + 1;
            }

            let top_x = result.top_x;
            result.top_x = result.bot_x;
            result.bot_x = top_x;
        }
    }

    pub unsafe fn append(
        sw: *mut SlidingWindow,
        node: *mut PageListNode,
    ) -> usize {
        if sw.is_null() || node.is_null() {
            return 0;
        }
        unsafe {
            let alloc = (*sw).alloc;
            let serial = (*node).serial;

            let mut cell_map_ptr: *mut Coordinate;
            let mut cell_map_len: usize = 0;
            let mut cell_map_cap: usize;

            let initial_cap: usize = 64;
            let cm_byte_size = initial_cap * core::mem::size_of::<Coordinate>();
            cell_map_ptr = alloc_alloc_impl(alloc, cm_byte_size) as *mut Coordinate;
            if cell_map_ptr.is_null() {
                return 0;
            }
            cell_map_cap = initial_cap;

            let mut encoded_ptr: *mut u8;
            let mut encoded_len: usize = 0;
            let mut encoded_cap: usize;

            let enc_cap: usize = 256;
            encoded_ptr = alloc_alloc_impl(alloc, enc_cap);
            if encoded_ptr.is_null() {
                alloc_free_impl(alloc, cell_map_ptr as *mut u8, cm_byte_size);
                return 0;
            }
            encoded_cap = enc_cap;

            let page = &(*node).data;
            let rows = page.size.rows as usize;
            let cols = page.size.cols as usize;

            for y in 0..rows {
                let row_ptr = page.get_row(y);
                if row_ptr.is_null() {
                    continue;
                }
                let cells = page.get_cells(row_ptr);

                for x in 0..cols {
                    let cell = *cells.get_unchecked(x);
                    let ch = cell_codepoint(cell.0);
                    let byte = if ch == 0 || ch == 0x20 {
                        b' '
                    } else if ch < 128 {
                        ch as u8
                    } else {
                        b'?'
                    };

                    if encoded_len >= encoded_cap {
                        let new_cap = encoded_cap * 2;
                        let new_ptr = alloc_alloc_impl(alloc, new_cap);
                        if new_ptr.is_null() {
                            break;
                        }
                        ptr::copy_nonoverlapping(encoded_ptr, new_ptr, encoded_len);
                        alloc_free_impl(alloc, encoded_ptr, encoded_cap);
                        encoded_ptr = new_ptr;
                        encoded_cap = new_cap;
                    }
                    *encoded_ptr.add(encoded_len) = byte;
                    encoded_len += 1;

                    if cell_map_len >= cell_map_cap {
                        let new_cap = cell_map_cap * 2;
                        let new_byte = new_cap * core::mem::size_of::<Coordinate>();
                        let new_ptr = alloc_alloc_impl(alloc, new_byte) as *mut Coordinate;
                        if new_ptr.is_null() {
                            break;
                        }
                        ptr::copy_nonoverlapping(cell_map_ptr, new_ptr, cell_map_len);
                        let old_byte = cell_map_cap * core::mem::size_of::<Coordinate>();
                        alloc_free_impl(alloc, cell_map_ptr as *mut u8, old_byte);
                        cell_map_ptr = new_ptr;
                        cell_map_cap = new_cap;
                    }
                    let coord = Coordinate { x: x as CellCountInt, y: y as u32 };
                    ptr::write(cell_map_ptr.add(cell_map_len), coord);
                    cell_map_len += 1;
                }
            }

            let last_row_idx = if rows > 0 { rows - 1 } else { 0 };
            let last_row_ptr = page.get_row(last_row_idx);
            let last_wrap = if !last_row_ptr.is_null() {
                (*last_row_ptr).wrap()
            } else {
                false
            };
            if !last_wrap {
                if encoded_len >= encoded_cap {
                    let new_cap = encoded_cap * 2;
                    let new_ptr = alloc_alloc_impl(alloc, new_cap);
                    if !new_ptr.is_null() {
                        ptr::copy_nonoverlapping(encoded_ptr, new_ptr, encoded_len);
                        alloc_free_impl(alloc, encoded_ptr, encoded_cap);
                        encoded_ptr = new_ptr;
                        encoded_cap = new_cap;
                    }
                }
                if encoded_len < encoded_cap {
                    *encoded_ptr.add(encoded_len) = b'\n';
                    encoded_len += 1;
                }

                if cell_map_len >= cell_map_cap {
                    let new_cap = cell_map_cap * 2;
                    let new_byte = new_cap * core::mem::size_of::<Coordinate>();
                    let new_ptr = alloc_alloc_impl(alloc, new_byte) as *mut Coordinate;
                    if !new_ptr.is_null() {
                        ptr::copy_nonoverlapping(cell_map_ptr, new_ptr, cell_map_len);
                        let old_byte = cell_map_cap * core::mem::size_of::<Coordinate>();
                        alloc_free_impl(alloc, cell_map_ptr as *mut u8, old_byte);
                        cell_map_ptr = new_ptr;
                        cell_map_cap = new_cap;
                    }
                }
                if cell_map_len < cell_map_cap {
                    let last_coord = if cell_map_len > 0 {
                        *cell_map_ptr.add(cell_map_len - 1)
                    } else {
                        Coordinate { x: 0, y: 0 }
                    };
                    ptr::write(cell_map_ptr.add(cell_map_len), last_coord);
                    cell_map_len += 1;
                }
            }

            if encoded_len == 0 {
                if !encoded_ptr.is_null() {
                    alloc_free_impl(alloc, encoded_ptr, encoded_cap);
                }
                if !cell_map_ptr.is_null() {
                    let cm_byte = cell_map_cap * core::mem::size_of::<Coordinate>();
                    alloc_free_impl(alloc, cell_map_ptr as *mut u8, cm_byte);
                }
                return 0;
            }

            let written = core::slice::from_raw_parts(encoded_ptr, encoded_len);

            if (*sw).direction == Direction::Reverse {
                reverse_bytes(encoded_ptr, encoded_len);
                let cm = core::slice::from_raw_parts_mut(cell_map_ptr, cell_map_len);
                reverse_coordinates(cm);
            }

            let ok = (*sw).data.ensure_unused_capacity(alloc, encoded_len);
            let ok2 = (*sw).meta.ensure_unused_capacity(alloc, 1);
            let meta_cap_needed = (*sw).meta.len() + 1;
            let ok3 = (*sw).chunk_buf.ensure_total_capacity(alloc, meta_cap_needed);

            if !ok || !ok2 || !ok3 {
                if !encoded_ptr.is_null() {
                    alloc_free_impl(alloc, encoded_ptr, encoded_cap);
                }
                if !cell_map_ptr.is_null() {
                    let cm_byte = cell_map_cap * core::mem::size_of::<Coordinate>();
                    alloc_free_impl(alloc, cell_map_ptr as *mut u8, cm_byte);
                }
                return 0;
            }

            (*sw).data.append_slice(written);

            let meta = Meta {
                node,
                serial,
                cell_map_ptr,
                cell_map_len,
                cell_map_cap,
            };
            (*sw).meta.append(meta);

            if !encoded_ptr.is_null() {
                alloc_free_impl(alloc, encoded_ptr, encoded_cap);
            }

            encoded_len
        }
    }
}

fn min_usize(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

unsafe fn reverse_bytes(ptr: *mut u8, len: usize) {
    if len <= 1 {
        return;
    }
    unsafe {
        let mut i = 0;
        let mut j = len - 1;
        while i < j {
            let tmp = *ptr.add(i);
            *ptr.add(i) = *ptr.add(j);
            *ptr.add(j) = tmp;
            i += 1;
            j -= 1;
        }
    }
}

fn reverse_coordinates(slice: &mut [Coordinate]) {
    let n = slice.len();
    if n <= 1 {
        return;
    }
    let mut i = 0;
    let mut j = n - 1;
    while i < j {
        let tmp = slice[i];
        slice[i] = slice[j];
        slice[j] = tmp;
        i += 1;
        j -= 1;
    }
}

fn ascii_to_lower(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

fn ascii_index_of_ignore_case(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    let h = haystack.len();
    if n == 0 || h < n {
        return None;
    }
    let last = h - n;
    let mut i = 0;
    while i <= last {
        let mut matched = true;
        let mut j = 0;
        while j < n {
            if ascii_to_lower(haystack[i + j]) != ascii_to_lower(needle[j]) {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return Some(i);
        }
        i += 1;
    }
    None
}
