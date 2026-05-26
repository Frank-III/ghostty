use crate::early::*;
use crate::size_types::*;

use core::cmp;
use core::mem;

const fn align_forward(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub total_size: usize,
    pub bitmap_count: usize,
    pub bitmap_start: usize,
    pub chunks_start: usize,
}

pub fn layout<const CHUNK_SIZE: usize>(cap: usize) -> Layout {
    debug_assert!(CHUNK_SIZE.is_power_of_two());

    let aligned_cap = align_forward(cap, CHUNK_SIZE);

    let chunk_count = aligned_cap / CHUNK_SIZE;
    let aligned_chunk_count = align_forward(chunk_count, 64);
    let bitmap_count = aligned_chunk_count / 64;

    let bitmap_start = 0;
    let bitmap_end = mem::size_of::<u64>() * bitmap_count;
    let chunks_start = align_forward(bitmap_end, mem::align_of::<u8>());
    let chunks_end = chunks_start + (aligned_cap * CHUNK_SIZE);
    let total_size = chunks_end;

    Layout {
        total_size,
        bitmap_count,
        bitmap_start,
        chunks_start,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BitmapAllocator<const CHUNK_SIZE: usize> {
    pub bitmap: Offset,
    pub bitmap_count: usize,
    pub chunks: Offset,
}

impl<const CHUNK_SIZE: usize> BitmapAllocator<CHUNK_SIZE> {
    pub const BASE_ALIGN: usize = mem::align_of::<u64>();
    pub const BITMAP_BIT_SIZE: usize = 64;

    pub fn init(buf: OffsetBuf, l: Layout) -> Self {
        debug_assert!(CHUNK_SIZE.is_power_of_two());
        debug_assert!((buf.start() as usize) % Self::BASE_ALIGN == 0);

        let bitmap = buf.member(l.bitmap_start);
        unsafe {
            let bitmap_ptr: *mut u64 = bitmap.ptr_mut(buf.base);
            let mut i: usize = 0;
            while i < l.bitmap_count {
                bitmap_ptr.add(i).write(u64::MAX);
                i += 1;
            }
        }

        Self {
            bitmap,
            bitmap_count: l.bitmap_count,
            chunks: buf.member(l.chunks_start),
        }
    }

    pub fn bytes_required<T>(n: usize) -> usize {
        let byte_count = mem::size_of::<T>() * n;
        align_forward(byte_count, CHUNK_SIZE)
    }

    pub unsafe fn alloc<'a, T>(
        &mut self,
        base: *mut u8,
        n: usize,
    ) -> Option<&'a mut [T]> {
        debug_assert!(CHUNK_SIZE % mem::align_of::<T>() == 0);
        debug_assert!(n > 0);

        let byte_count = mem::size_of::<T>().checked_mul(n)?;
        let chunk_count = div_ceil(byte_count, CHUNK_SIZE);

        unsafe {
            let bitmaps: *mut u64 = self.bitmap.ptr_mut(base);
            let bitmaps_slice = core::slice::from_raw_parts_mut(bitmaps, self.bitmap_count);
            let idx = find_free_chunks(bitmaps_slice, chunk_count)?;

            let chunks: *mut u8 = self.chunks.ptr_mut(base);
            let ptr: *mut T = chunks.add(idx * CHUNK_SIZE).cast();
            Some(core::slice::from_raw_parts_mut(ptr, n))
        }
    }

    pub unsafe fn free<T>(
        &mut self,
        base: *mut u8,
        slice: &[T],
    ) {
        let bytes_len = slice.len() * mem::size_of::<T>();
        let aligned_len = align_forward(bytes_len, CHUNK_SIZE);
        let chunk_count = aligned_len / CHUNK_SIZE;

        unsafe {
            let chunks: *mut u8 = self.chunks.ptr_mut(base);
            let chunk_idx =
                ((slice.as_ptr() as *const u8 as usize) - (chunks as usize)) / CHUNK_SIZE;

            let bitmaps: *mut u64 = self.bitmap.ptr_mut(base);

            let mut i: usize = chunk_idx / 64;
            let mut rem: usize = chunk_count;

            {
                let bit = chunk_idx % 64;
                let bits = cmp::min(rem, 64 - bit);

                let mask: u64 = (!0u64) >> ((64 - bits) as u32) << (bit as u32);
                let bm = bitmaps.add(i);
                bm.write(bm.read() | mask);
                rem -= bits;
            }

            i += 1;
            while rem > 64 {
                bitmaps.add(i).write(u64::MAX);
                rem -= 64;
                i += 1;
            }

            if rem > 0 {
                let mask: u64 = (!0u64) >> ((64 - rem) as u32);
                let bm = bitmaps.add(i);
                bm.write(bm.read() | mask);
            }
        }
    }

    pub fn capacity_bytes(&self) -> usize {
        self.bitmap_count * Self::BITMAP_BIT_SIZE * CHUNK_SIZE
    }

    pub unsafe fn used_bytes(&self, base: *const u8) -> usize {
        unsafe {
            let bitmaps: *const u64 = self.bitmap.ptr(base);
            let mut free_chunks: usize = 0;
            let mut i: usize = 0;
            while i < self.bitmap_count {
                free_chunks += bitmaps.add(i).read().count_ones() as usize;
                i += 1;
            }
            let total_chunks = self.bitmap_count * Self::BITMAP_BIT_SIZE;
            (total_chunks - free_chunks) * CHUNK_SIZE
        }
    }

    pub fn compute_layout(cap: usize) -> Layout {
        layout::<CHUNK_SIZE>(cap)
    }
}

fn find_free_chunks(bitmaps: &mut [u64], n: usize) -> Option<usize> {
    if n > 64 {
        let mut i: usize = 0;
        'search: while i < bitmaps.len() {
            let prefix = (!bitmaps[i]).leading_zeros() as usize;

            if prefix == 0 {
                i += 1;
                continue;
            }

            let start_bitmap = i;
            let start_bit = 64 - prefix;

            let mut rem: usize = n - prefix;

            i += 1;
            while rem > 64 {
                if i >= bitmaps.len() {
                    return None;
                }
                if bitmaps[i] != u64::MAX {
                    continue 'search;
                }
                rem -= 64;
                i += 1;
            }

            if (!bitmaps[i]).trailing_zeros() as usize >= rem {
                let suffix = (n - prefix) % 64;

                bitmaps[start_bitmap] ^=
                    (!0u64) >> (start_bit as u32) << (start_bit as u32);
                let full_bitmaps = (n - prefix - suffix) / 64;
                let mut j: usize = 0;
                while j < full_bitmaps {
                    bitmaps[start_bitmap + 1 + j] = 0;
                    j += 1;
                }
                if suffix > 0 {
                    bitmaps[i] ^= (!0u64) >> ((64 - suffix) as u32);
                }

                return Some(start_bitmap * 64 + start_bit);
            }
        }

        return None;
    }

    debug_assert!(n <= 64);
    for (idx, bitmap) in bitmaps.iter_mut().enumerate() {
        let mut shifted: u64 = *bitmap;
        let mut i: usize = 1;
        while i < n {
            shifted &= *bitmap >> (i as u32);
            i += 1;
        }

        if shifted == 0 {
            continue;
        }

        let bit = shifted.trailing_zeros() as usize;
        let mask: u64 = (u64::MAX >> ((64 - n) as u32)) << (bit as u32);
        *bitmap ^= mask;

        return Some((idx * 64) + bit);
    }

    None
}
