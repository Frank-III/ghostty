use crate::constants::*;
use crate::early::*;
use crate::size_types::*;
use core::ffi::c_void;

pub type HyperlinkId = HyperlinkCountInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyperlinkEntryId<'a> {
    Explicit(&'a [u8]),
    Implicit(OffsetInt),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HyperlinkPageEntryId {
    pub tag: u8,
    pub explicit: OffsetSlice,
    pub implicit: OffsetInt,
}

impl HyperlinkPageEntryId {
    pub const TAG_EXPLICIT: u8 = 0;
    pub const TAG_IMPLICIT: u8 = 1;

    pub fn implicit(v: OffsetInt) -> Self {
        Self {
            tag: Self::TAG_IMPLICIT,
            explicit: OffsetSlice::default(),
            implicit: v,
        }
    }

    pub fn explicit(offset_slice: OffsetSlice) -> Self {
        Self {
            tag: Self::TAG_EXPLICIT,
            explicit: offset_slice,
            implicit: 0,
        }
    }

    pub fn is_explicit(&self) -> bool {
        self.tag == Self::TAG_EXPLICIT
    }

    pub unsafe fn explicit_slice<'a>(&self, base: *const u8) -> &'a [u8] {
        unsafe { self.explicit.slice(base) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HyperlinkPageEntry {
    pub id: HyperlinkPageEntryId,
    pub uri: OffsetSlice,
}

impl HyperlinkPageEntry {
    pub unsafe fn uri_slice<'a>(&self, base: *const u8) -> &'a [u8] {
        unsafe { self.uri.slice(base) }
    }

    pub unsafe fn hash(&self, base: *const u8) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.id.tag as u64;
        h = h.wrapping_mul(0x100000001b3);

        match self.id.tag {
            HyperlinkPageEntryId::TAG_IMPLICIT => {
                let bytes = self.id.implicit.to_le_bytes();
                for &b in &bytes {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
            }
            HyperlinkPageEntryId::TAG_EXPLICIT => {
                let slice = unsafe { self.id.explicit_slice(base) };
                for &b in slice {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
            }
            _ => {}
        }

        let uri = unsafe { self.uri_slice(base) };
        for &b in uri {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub unsafe fn eql(
        &self,
        self_base: *const u8,
        other: &HyperlinkPageEntry,
        other_base: *const u8,
    ) -> bool {
        if self.id.tag != other.id.tag {
            return false;
        }
        match self.id.tag {
            HyperlinkPageEntryId::TAG_IMPLICIT => {
                if self.id.implicit != other.id.implicit {
                    return false;
                }
            }
            HyperlinkPageEntryId::TAG_EXPLICIT => {
                let a = unsafe { self.id.explicit_slice(self_base) };
                let b = unsafe { other.id.explicit_slice(other_base) };
                if a != b {
                    return false;
                }
            }
            _ => return false,
        }
        let a_uri = unsafe { self.uri_slice(self_base) };
        let b_uri = unsafe { other.uri_slice(other_base) };
        a_uri == b_uri
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Hyperlink {
    pub id: HyperlinkId,
    pub uri_ptr: *const u8,
    pub uri_len: usize,
}

impl Hyperlink {
    pub fn uri(&self) -> &[u8] {
        if self.uri_ptr.is_null() || self.uri_len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.uri_ptr, self.uri_len) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HyperlinkSetContext {
    pub page: *mut c_void,
    pub src_page: *const c_void,
}

impl Default for HyperlinkSetContext {
    fn default() -> Self {
        Self {
            page: core::ptr::null_mut(),
            src_page: core::ptr::null(),
        }
    }
}

pub unsafe fn hyperlink_page_entry_hash(entry: *const HyperlinkPageEntry, base: *const u8) -> u64 {
    unsafe { (*entry).hash(base) }
}

pub unsafe fn hyperlink_page_entry_eq(
    a: *const HyperlinkPageEntry,
    a_base: *const u8,
    b: *const HyperlinkPageEntry,
    b_base: *const u8,
) -> bool {
    unsafe { (*a).eql(a_base, &*b, b_base) }
}

pub const HYPERLINK_DEFAULT_ID: HyperlinkId = 0;
