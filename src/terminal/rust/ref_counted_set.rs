use crate::size_types::*;

use core::mem;
use core::ptr;

pub type Id = u16;
pub type RefCountInt = u16;
pub type RefCount = RefCountInt;
pub const DEFAULT_ID: Id = 0;

const PSL_STATS_LEN: usize = 32;
const LOAD_FACTOR_NUM: usize = 13;
const LOAD_FACTOR_DEN: usize = 16;
const REHASH_THRESHOLD_NUM: usize = 9;
const REHASH_THRESHOLD_DEN: usize = 10;

#[inline(always)]
fn align_forward(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AddError {
    OutOfMemory,
    NeedsRehash,
}

pub trait RefCountedSetContext<T> {
    fn hash(value: &T) -> u64;
    fn eql(a: &T, b: &T) -> bool;
    fn deleted(_set: &RefCountedSet, _value: &T) {}
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ItemMeta {
    bucket: Id,
    psl: Id,
    ref_count: RefCountInt,
}

impl ItemMeta {
    #[inline(always)]
    fn empty() -> Self {
        Self {
            bucket: Id::MAX,
            psl: 0,
            ref_count: 0,
        }
    }

    #[inline(always)]
    fn is_in_table(&self, table_cap: usize) -> bool {
        (self.bucket as usize) < table_cap
    }
}

pub fn capacity_for_count(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let needed = n + 1;
    (needed * LOAD_FACTOR_DEN + LOAD_FACTOR_NUM - 1) / LOAD_FACTOR_NUM
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RefCountedSetLayout {
    pub cap: usize,
    pub table_cap: usize,
    pub table_mask: Id,
    pub total_size: usize,
    value_size: usize,
    item_stride: usize,
    items_start: usize,
}

impl RefCountedSetLayout {
    pub fn init(cap: u16, value_size: usize, value_align: usize) -> Self {
        Self::init_usize(cap as usize, value_size, value_align)
    }

    pub fn init_usize(cap: usize, value_size: usize, value_align: usize) -> Self {
        if cap == 0 {
            return Self {
                cap: 0,
                table_cap: 0,
                table_mask: 0,
                total_size: 0,
                value_size: 0,
                item_stride: 0,
                items_start: 0,
            };
        }

        let table_cap = cap.next_power_of_two();
        let items_cap = table_cap * LOAD_FACTOR_NUM / LOAD_FACTOR_DEN;
        let table_mask = (table_cap - 1) as Id;

        let align = if value_align > 2 { value_align } else { 2 };
        let table_end = table_cap * mem::size_of::<Id>();
        let items_start = align_forward(table_end, align);

        let meta_size = mem::size_of::<ItemMeta>();
        let raw_stride = value_size + meta_size;
        let item_stride = align_forward(raw_stride, align);

        let items_end = items_start + items_cap * item_stride;
        let total_size = items_end;

        Self {
            cap: items_cap,
            table_cap,
            table_mask,
            total_size,
            value_size,
            item_stride,
            items_start,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RefCountedSet {
    table: Offset,
    items: Offset,
    max_psl: Id,
    psl_stats: [Id; PSL_STATS_LEN],
    living: usize,
    next_id: Id,
    layout: RefCountedSetLayout,
}

impl RefCountedSet {
    pub fn new() -> Self {
        Self {
            table: Offset::default(),
            items: Offset::default(),
            max_psl: 0,
            psl_stats: [0; PSL_STATS_LEN],
            living: 0,
            next_id: 1,
            layout: RefCountedSetLayout {
                cap: 0,
                table_cap: 0,
                table_mask: 0,
                total_size: 0,
                value_size: 0,
                item_stride: 0,
                items_start: 0,
            },
        }
    }

    pub fn init(buf: OffsetBuf, l: RefCountedSetLayout) -> Self {
        let table_offset = buf.member(0);
        let items_offset = buf.member(l.items_start);
        let base: *mut u8 = buf.base;

        unsafe {
            let tbl: *mut u8 = table_offset.ptr_mut::<u8>(base);
            ptr::write_bytes(tbl, 0, l.table_cap * mem::size_of::<Id>());

            let itms: *mut u8 = items_offset.ptr_mut::<u8>(base);
            ptr::write_bytes(itms, 0, l.cap * l.item_stride);

            let meta_off = l.value_size;
            let mut i: usize = 0;
            while i < l.cap {
                let slot = itms.add(i * l.item_stride);
                let meta: *mut ItemMeta = slot.add(meta_off) as *mut ItemMeta;
                ptr::write(meta, ItemMeta::empty());
                i += 1;
            }
        }

        Self {
            table: table_offset,
            items: items_offset,
            max_psl: 0,
            psl_stats: [0; PSL_STATS_LEN],
            living: 0,
            next_id: 1,
            layout: l,
        }
    }

    #[inline]
    pub fn capacity(&self) -> u16 {
        self.layout.cap as u16
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.living
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.living == 0
    }

    #[inline]
    pub fn living_count(&self) -> usize {
        self.living
    }

    #[inline]
    unsafe fn slot_ptr(&self, base: *const u8, id: Id) -> *const u8 {
        unsafe {
            let p: *const u8 = self.items.ptr::<u8>(base);
            p.add(id as usize * self.layout.item_stride)
        }
    }

    #[inline]
    unsafe fn slot_ptr_mut(&self, base: *mut u8, id: Id) -> *mut u8 {
        unsafe {
            let p: *mut u8 = self.items.ptr_mut::<u8>(base);
            p.add(id as usize * self.layout.item_stride)
        }
    }

    #[inline]
    unsafe fn meta_at(&self, base: *const u8, id: Id) -> ItemMeta {
        unsafe {
            let slot = self.slot_ptr(base, id);
            let mp: *const ItemMeta = slot.add(self.layout.value_size) as *const ItemMeta;
            ptr::read(mp)
        }
    }

    #[inline]
    unsafe fn meta_ptr_mut(&self, base: *mut u8, id: Id) -> *mut ItemMeta {
        unsafe {
            let slot = self.slot_ptr_mut(base, id);
            slot.add(self.layout.value_size) as *mut ItemMeta
        }
    }

    #[inline]
    unsafe fn read_meta_or_default(&self, base: *const u8, id: Id) -> ItemMeta {
        unsafe {
            if (id as usize) >= self.layout.cap {
                return ItemMeta::empty();
            }
            let slot = self.slot_ptr(base, id);
            let mp: *const ItemMeta = slot.add(self.layout.value_size) as *const ItemMeta;
            ptr::read(mp)
        }
    }

    #[inline]
    unsafe fn write_meta(&self, base: *mut u8, id: Id, m: ItemMeta) {
        unsafe {
            let p = self.meta_ptr_mut(base, id);
            ptr::write(p, m);
        }
    }

    #[inline]
    unsafe fn table_ptr(&self, base: *const u8) -> *const Id {
        unsafe { self.table.ptr::<Id>(base) }
    }

    #[inline]
    unsafe fn table_ptr_mut(&self, base: *mut u8) -> *mut Id {
        unsafe { self.table.ptr_mut::<Id>(base) }
    }

    #[inline]
    unsafe fn psl_stat(&self, i: usize) -> Id {
        unsafe { ptr::read(self.psl_stats.as_ptr().add(i)) }
    }

    #[inline]
    unsafe fn psl_stat_inc(&mut self, i: usize) {
        unsafe {
            let p = self.psl_stats.as_mut_ptr().add(i);
            *p = (*p).wrapping_add(1);
        }
    }

    #[inline]
    unsafe fn psl_stat_dec(&mut self, i: usize) {
        unsafe {
            let p = self.psl_stats.as_mut_ptr().add(i);
            *p = (*p).wrapping_sub(1);
        }
    }

    #[inline]
    fn table_bucket_mask(&self) -> usize {
        self.layout.table_mask as usize
    }

    #[inline]
    unsafe fn shrink_max_psl(&mut self) {
        unsafe {
            while self.max_psl > 0 && self.psl_stat(self.max_psl as usize) == 0 {
                self.max_psl -= 1;
            }
        }
    }

    pub unsafe fn get<T: Copy>(&self, base: *const u8, id: u16) -> T {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let slot = self.slot_ptr(base, id);
            ptr::read(slot as *const T)
        }
    }

    pub unsafe fn get_ref<'a, T>(&self, base: *const u8, id: u16) -> &'a T {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let slot = self.slot_ptr(base, id);
            &*(slot as *const T)
        }
    }

    pub unsafe fn get_mut<'a, T>(&self, base: *mut u8, id: u16) -> &'a mut T {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let slot = self.slot_ptr_mut(base, id);
            &mut *(slot as *mut T)
        }
    }

    pub unsafe fn ref_count(&self, base: *const u8, id: u16) -> RefCount {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe { self.meta_at(base, id).ref_count }
    }

    pub unsafe fn add_id(&mut self, base: *mut u8, id: u16) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let m = self.meta_ptr_mut(base, id);
            debug_assert!((*m).ref_count > 0);
            (*m).ref_count = (*m).ref_count.wrapping_add(1);
        }
    }

    pub unsafe fn use_id(&mut self, base: *mut u8, id: u16) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let m = self.meta_ptr_mut(base, id);
            debug_assert!((*m).ref_count > 0);
            (*m).ref_count = (*m).ref_count.wrapping_add(1);
        }
    }

    pub unsafe fn use_multiple(&mut self, base: *mut u8, id: u16, n: RefCount) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let m = self.meta_ptr_mut(base, id);
            debug_assert!((*m).ref_count > 0);
            (*m).ref_count = (*m).ref_count.wrapping_add(n);
        }
    }

    pub unsafe fn release(&mut self, base: *mut u8, id: u16) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let m = self.meta_ptr_mut(base, id);
            debug_assert!((*m).ref_count > 0);
            (*m).ref_count = (*m).ref_count.wrapping_sub(1);
            if (*m).ref_count == 0 {
                self.living = self.living.wrapping_sub(1);
            }
        }
    }

    pub unsafe fn release_multiple(&mut self, base: *mut u8, id: u16, n: RefCount) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            let m = self.meta_ptr_mut(base, id);
            debug_assert!((*m).ref_count >= n);
            (*m).ref_count = (*m).ref_count.wrapping_sub(n);
            if (*m).ref_count == 0 {
                self.living = self.living.wrapping_sub(1);
            }
        }
    }

    pub unsafe fn remove(&mut self, base: *mut u8, id: u16) {
        debug_assert!(id > DEFAULT_ID);
        debug_assert!((id as usize) < self.layout.cap);
        unsafe {
            self.release(base, id);
            self.trim_dead_trailing(base);
        }
    }

    pub unsafe fn next_id<T: Copy>(&mut self, base: *mut u8, value: T) -> Option<u16> {
        if self.layout.cap == 0 {
            return None;
        }
        if (self.next_id as usize) >= self.layout.cap {
            return None;
        }
        unsafe {
            let id = self.next_id;
            let slot = self.slot_ptr_mut(base, id);
            ptr::write(slot as *mut T, value);
            self.write_meta(
                base,
                id,
                ItemMeta {
                    bucket: Id::MAX,
                    psl: 0,
                    ref_count: 1,
                },
            );
            self.living = self.living.wrapping_add(1);
            self.next_id = id.wrapping_add(1);
            Some(id)
        }
    }

    unsafe fn delete_item_raw(&mut self, base: *mut u8, id: Id) {
        unsafe {
            if self.layout.table_cap == 0 {
                self.write_meta(base, id, ItemMeta::empty());
                return;
            }
            let m = self.meta_at(base, id);
            if !m.is_in_table(self.layout.table_cap) {
                self.write_meta(base, id, ItemMeta::empty());
                return;
            }
            self.delete_from_table(base, id, m);
        }
    }

    unsafe fn delete_item_with_ctx<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        id: Id,
    ) {
        unsafe {
            if self.layout.table_cap == 0 {
                self.write_meta(base, id, ItemMeta::empty());
                return;
            }
            let m = self.meta_at(base, id);
            if !m.is_in_table(self.layout.table_cap) {
                self.write_meta(base, id, ItemMeta::empty());
                return;
            }

            let slot = self.slot_ptr(base as *const u8, id);
            let value_ref: &T = &*(slot as *const T);
            C::deleted(self, value_ref);

            self.delete_from_table(base, id, m);
        }
    }

    unsafe fn delete_from_table(&mut self, base: *mut u8, id: Id, m: ItemMeta) {
        unsafe {
            self.psl_stat_dec(m.psl as usize);

            let table: *mut Id = self.table_ptr_mut(base);
            ptr::write(table.add(m.bucket as usize), 0);
            self.write_meta(base, id, ItemMeta::empty());

            let mask = self.table_bucket_mask();
            let mut p: usize = m.bucket as usize;
            let mut n: usize = (p.wrapping_add(1)) & mask;

            loop {
                let entry = ptr::read(table.add(n));
                if entry == 0 {
                    break;
                }
                let em = self.meta_ptr_mut(base, entry);
                let entry_psl = (*em).psl;
                if entry_psl == 0 {
                    break;
                }

                (*em).bucket = p as Id;
                self.psl_stat_dec(entry_psl as usize);
                (*em).psl = entry_psl.wrapping_sub(1);
                self.psl_stat_inc((*em).psl as usize);

                ptr::copy_nonoverlapping(table.add(n), table.add(p), 1);
                p = n;
                n = (p.wrapping_add(1)) & mask;
            }

            self.shrink_max_psl();
            ptr::write(table.add(p), 0);
        }
    }

    pub unsafe fn lookup<T, C: RefCountedSetContext<T>>(
        &self,
        base: *const u8,
        value: &T,
    ) -> Option<Id> {
        if self.layout.table_cap == 0 || self.max_psl == 0 && self.living == 0 {
            return None;
        }
        unsafe {
            let hash = C::hash(value);
            let mask = self.table_bucket_mask();
            let table: *const Id = self.table_ptr(base);
            let limit = (self.max_psl as usize).wrapping_add(1);

            let mut i: usize = 0;
            while i < limit {
                let p: usize = ((hash as usize).wrapping_add(i)) & mask;
                let entry = ptr::read(table.add(p));

                if entry == 0 {
                    return None;
                }

                let m = self.meta_at(base, entry);

                if m.psl < (i as Id) {
                    return None;
                }

                if m.psl == (i as Id) && m.ref_count > 0 {
                    let slot = self.slot_ptr(base, entry);
                    let stored: &T = &*(slot as *const T);
                    if C::eql(value, stored) {
                        return Some(entry);
                    }
                }

                i += 1;
            }

            None
        }
    }

    unsafe fn insert<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        value: T,
        new_id: Id,
    ) -> Id {
        unsafe {
            let hash = C::hash(&value);
            let mask = self.table_bucket_mask();
            let table_cap = self.layout.table_cap;
            let table: *mut Id = self.table_ptr_mut(base);

            let mut new_meta = ItemMeta {
                bucket: Id::MAX,
                psl: 0,
                ref_count: 0,
            };
            let mut held_meta_ptr: *mut ItemMeta = &raw mut new_meta;
            let mut held_id: Id = new_id;
            let mut chosen_id: Id = new_id;

            let iters = table_cap.wrapping_sub(1);
            let mut i: usize = 0;
            while i < iters {
                let p: usize = ((hash as usize).wrapping_add(i)) & mask;
                let entry = ptr::read(table.add(p));

                if entry == 0 {
                    ptr::write(table.add(p), held_id);
                    (*held_meta_ptr).bucket = p as Id;
                    self.psl_stat_inc((*held_meta_ptr).psl as usize);
                    if (*held_meta_ptr).psl > self.max_psl {
                        self.max_psl = (*held_meta_ptr).psl;
                    }
                    break;
                }

                let entry_m = self.meta_at(base, entry);

                if entry_m.ref_count == 0 {
                    if entry_m.is_in_table(table_cap) {
                        self.psl_stat_dec(entry_m.psl as usize);
                    }
                    self.write_meta(base, entry, ItemMeta::empty());

                    if entry < new_id {
                        chosen_id = entry;
                    }

                    ptr::write(table.add(p), held_id);
                    (*held_meta_ptr).bucket = p as Id;
                    self.psl_stat_inc((*held_meta_ptr).psl as usize);
                    if (*held_meta_ptr).psl > self.max_psl {
                        self.max_psl = (*held_meta_ptr).psl;
                    }
                    break;
                }

                let held_psl = (*held_meta_ptr).psl;
                let held_ref = (*held_meta_ptr).ref_count;

                if entry_m.psl < held_psl
                    || (entry_m.psl == held_psl && entry_m.ref_count < held_ref)
                {
                    ptr::write(table.add(p), held_id);
                    (*held_meta_ptr).bucket = p as Id;
                    self.psl_stat_inc(held_psl as usize);
                    if held_psl > self.max_psl {
                        self.max_psl = held_psl;
                    }

                    held_id = entry;
                    held_meta_ptr = self.meta_ptr_mut(base, entry);
                    self.psl_stat_dec(entry_m.psl as usize);
                }

                (*held_meta_ptr).psl = (*held_meta_ptr).psl.wrapping_add(1);
                i += 1;
            }

            ptr::write(table.add(new_meta.bucket as usize), chosen_id);

            let chosen_slot = self.slot_ptr_mut(base, chosen_id);
            ptr::copy_nonoverlapping(
                &value as *const T as *const u8,
                chosen_slot,
                mem::size_of::<T>(),
            );
            let chosen_meta: *mut ItemMeta =
                chosen_slot.add(self.layout.value_size) as *mut ItemMeta;
            ptr::write(chosen_meta, new_meta);

            mem::forget(value);
            chosen_id
        }
    }

    unsafe fn upsert<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        value: T,
        new_id: Id,
    ) -> Id {
        unsafe {
            if let Some(existing) = self.lookup::<T, C>(base, &value) {
                C::deleted(self, &value);
                mem::forget(value);
                return existing;
            }
            self.insert::<T, C>(base, value, new_id)
        }
    }

    pub unsafe fn add<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        value: T,
    ) -> Result<Id, AddError> {
        unsafe { self.add_context::<T, C>(base, value) }
    }

    pub unsafe fn add_context<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        value: T,
    ) -> Result<Id, AddError> {
        unsafe {
            self.trim_dead_trailing_ctx::<T, C>(base);

            if let Some(id) = self.lookup::<T, C>(base, &value) {
                C::deleted(self, &value);
                mem::forget(value);
                let m = self.meta_ptr_mut(base, id);
                (*m).ref_count = (*m).ref_count.wrapping_add(1);
                return Ok(id);
            }

            if self.psl_stat(PSL_STATS_LEN - 1) > 0 {
                mem::forget(value);
                return Err(AddError::OutOfMemory);
            }

            if (self.next_id as usize) >= self.layout.cap {
                let threshold = self.layout.cap * REHASH_THRESHOLD_NUM / REHASH_THRESHOLD_DEN;
                if self.living < threshold {
                    mem::forget(value);
                    return Err(AddError::NeedsRehash);
                }
                mem::forget(value);
                return Err(AddError::OutOfMemory);
            }

            let id = self.insert::<T, C>(base, value, self.next_id);

            let m = self.meta_ptr_mut(base, id);
            (*m).ref_count = (*m).ref_count.wrapping_add(1);
            debug_assert!((*m).ref_count == 1);
            self.living = self.living.wrapping_add(1);

            if id == self.next_id {
                self.next_id = id.wrapping_add(1);
            }

            Ok(id)
        }
    }

    unsafe fn trim_dead_trailing_ctx<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
    ) {
        if self.layout.cap == 0 {
            return;
        }
        unsafe {
            while self.next_id > 1 {
                let check_id = self.next_id.wrapping_sub(1);
                if (check_id as usize) >= self.layout.cap {
                    break;
                }
                let m = self.meta_at(base, check_id);
                if m.ref_count != 0 {
                    break;
                }
                self.next_id = check_id;
                self.delete_item_with_ctx::<T, C>(base, check_id);
            }
        }
    }

    unsafe fn trim_dead_trailing(&mut self, base: *mut u8) {
        if self.layout.cap == 0 {
            return;
        }
        unsafe {
            while self.next_id > 1 {
                let check_id = self.next_id.wrapping_sub(1);
                if (check_id as usize) >= self.layout.cap {
                    break;
                }
                let m = self.meta_at(base, check_id);
                if m.ref_count != 0 {
                    break;
                }
                self.next_id = check_id;
                self.delete_item_raw(base, check_id);
            }
        }
    }

    pub unsafe fn add_with_id<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        value: T,
        id: Id,
    ) -> Result<Option<Id>, AddError> {
        unsafe { self.add_with_id_context::<T, C>(base, value, id) }
    }

    pub unsafe fn add_with_id_context<T, C: RefCountedSetContext<T>>(
        &mut self,
        base: *mut u8,
        value: T,
        id: Id,
    ) -> Result<Option<Id>, AddError> {
        debug_assert!(id > 0);

        unsafe {
            if (id as usize) < self.layout.cap && id < self.next_id {
                let m = self.meta_at(base, id);
                if m.ref_count == 0 {
                    if self.psl_stat(PSL_STATS_LEN - 1) > 0 {
                        mem::forget(value);
                        return Err(AddError::OutOfMemory);
                    }

                    self.delete_item_with_ctx::<T, C>(base, id);
                    let added_id = self.upsert::<T, C>(base, value, id);

                    let am = self.meta_ptr_mut(base, added_id);
                    (*am).ref_count = (*am).ref_count.wrapping_add(1);
                    self.living = self.living.wrapping_add(1);

                    if added_id == id {
                        return Ok(None);
                    } else {
                        return Ok(Some(added_id));
                    }
                }

                let slot = self.slot_ptr(base, id);
                let stored: &T = &*(slot as *const T);
                if C::eql(&value, stored) {
                    C::deleted(self, &value);
                    mem::forget(value);
                    let em = self.meta_ptr_mut(base, id);
                    (*em).ref_count = (*em).ref_count.wrapping_add(1);
                    return Ok(None);
                }
            }

            let added = self.add_context::<T, C>(base, value)?;
            Ok(Some(added))
        }
    }

    pub unsafe fn rehash<T, C: RefCountedSetContext<T>>(&mut self, base: *mut u8) {
        if self.layout.table_cap == 0 {
            return;
        }
        unsafe {
            while self.next_id > 1 {
                let check_id = self.next_id.wrapping_sub(1);
                if (check_id as usize) >= self.layout.cap {
                    break;
                }
                let m = self.meta_at(base, check_id);
                if m.ref_count != 0 {
                    break;
                }
                self.next_id = check_id;
                self.write_meta(base, check_id, ItemMeta::empty());
            }

            let table: *mut Id = self.table_ptr_mut(base);
            ptr::write_bytes(table, 0, self.layout.table_cap);

            self.max_psl = 0;
            ptr::write_bytes(self.psl_stats.as_mut_ptr(), 0, PSL_STATS_LEN);

            let mask = self.table_bucket_mask();

            let mut id_idx: Id = 1;
            while (id_idx as usize) < self.layout.cap && id_idx < self.next_id {
                let m = self.meta_at(base, id_idx);
                if m.ref_count == 0 {
                    id_idx = id_idx.wrapping_add(1);
                    continue;
                }

                let slot = self.slot_ptr(base, id_idx);
                let value_ref: &T = &*(slot as *const T);
                let hash = C::hash(value_ref);

                let mut held_id: Id = id_idx;
                let mut held_psl: Id = 0;

                let mut placed = false;
                let mut probe: usize = 0;
                while probe < self.layout.table_cap {
                    let p: usize = ((hash as usize).wrapping_add(probe)) & mask;
                    let entry = ptr::read(table.add(p));

                    if entry == 0 {
                        ptr::write(table.add(p), held_id);
                        let em = self.meta_ptr_mut(base, held_id);
                        (*em).bucket = p as Id;
                        (*em).psl = held_psl;
                        self.psl_stat_inc(held_psl as usize);
                        if held_psl > self.max_psl {
                            self.max_psl = held_psl;
                        }
                        placed = true;
                        break;
                    }

                    let em_ref = self.meta_ptr_mut(base, entry);
                    let entry_psl = (*em_ref).psl;
                    let entry_ref = (*em_ref).ref_count;

                    let hm_ref = {
                        let hm = self.meta_ptr_mut(base, held_id);
                        (*hm).ref_count
                    };

                    if entry_psl < held_psl
                        || (entry_psl == held_psl && entry_ref < hm_ref)
                    {
                        ptr::write(table.add(p), held_id);
                        let hm = self.meta_ptr_mut(base, held_id);
                        (*hm).bucket = p as Id;
                        (*hm).psl = held_psl;
                        self.psl_stat_inc(held_psl as usize);
                        if held_psl > self.max_psl {
                            self.max_psl = held_psl;
                        }

                        held_id = entry;
                        held_psl = entry_psl;
                        self.psl_stat_dec(entry_psl as usize);
                    }

                    held_psl = held_psl.wrapping_add(1);
                    probe += 1;
                }

                if !placed {
                    break;
                }

                id_idx = id_idx.wrapping_add(1);
            }
        }
    }
}
