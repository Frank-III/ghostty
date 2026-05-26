use core::ptr;

use crate::bitmap_allocator::*;
use crate::early::*;
use crate::page_types::*;
use crate::size_types::*;

pub const PAGE_SIZE_MIN: usize = 4096;
pub const GRAPHEME_CHUNK_LEN: usize = 4;
pub const GRAPHEME_CHUNK: usize = GRAPHEME_CHUNK_LEN * 4;
pub const STRING_CHUNK_LEN: usize = 32;
pub const STRING_CHUNK: usize = STRING_CHUNK_LEN;
pub const HYPERLINK_COUNT_DEFAULT: u16 = 4;
pub const HYPERLINK_CELL_MULTIPLIER: usize = 16;

#[inline]
const fn align_forward(val: usize, alignment: usize) -> usize {
    (val + alignment - 1) & !(alignment - 1)
}

fn zero_offset() -> Offset {
    Offset { offset: 0 }
}

fn zero_bitmap_allocator<const N: usize>() -> BitmapAllocator<N> {
    BitmapAllocator {
        bitmap: zero_offset(),
        bitmap_count: 0,
        chunks: zero_offset(),
    }
}

pub struct PageLayout {
    pub total_size: usize,
    pub rows_start: usize,
    pub rows_size: usize,
    pub cells_start: usize,
    pub cells_size: usize,
    pub styles_start: usize,
    pub grapheme_alloc_start: usize,
    pub grapheme_map_start: usize,
    pub string_alloc_start: usize,
    pub hyperlink_map_start: usize,
    pub hyperlink_set_start: usize,
    pub capacity: PageCapacity,
}

pub struct Page {
    pub memory: *mut u8,
    pub memory_len: usize,
    pub rows: Offset,
    pub cells: Offset,
    pub dirty: bool,
    pub string_alloc: BitmapAllocator<STRING_CHUNK>,
    pub grapheme_alloc: BitmapAllocator<GRAPHEME_CHUNK>,
    pub grapheme_map: usize,
    pub styles: usize,
    pub hyperlink_map: usize,
    pub hyperlink_set: usize,
    pub size: PageSize,
    pub capacity: PageCapacity,
}

impl Page {
    pub fn layout(cap: PageCapacity) -> PageLayout {
        let rows_count = cap.rows as usize;
        let rows_start: usize = 0;
        let rows_end = rows_start + rows_count * core::mem::size_of::<Row>();

        let cells_count = cap.cols as usize * cap.rows as usize;
        let cells_start = align_forward(rows_end, core::mem::align_of::<u64>());
        let cells_end = cells_start + cells_count * core::mem::size_of::<Cell>();

        let styles_start = align_forward(cells_end, 8);
        let styles_end = styles_start;

        let grapheme_alloc_start = align_forward(styles_end, 8);
        let grapheme_alloc_end = grapheme_alloc_start;

        let grapheme_map_start = align_forward(grapheme_alloc_end, 8);
        let grapheme_map_end = grapheme_map_start;

        let string_alloc_start = align_forward(grapheme_map_end, 8);
        let string_alloc_end = string_alloc_start;

        let hyperlink_set_start = align_forward(string_alloc_end, 8);
        let hyperlink_set_end = hyperlink_set_start;

        let hyperlink_map_start = align_forward(hyperlink_set_end, 8);
        let hyperlink_map_end = hyperlink_map_start;

        let total_size = align_forward(hyperlink_map_end, PAGE_SIZE_MIN);

        PageLayout {
            total_size,
            rows_start,
            rows_size: rows_end - rows_start,
            cells_start,
            cells_size: cells_end - cells_start,
            styles_start,
            grapheme_alloc_start,
            grapheme_map_start,
            string_alloc_start,
            hyperlink_map_start,
            hyperlink_set_start,
            capacity: cap,
        }
    }

    pub fn init(cap: PageCapacity) -> Result<Page, ()> {
        let l = Self::layout(cap);
        debug_assert!(l.total_size % PAGE_SIZE_MIN == 0);

        let backing = page_alloc(l.total_size)?;
        let base = backing.as_mut_ptr();

        let buf = OffsetBuf::init(base);
        Ok(Self::init_buf(buf, l))
    }

    pub fn init_buf(buf: OffsetBuf, l: PageLayout) -> Page {
        let cap = l.capacity;
        let rows = buf.member(l.rows_start);
        let cells = buf.member(l.cells_start);
        let base = buf.start();

        unsafe {
            let cells_ptr: *mut Cell = cells.ptr_mut(base);
            let rows_ptr: *mut Row = rows.ptr_mut(base);

            for y in 0..cap.rows as usize {
                let start = y * cap.cols as usize;
                let cell_offset = get_offset(base, cells_ptr.add(start) as *const u8);
                let row = &mut *rows_ptr.add(y);
                row.set_cells(cell_offset);
            }
        }

        Page {
            memory: base,
            memory_len: l.total_size,
            rows,
            cells,
            dirty: false,
            string_alloc: zero_bitmap_allocator(),
            grapheme_alloc: zero_bitmap_allocator(),
            grapheme_map: 0,
            styles: 0,
            hyperlink_map: 0,
            hyperlink_set: 0,
            size: PageSize {
                cols: cap.cols,
                rows: cap.rows,
            },
            capacity: cap,
        }
    }

    pub fn deinit(&mut self) {
        if !self.memory.is_null() {
            let slice = unsafe { core::slice::from_raw_parts_mut(self.memory, self.memory_len) };
            page_free(slice);
        }
        self.memory = ptr::null_mut();
        self.memory_len = 0;
    }

    pub fn reinit(&mut self) {
        unsafe {
            let ptr64 = self.memory as *mut u64;
            let len64 = self.memory_len / 8;
            for i in 0..len64 {
                ptr::write(ptr64.add(i), 0);
            }
        }
        let cap = self.capacity;
        let l = Self::layout(cap);
        let buf = OffsetBuf::init(self.memory);
        *self = Self::init_buf(buf, l);
    }

    #[inline]
    pub fn get_row(&self, y: usize) -> *mut Row {
        debug_assert!(y < self.size.rows as usize);
        unsafe {
            let rows_ptr: *mut Row = self.rows.ptr_mut(self.memory);
            rows_ptr.add(y)
        }
    }

    #[inline]
    pub fn get_cells(&self, row: *const Row) -> &[Cell] {
        unsafe {
            let r = &*row;
            let cells_offset = r.cells();
            let cells_ptr: *const Cell = cells_offset.ptr(self.memory);
            core::slice::from_raw_parts(cells_ptr, self.size.cols as usize)
        }
    }

    #[inline]
    pub fn get_cells_mut(&self, row: *mut Row) -> &mut [Cell] {
        unsafe {
            let r = &*row;
            let cells_offset = r.cells();
            let cells_ptr: *mut Cell = cells_offset.ptr_mut(self.memory);
            core::slice::from_raw_parts_mut(cells_ptr, self.size.cols as usize)
        }
    }

    #[inline]
    pub fn row_cells_ptr(&self, row: *const Row) -> *mut Cell {
        unsafe {
            let r = &*row;
            r.cells().ptr_mut(self.memory)
        }
    }

    #[inline]
    pub fn rows_ptr(&self) -> *mut Row {
        unsafe { self.rows.ptr_mut(self.memory) }
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        if self.dirty {
            return true;
        }
        unsafe {
            let rows_ptr: *const Row = self.rows.ptr(self.memory);
            for i in 0..self.size.rows as usize {
                let row = &*rows_ptr.add(i);
                if row.dirty() {
                    return true;
                }
            }
        }
        false
    }

    #[inline]
    pub fn pause_integrity_checks(&mut self, _v: bool) {}

    #[inline]
    pub fn assert_integrity(&self) {}

    #[inline]
    pub fn verify_integrity(&self) {}
}

pub fn std_capacity() -> PageCapacity {
    PageCapacity {
        cols: 215,
        rows: 215,
        styles: 16,
        hyperlink_bytes: 0,
        grapheme_bytes: 0,
        string_bytes: 0,
    }
}
