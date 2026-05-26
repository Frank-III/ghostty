use crate::early::*;
use crate::constants::*;
use crate::size_types::*;

use core::mem;
use core::ptr;

pub type RefCount = u32;
pub const DEFAULT_ID: u16 = 0;

#[inline(always)]
fn align_forward(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[derive(Debug, Clone, Copy)]
pub struct RefCountedSetLayout {
    pub cap: u16,
    pub items_start: usize,
    pub ref_counts_start: usize,
    pub total_size: usize,
}

impl RefCountedSetLayout {
    pub fn init(cap: u16, item_size: usize, item_align: usize) -> Self {
        if cap == 0 {
            return Self {
                cap: 0,
                items_start: 0,
                ref_counts_start: 0,
                total_size: 0,
            };
        }
        let n = cap as usize;
        let items_start = 0;
        let items_end = items_start + n * item_size;
        let rc_align = if mem::align_of::<RefCount>() > item_align {
            mem::align_of::<RefCount>()
        } else {
            item_align
        };
        let ref_counts_start = align_forward(items_end, rc_align);
        let ref_counts_end = ref_counts_start + n * mem::size_of::<RefCount>();
        Self {
            cap,
            items_start,
            ref_counts_start,
            total_size: ref_counts_end,
        }
    }
}

#[repr(C)]
pub struct RefCountedSet {
    items_offset: Offset,
    ref_counts_offset: Offset,
    len: u16,
    capacity: u16,
}

impl RefCountedSet {
    pub fn init(buf: OffsetBuf, layout: RefCountedSetLayout) -> Self {
        let items_off = buf.member(layout.items_start);
        let rc_off = buf.member(layout.ref_counts_start);
        unsafe {
            let base: *mut u8 = buf.base;
            if layout.cap > 0 {
                let items_bytes = layout.ref_counts_start - layout.items_start;
                ptr::write_bytes(items_off.ptr_mut::<u8>(base), 0, items_bytes);
                ptr::write_bytes(rc_off.ptr_mut::<u8>(base), 0, layout.cap as usize * mem::size_of::<RefCount>());
            }
        }
        Self {
            items_offset: items_off,
            ref_counts_offset: rc_off,
            len: 0,
            capacity: layout.cap,
        }
    }

    #[inline]
    pub fn capacity(&self) -> u16 {
        self.capacity
    }

    #[inline]
    pub fn count(&self) -> u16 {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    unsafe fn item_ptr<T>(&self, base: *const u8, id: u16) -> *const T {
        unsafe {
            let p: *const T = self.items_offset.ptr(base);
            p.add(id as usize)
        }
    }

    #[inline]
    unsafe fn item_ptr_mut<T>(&self, base: *mut u8, id: u16) -> *mut T {
        unsafe {
            let p: *mut T = self.items_offset.ptr_mut(base);
            p.add(id as usize)
        }
    }

    #[inline]
    unsafe fn rc_ptr(&self, base: *const u8, id: u16) -> *const RefCount {
        unsafe {
            let p: *const RefCount = self.ref_counts_offset.ptr(base);
            p.add(id as usize)
        }
    }

    #[inline]
    unsafe fn rc_ptr_mut(&self, base: *mut u8, id: u16) -> *mut RefCount {
        unsafe {
            let p: *mut RefCount = self.ref_counts_offset.ptr_mut(base);
            p.add(id as usize)
        }
    }

    pub unsafe fn get<T: Copy>(&self, base: *const u8, id: u16) -> T {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.item_ptr::<T>(base, id);
            p.read()
        }
    }

    pub unsafe fn get_ref<'a, T>(&self, base: *const u8, id: u16) -> &'a T {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.item_ptr::<T>(base, id);
            &*p
        }
    }

    pub unsafe fn get_mut<'a, T>(&self, base: *mut u8, id: u16) -> &'a mut T {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.item_ptr_mut::<T>(base, id);
            &mut *p
        }
    }

    pub unsafe fn ref_count(&self, base: *const u8, id: u16) -> RefCount {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe { *self.rc_ptr(base, id) }
    }

    pub unsafe fn add_id(&mut self, base: *mut u8, id: u16) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.rc_ptr_mut(base, id);
            debug_assert!(*p > 0);
            *p += 1;
        }
    }

    pub unsafe fn use_multiple(&mut self, base: *mut u8, id: u16, n: RefCount) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.rc_ptr_mut(base, id);
            debug_assert!(*p > 0);
            *p += n;
        }
    }

    pub unsafe fn remove(&mut self, base: *mut u8, id: u16) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.rc_ptr_mut(base, id);
            debug_assert!(*p > 0);
            *p -= 1;
            if *p == 0 {
                if id == self.len {
                    while self.len > 0 {
                        let prev = self.rc_ptr_mut(base, self.len);
                        if *prev != 0 {
                            break;
                        }
                        self.len -= 1;
                    }
                }
            }
        }
    }

    pub unsafe fn release_multiple(&mut self, base: *mut u8, id: u16, n: RefCount) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!(id <= self.capacity);
        unsafe {
            let p = self.rc_ptr_mut(base, id);
            debug_assert!(*p >= n);
            *p -= n;
            if *p == 0 {
                if id == self.len {
                    while self.len > 0 {
                        let prev = self.rc_ptr_mut(base, self.len);
                        if *prev != 0 {
                            break;
                        }
                        self.len -= 1;
                    }
                }
            }
        }
    }

    pub unsafe fn next_id<T: Copy>(&mut self, base: *mut u8, value: T) -> Option<u16> {
        let id = self.len + 1;
        if id > self.capacity {
            return None;
        }
        unsafe {
            let ip = self.item_ptr_mut::<T>(base, id);
            ptr::write(ip, value);
            let rp = self.rc_ptr_mut(base, id);
            *rp = 1;
        }
        self.len = id;
        Some(id)
    }
}
