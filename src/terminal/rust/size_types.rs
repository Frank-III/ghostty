use crate::constants::*;
use crate::early::*;

pub const MAX_PAGE_SIZE: u32 = u32::MAX;

pub type OffsetInt = u32;
pub type CellCountInt = u16;
pub type StyleCountInt = CellCountInt;
pub type HyperlinkCountInt = CellCountInt;
pub type GraphemeBytesInt = u32;
pub type StringBytesInt = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Offset {
    pub(crate) offset: OffsetInt,
}

impl Default for Offset {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

impl Offset {
    pub unsafe fn ptr<T>(&self, base: *const u8) -> *const T {
        let addr = base as usize + self.offset as usize;
        debug_assert!(addr % core::mem::align_of::<T>() == 0);
        addr as *const T
    }

    pub unsafe fn ptr_mut<T>(&self, base: *mut u8) -> *mut T {
        let addr = base as usize + self.offset as usize;
        debug_assert!(addr % core::mem::align_of::<T>() == 0);
        addr as *mut T
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OffsetSlice {
    pub(crate) offset: Offset,
    pub(crate) len: usize,
}

impl Default for OffsetSlice {
    fn default() -> Self {
        Self {
            offset: Offset::default(),
            len: 0,
        }
    }
}

impl OffsetSlice {
    pub unsafe fn slice<'a, T>(&self, base: *const u8) -> &'a [T] {
        unsafe {
            let ptr: *const T = self.offset.ptr(base);
            core::slice::from_raw_parts(ptr, self.len)
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OffsetBuf {
    pub(crate) base: *mut u8,
    pub(crate) offset: usize,
}

impl OffsetBuf {
    pub fn init(base: *mut u8) -> Self {
        Self { base, offset: 0 }
    }

    pub fn init_offset(base: *mut u8, offset: usize) -> Self {
        Self { base, offset }
    }

    pub fn start(&self) -> *mut u8 {
        unsafe { self.base.add(self.offset) }
    }

    pub fn member(&self, len: usize) -> Offset {
        Offset {
            offset: (self.offset + len) as OffsetInt,
        }
    }

    pub fn add(&self, offset: usize) -> Self {
        Self {
            base: self.base,
            offset: self.offset + offset,
        }
    }

    pub fn rebase(&self, offset: usize) -> Self {
        Self {
            base: unsafe { self.start().add(offset) },
            offset: 0,
        }
    }
}

pub unsafe fn get_offset(base: *const u8, ptr: *const u8) -> Offset {
    let base_int = base as usize;
    let ptr_int = ptr as usize;
    Offset {
        offset: (ptr_int - base_int) as OffsetInt,
    }
}

fn int_from_base(base: *const u8) -> usize {
    base as usize
}
