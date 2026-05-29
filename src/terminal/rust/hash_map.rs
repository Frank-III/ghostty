use crate::constants::*;
use crate::early::*;
use crate::size_types::*;

use core::marker::PhantomData;
use core::mem;
use core::ptr;

const fn align_forward(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

const fn max3(a: usize, b: usize, c: usize) -> usize {
    if a >= b && a >= c {
        a
    } else if b >= c {
        b
    } else {
        c
    }
}

pub type Size = u32;
pub type Hash = u64;

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Metadata(u8);

impl Metadata {
    const FREE: u8 = 0x00;
    const TOMBSTONE: u8 = 0x01;

    #[inline]
    pub fn is_used(self) -> bool {
        (self.0 & 0x80) != 0
    }

    #[inline]
    pub fn is_tombstone(self) -> bool {
        self.0 == Self::TOMBSTONE
    }

    #[inline]
    pub fn is_free(self) -> bool {
        self.0 == Self::FREE
    }

    #[inline]
    pub fn fingerprint(self) -> u8 {
        self.0 & 0x7F
    }

    #[inline]
    pub fn take_fingerprint(hash: Hash) -> u8 {
        ((hash >> 57) & 0x7F) as u8
    }

    #[inline]
    pub fn fill(slot: &mut Self, fp: u8) {
        *slot = Self(0x80 | (fp & 0x7F));
    }

    #[inline]
    pub fn remove_slot(slot: &mut Self) {
        *slot = Self(Self::TOMBSTONE);
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct Header {
    pub values: Offset,
    pub keys: Offset,
    pub capacity: Size,
    pub size: Size,
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub total_size: usize,
    pub keys_start: usize,
    pub vals_start: usize,
    pub capacity: Size,
}

// ---------------------------------------------------------------------------
// KV / GetOrPutResult
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct KV<K, V> {
    pub key: K,
    pub value: V,
}

pub struct GetOrPutResult<K, V> {
    pub key_ptr: *mut K,
    pub value_ptr: *mut V,
    pub found_existing: bool,
}

// ---------------------------------------------------------------------------
// HashMapContext trait
// ---------------------------------------------------------------------------

pub trait HashMapContext<K> {
    fn hash(key: &K) -> Hash;
    fn eql(a: &K, b: &K) -> bool;
}

// ---------------------------------------------------------------------------
// AutoContext (FNV-1a)
// ---------------------------------------------------------------------------

pub struct AutoContext;

impl<K: Copy> HashMapContext<K> for AutoContext {
    fn hash(key: &K) -> Hash {
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;
        let bytes: &[u8] = unsafe {
            core::slice::from_raw_parts((key as *const K) as *const u8, mem::size_of::<K>())
        };
        let mut h: u64 = FNV_OFFSET;
        let mut i: usize = 0;
        while i < bytes.len() {
            h ^= bytes[i] as u64;
            h = h.wrapping_mul(FNV_PRIME);
            i += 1;
        }
        h
    }

    fn eql(a: &K, b: &K) -> bool {
        let a_bytes: &[u8] = unsafe {
            core::slice::from_raw_parts((a as *const K) as *const u8, mem::size_of::<K>())
        };
        let b_bytes: &[u8] = unsafe {
            core::slice::from_raw_parts((b as *const K) as *const u8, mem::size_of::<K>())
        };
        a_bytes == b_bytes
    }
}

// ---------------------------------------------------------------------------
// HashMapUnmanaged
// ---------------------------------------------------------------------------

pub struct HashMapUnmanaged<K, V, C> {
    metadata: *mut Metadata,
    _phantom: PhantomData<(K, V, C)>,
}

impl<K, V, C> Copy for HashMapUnmanaged<K, V, C> {}

impl<K, V, C> Clone for HashMapUnmanaged<K, V, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V, C: HashMapContext<K>> HashMapUnmanaged<K, V, C> {
    const HEADER_ALIGN: usize = mem::align_of::<Header>();
    const KEY_ALIGN: usize = if mem::size_of::<K>() == 0 {
        1
    } else {
        mem::align_of::<K>()
    };
    const VAL_ALIGN: usize = if mem::size_of::<V>() == 0 {
        1
    } else {
        mem::align_of::<V>()
    };
    pub const BASE_ALIGN: usize = max3(Self::HEADER_ALIGN, Self::KEY_ALIGN, Self::VAL_ALIGN);

    // -- Layout ---------------------------------------------------------------

    pub fn layout_for_capacity(new_capacity: Size) -> Layout {
        debug_assert!(new_capacity == 0 || new_capacity.is_power_of_two());
        let cap = new_capacity as usize;
        let meta_start = mem::size_of::<Header>();
        let meta_end = meta_start + cap;
        let keys_start = align_forward(meta_end, Self::KEY_ALIGN);
        let keys_end = keys_start + cap * mem::size_of::<K>();
        let vals_start = align_forward(keys_end, Self::VAL_ALIGN);
        let vals_end = vals_start + cap * mem::size_of::<V>();
        let total_size = align_forward(vals_end, Self::BASE_ALIGN);
        Layout {
            total_size,
            keys_start: keys_start - meta_start,
            vals_start: vals_start - meta_start,
            capacity: new_capacity,
        }
    }

    // -- Init -----------------------------------------------------------------

    pub fn init(buf: OffsetBuf, layout: Layout) -> Self {
        let metadata_buf = buf.rebase(mem::size_of::<Header>());
        let metadata_ptr = metadata_buf.start() as *mut Metadata;

        let hdr = unsafe { (metadata_ptr as *mut Header).sub(1) };
        unsafe {
            (*hdr).capacity = layout.capacity;
            (*hdr).size = 0;
            (*hdr).keys = metadata_buf.member(layout.keys_start);
            (*hdr).values = metadata_buf.member(layout.vals_start);
        }

        unsafe {
            ptr::write_bytes(metadata_ptr, 0, layout.capacity as usize);
        }

        Self {
            metadata: metadata_ptr,
            _phantom: PhantomData,
        }
    }

    // -- Accessors ------------------------------------------------------------

    #[inline]
    fn header_ptr(&self) -> *const Header {
        unsafe { (self.metadata as *const Header).sub(1) }
    }

    #[inline]
    fn header_mut_ptr(&mut self) -> *mut Header {
        unsafe { (self.metadata as *mut Header).sub(1) }
    }

    #[inline]
    pub fn header(&self) -> &Header {
        unsafe { &*self.header_ptr() }
    }

    #[inline]
    pub fn header_mut(&mut self) -> &mut Header {
        unsafe { &mut *self.header_mut_ptr() }
    }

    #[inline]
    pub fn keys(&self) -> *const K {
        unsafe { (*self.header_ptr()).keys.ptr(self.metadata as *const u8) }
    }

    #[inline]
    pub fn keys_mut(&mut self) -> *mut K {
        unsafe { (*self.header_ptr()).keys.ptr_mut(self.metadata as *mut u8) }
    }

    #[inline]
    pub fn values(&self) -> *const V {
        unsafe { (*self.header_ptr()).values.ptr(self.metadata as *const u8) }
    }

    #[inline]
    pub fn values_mut(&mut self) -> *mut V {
        unsafe {
            (*self.header_ptr())
                .values
                .ptr_mut(self.metadata as *mut u8)
        }
    }

    #[inline]
    pub fn capacity(&self) -> Size {
        unsafe { (*self.header_ptr()).capacity }
    }

    #[inline]
    pub fn count(&self) -> Size {
        unsafe { (*self.header_ptr()).size }
    }

    fn init_metadatas(&mut self) {
        unsafe {
            ptr::write_bytes(self.metadata, 0, self.capacity() as usize);
        }
    }

    // -- Lookup ---------------------------------------------------------------

    #[inline]
    pub fn get_index(&self, key: &K, _ctx: C) -> Option<usize> {
        if self.count() == 0 {
            return None;
        }
        let hash = C::hash(key);
        let cap = self.capacity() as usize;
        let mask = cap - 1;
        let fingerprint = Metadata::take_fingerprint(hash);
        let mut limit = cap;
        let mut idx = (hash as usize) & mask;

        loop {
            let slot = unsafe { *self.metadata.add(idx) };
            if slot.is_free() || limit == 0 {
                break;
            }
            if slot.is_used() && slot.fingerprint() == fingerprint {
                let test_key = unsafe { &*self.keys().add(idx) };
                if C::eql(key, test_key) {
                    return Some(idx);
                }
            }
            limit -= 1;
            idx = (idx + 1) & mask;
        }
        None
    }

    pub fn get(&self, key: &K, ctx: C) -> Option<&V> {
        let idx = self.get_index(key, ctx)?;
        Some(unsafe { &*self.values().add(idx) })
    }

    pub fn get_mut(&mut self, key: &K, ctx: C) -> Option<&mut V> {
        let idx = self.get_index(key, ctx)?;
        Some(unsafe { &mut *self.values_mut().add(idx) })
    }

    pub fn contains(&self, key: &K, ctx: C) -> bool {
        self.get_index(key, ctx).is_some()
    }

    // -- Insert ---------------------------------------------------------------

    fn get_or_put_assume_capacity_adapted(&mut self, key: &K, _ctx: C) -> GetOrPutResult<K, V> {
        let hash = C::hash(key);
        let cap = self.capacity() as usize;
        let mask = cap - 1;
        let fingerprint = Metadata::take_fingerprint(hash);
        let mut limit = cap;
        let mut idx = (hash as usize) & mask;
        let mut first_tombstone_idx: usize = cap;

        loop {
            let slot = unsafe { *self.metadata.add(idx) };
            if slot.is_free() || limit == 0 {
                break;
            }
            if slot.is_used() && slot.fingerprint() == fingerprint {
                let test_key = unsafe { &*self.keys().add(idx) };
                if C::eql(key, test_key) {
                    let kp = unsafe { self.keys_mut().add(idx) };
                    let vp = unsafe { self.values_mut().add(idx) };
                    return GetOrPutResult {
                        key_ptr: kp,
                        value_ptr: vp,
                        found_existing: true,
                    };
                }
            } else if first_tombstone_idx == cap && slot.is_tombstone() {
                first_tombstone_idx = idx;
            }
            limit -= 1;
            idx = (idx + 1) & mask;
        }

        if first_tombstone_idx < cap {
            idx = first_tombstone_idx;
        }

        unsafe {
            Metadata::fill(&mut *self.metadata.add(idx), fingerprint);
            (*self.header_mut_ptr()).size += 1;
        }

        GetOrPutResult {
            key_ptr: unsafe { self.keys_mut().add(idx) },
            value_ptr: unsafe { self.values_mut().add(idx) },
            found_existing: false,
        }
    }

    pub fn get_or_put_assume_capacity(&mut self, key: K, ctx: C) -> GetOrPutResult<K, V> {
        let result = self.get_or_put_assume_capacity_adapted(&key, ctx);
        if !result.found_existing {
            unsafe {
                ptr::write(result.key_ptr, key);
            }
        }
        result
    }

    pub fn put_assume_capacity(&mut self, key: K, value: V, ctx: C) {
        let gop = self.get_or_put_assume_capacity(key, ctx);
        unsafe {
            ptr::write(gop.value_ptr, value);
        }
    }

    pub fn put_assume_capacity_no_clobber(&mut self, key: K, value: V, ctx: C) {
        debug_assert!(!self.contains(&key, ctx));
        let hash = C::hash(&key);
        let cap = self.capacity() as usize;
        let mask = cap - 1;
        let mut idx = (hash as usize) & mask;

        loop {
            let slot = unsafe { *self.metadata.add(idx) };
            if !slot.is_used() {
                break;
            }
            idx = (idx + 1) & mask;
        }

        let fingerprint = Metadata::take_fingerprint(hash);
        unsafe {
            Metadata::fill(&mut *self.metadata.add(idx), fingerprint);
            ptr::write(self.keys_mut().add(idx), key);
            ptr::write(self.values_mut().add(idx), value);
            (*self.header_mut_ptr()).size += 1;
        }
    }

    pub fn fetch_put_assume_capacity(&mut self, key: K, value: V, ctx: C) -> Option<KV<K, V>> {
        let gop = self.get_or_put_assume_capacity(key, ctx);
        let result = if gop.found_existing {
            Some(KV {
                key: unsafe { ptr::read(gop.key_ptr) },
                value: unsafe { ptr::read(gop.value_ptr) },
            })
        } else {
            None
        };
        unsafe {
            ptr::write(gop.value_ptr, value);
        }
        result
    }

    // -- Remove ---------------------------------------------------------------

    fn remove_by_index(&mut self, idx: usize) {
        unsafe {
            Metadata::remove_slot(&mut *self.metadata.add(idx));
            (*self.header_mut_ptr()).size -= 1;
        }
    }

    pub fn remove(&mut self, key: &K, ctx: C) -> bool {
        if let Some(idx) = self.get_index(key, ctx) {
            self.remove_by_index(idx);
            true
        } else {
            false
        }
    }

    pub fn fetch_remove(&mut self, key: &K, ctx: C) -> Option<KV<K, V>> {
        let idx = self.get_index(key, ctx)?;
        unsafe {
            let old_key = ptr::read(self.keys().add(idx));
            let old_val = ptr::read(self.values().add(idx));
            Metadata::remove_slot(&mut *self.metadata.add(idx));
            (*self.header_mut_ptr()).size -= 1;
            Some(KV {
                key: old_key,
                value: old_val,
            })
        }
    }

    // -- Misc -----------------------------------------------------------------

    pub fn clear_retaining_capacity(&mut self) {
        self.init_metadatas();
        self.header_mut().size = 0;
    }

    pub fn iterator(&self) -> HashMapIterator<'_, K, V, C> {
        HashMapIterator {
            map: self,
            index: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// HashMapIterator
// ---------------------------------------------------------------------------

pub struct HashMapIterator<'a, K, V, C> {
    map: &'a HashMapUnmanaged<K, V, C>,
    index: usize,
}

impl<'a, K, V, C: HashMapContext<K>> Iterator for HashMapIterator<'a, K, V, C> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let cap = self.map.capacity() as usize;
        while self.index < cap {
            let slot = unsafe { *self.map.metadata.add(self.index) };
            if slot.is_used() {
                let key = unsafe { &*self.map.keys().add(self.index) };
                let val = unsafe { &*self.map.values().add(self.index) };
                self.index += 1;
                return Some((key, val));
            }
            self.index += 1;
        }
        None
    }
}

// ---------------------------------------------------------------------------
// OffsetHashMap
// ---------------------------------------------------------------------------

pub struct OffsetHashMap<K, V, C> {
    pub metadata: Offset,
    _phantom: PhantomData<(K, V, C)>,
}

impl<K, V, C> Copy for OffsetHashMap<K, V, C> {}

impl<K, V, C> Clone for OffsetHashMap<K, V, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V, C: HashMapContext<K>> OffsetHashMap<K, V, C> {
    pub const BASE_ALIGN: usize = HashMapUnmanaged::<K, V, C>::BASE_ALIGN;

    pub fn new() -> Self {
        Self {
            metadata: Offset { offset: 0 },
            _phantom: PhantomData,
        }
    }

    pub fn layout(cap: Size) -> Layout {
        HashMapUnmanaged::<K, V, C>::layout_for_capacity(cap)
    }

    pub fn init(buf: OffsetBuf, layout: Layout) -> Self {
        let m = HashMapUnmanaged::<K, V, C>::init(buf, layout);
        let offset = unsafe { get_offset(buf.base as *const u8, m.metadata as *const u8) };
        Self {
            metadata: offset,
            _phantom: PhantomData,
        }
    }

    pub fn map(&self, base: *mut u8) -> HashMapUnmanaged<K, V, C> {
        unsafe {
            let meta_ptr: *mut Metadata = self.metadata.ptr_mut(base);
            HashMapUnmanaged {
                metadata: meta_ptr,
                _phantom: PhantomData,
            }
        }
    }
}
