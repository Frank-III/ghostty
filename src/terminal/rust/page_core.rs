use core::ptr;

use crate::bitmap_allocator::*;
use crate::early::*;
use crate::hash_map::{AutoContext, Layout as HashMapLayout, OffsetHashMap};
use crate::hyperlink::HyperlinkPageEntry;
use crate::page_types::*;
use crate::ref_counted_set::{RefCountedSet, RefCountedSetLayout};
use crate::size_types::*;
use crate::style_types::Style;

pub const PAGE_SIZE_MIN: usize = 4096;
pub const GRAPHEME_CHUNK_LEN: usize = 4;
pub const GRAPHEME_CHUNK: usize = GRAPHEME_CHUNK_LEN * 4;
pub const STRING_CHUNK_LEN: usize = 32;
pub const STRING_CHUNK: usize = STRING_CHUNK_LEN;
pub const HYPERLINK_COUNT_DEFAULT: u16 = 4;
pub const HYPERLINK_CELL_MULTIPLIER: usize = 16;

type GraphemeAlloc = BitmapAllocator<GRAPHEME_CHUNK>;
type StringAlloc = BitmapAllocator<STRING_CHUNK>;
type StyleSet = RefCountedSet;
type GraphemeMap = OffsetHashMap<OffsetInt, OffsetSlice, AutoContext>;
type HyperlinkSet = RefCountedSet;
type HyperlinkMap = OffsetHashMap<OffsetInt, u16, AutoContext>;

#[inline]
const fn align_forward(val: usize, alignment: usize) -> usize {
    (val + alignment - 1) & !(alignment - 1)
}

const fn div_ceil(a: usize, b: usize) -> usize {
    if b == 0 {
        return 0;
    }
    (a + b - 1) / b
}

const fn ceil_power_of_two_usize(v: usize) -> usize {
    if v == 0 {
        return 1;
    }
    let mut x = v - 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    #[cfg(target_pointer_width = "64")]
    {
        x |= x >> 32;
    }
    x + 1
}

const fn max_usize(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
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
    pub styles_layout: RefCountedSetLayout,
    pub grapheme_alloc_start: usize,
    pub grapheme_alloc_layout: Layout,
    pub grapheme_map_start: usize,
    pub grapheme_map_layout: HashMapLayout,
    pub string_alloc_start: usize,
    pub string_alloc_layout: Layout,
    pub hyperlink_set_start: usize,
    pub hyperlink_set_layout: RefCountedSetLayout,
    pub hyperlink_map_start: usize,
    pub hyperlink_map_layout: HashMapLayout,
    pub capacity: PageCapacity,
}

#[repr(C)]
pub struct Page {
    pub memory: *mut u8,
    pub memory_len: usize,
    pub rows: Offset,
    pub cells: Offset,
    pub dirty: bool,
    pub string_alloc: StringAlloc,
    pub grapheme_alloc: GraphemeAlloc,
    pub grapheme_map: GraphemeMap,
    pub styles: StyleSet,
    pub hyperlink_map: HyperlinkMap,
    pub hyperlink_set: HyperlinkSet,
    pub size: PageSize,
    pub capacity: PageCapacity,
}

impl Page {
    pub fn layout(cap: PageCapacity) -> PageLayout {
        let rows_count = cap.rows as usize;
        let rows_start: usize = 0;
        let rows_end = rows_start + rows_count * core::mem::size_of::<Row>();

        let cells_count = cap.cols as usize * cap.rows as usize;
        let cells_start = align_forward(rows_end, core::mem::align_of::<Cell>());
        let cells_end = cells_start + cells_count * core::mem::size_of::<Cell>();

        let styles_layout = RefCountedSetLayout::init(
            cap.styles,
            core::mem::size_of::<Style>(),
            core::mem::align_of::<Style>(),
        );
        let styles_align = max_usize(core::mem::align_of::<Style>(), core::mem::align_of::<u32>());
        let styles_start = align_forward(cells_end, styles_align);
        let styles_end = styles_start + styles_layout.total_size;

        let grapheme_alloc_layout = GraphemeAlloc::compute_layout(cap.grapheme_bytes as usize);
        let grapheme_alloc_start = align_forward(styles_end, GraphemeAlloc::BASE_ALIGN);
        let grapheme_alloc_end = grapheme_alloc_start + grapheme_alloc_layout.total_size;

        let grapheme_count: usize = if cap.grapheme_bytes == 0 {
            0
        } else {
            let base = div_ceil(cap.grapheme_bytes as usize, GRAPHEME_CHUNK);
            ceil_power_of_two_usize(base)
        };
        let grapheme_map_layout = GraphemeMap::layout(grapheme_count as u32);
        let grapheme_map_start = align_forward(grapheme_alloc_end, GraphemeMap::BASE_ALIGN);
        let grapheme_map_end = grapheme_map_start + grapheme_map_layout.total_size;

        let string_alloc_layout = StringAlloc::compute_layout(cap.string_bytes as usize);
        let string_alloc_start = align_forward(grapheme_map_end, StringAlloc::BASE_ALIGN);
        let string_alloc_end = string_alloc_start + string_alloc_layout.total_size;

        let hyperlink_item_size = core::mem::size_of::<HyperlinkPageEntry>();
        let hyperlink_count: usize = if hyperlink_item_size == 0 {
            0
        } else {
            (cap.hyperlink_bytes as usize) / hyperlink_item_size
        };
        let hyperlink_set_layout = RefCountedSetLayout::init(
            hyperlink_count as u16,
            hyperlink_item_size,
            core::mem::align_of::<HyperlinkPageEntry>(),
        );
        let hyperlink_set_align = max_usize(
            core::mem::align_of::<HyperlinkPageEntry>(),
            core::mem::align_of::<u32>(),
        );
        let hyperlink_set_start = align_forward(string_alloc_end, hyperlink_set_align);
        let hyperlink_set_end = hyperlink_set_start + hyperlink_set_layout.total_size;

        let hyperlink_map_count: u32 = if hyperlink_count == 0 {
            0
        } else {
            let mult = hyperlink_count * HYPERLINK_CELL_MULTIPLIER;
            let mult32 = if mult > (u32::MAX as usize) {
                u32::MAX
            } else {
                mult as u32
            };
            ceil_power_of_two_usize(mult32 as usize) as u32
        };
        let hyperlink_map_layout = HyperlinkMap::layout(hyperlink_map_count);
        let hyperlink_map_start = align_forward(hyperlink_set_end, HyperlinkMap::BASE_ALIGN);
        let hyperlink_map_end = hyperlink_map_start + hyperlink_map_layout.total_size;

        let total_size = align_forward(hyperlink_map_end, PAGE_SIZE_MIN);

        PageLayout {
            total_size,
            rows_start,
            rows_size: rows_end - rows_start,
            cells_start,
            cells_size: cells_end - cells_start,
            styles_start,
            styles_layout,
            grapheme_alloc_start,
            grapheme_alloc_layout,
            grapheme_map_start,
            grapheme_map_layout,
            string_alloc_start,
            string_alloc_layout,
            hyperlink_set_start,
            hyperlink_set_layout,
            hyperlink_map_start,
            hyperlink_map_layout,
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

        let styles = if l.styles_layout.cap > 0 {
            StyleSet::init(buf.add(l.styles_start), l.styles_layout)
        } else {
            StyleSet::new()
        };

        let grapheme_alloc = if l.grapheme_alloc_layout.total_size > 0 {
            GraphemeAlloc::init(buf.add(l.grapheme_alloc_start), l.grapheme_alloc_layout)
        } else {
            zero_bitmap_allocator::<GRAPHEME_CHUNK>()
        };

        let grapheme_map = if l.grapheme_map_layout.capacity > 0 {
            GraphemeMap::init(buf.add(l.grapheme_map_start), l.grapheme_map_layout)
        } else {
            GraphemeMap::new()
        };

        let string_alloc = if l.string_alloc_layout.total_size > 0 {
            StringAlloc::init(buf.add(l.string_alloc_start), l.string_alloc_layout)
        } else {
            zero_bitmap_allocator::<STRING_CHUNK>()
        };

        let hyperlink_set = if l.hyperlink_set_layout.cap > 0 {
            HyperlinkSet::init(buf.add(l.hyperlink_set_start), l.hyperlink_set_layout)
        } else {
            HyperlinkSet::new()
        };

        let hyperlink_map = if l.hyperlink_map_layout.capacity > 0 {
            HyperlinkMap::init(buf.add(l.hyperlink_map_start), l.hyperlink_map_layout)
        } else {
            HyperlinkMap::new()
        };

        Page {
            memory: base,
            memory_len: l.total_size,
            rows,
            cells,
            dirty: false,
            string_alloc,
            grapheme_alloc,
            grapheme_map,
            styles,
            hyperlink_map,
            hyperlink_set,
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
        styles: 128,
        hyperlink_bytes: (HYPERLINK_COUNT_DEFAULT as usize
            * core::mem::size_of::<HyperlinkPageEntry>()) as u16,
        grapheme_bytes: 512,
        string_bytes: (StringAlloc::BITMAP_BIT_SIZE * STRING_CHUNK) as u32,
    }
}
