use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::highlight::Pin;
use crate::kitty_graphics_command::*;
use crate::kitty_graphics_image::*;
use crate::kitty_placement_layer::kitty_placement_layer_matches_impl;

#[repr(C)]
pub struct KittyImageSnapshot {
    pub id: u32,
    pub number: u32,
    pub width: u32,
    pub height: u32,
    pub format: c_int,
    pub compression: c_int,
    pub data_ptr: *const u8,
    pub data_len: usize,
}

#[repr(C)]
pub struct KittyPlacementSnapshot {
    pub image_id: u32,
    pub placement_id: u32,
    pub is_virtual: bool,
    pub pin_node: *mut c_void,
    pub pin_x: u16,
    pub pin_y: u16,
    pub x_offset: u32,
    pub y_offset: u32,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub z: i32,
}

pub(crate) const DEFAULT_TOTAL_LIMIT: usize = 320 * 1000 * 1000;
pub(crate) const DEFAULT_NEXT_IMAGE_ID: u32 = 2147483647;
pub(crate) const MAX_IMAGES: usize = 256;
pub(crate) const MAX_PLACEMENTS: usize = 1024;
pub(crate) const DEFAULT_SCRATCH_CAP: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct PlacementKey {
    pub image_id: u32,
    pub is_internal: bool,
    pub placement_id: u32,
}

impl PlacementKey {
    pub(crate) fn eq(&self, other: &PlacementKey) -> bool {
        self.image_id == other.image_id
            && self.is_internal == other.is_internal
            && self.placement_id == other.placement_id
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Placement {
    pub location_is_virtual: bool,
    pub pin: *mut c_void,
    pub pin_node: *mut c_void,
    pub pin_x: u16,
    pub pin_y: u16,
    pub x_offset: u32,
    pub y_offset: u32,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub z: i32,
}

impl Placement {
    pub(crate) fn new() -> Self {
        Self {
            location_is_virtual: false,
            pin: ptr::null_mut(),
            pin_node: ptr::null_mut(),
            pin_x: 0,
            pin_y: 0,
            x_offset: 0,
            y_offset: 0,
            source_x: 0,
            source_y: 0,
            source_width: 0,
            source_height: 0,
            columns: 0,
            rows: 0,
            z: 0,
        }
    }

    pub(crate) fn pixel_size(
        &self,
        image: &Image,
        terminal_width_px: u32,
        terminal_height_px: u32,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> (u32, u32) {
        let width = if self.source_width > 0 {
            self.source_width
        } else {
            image.width
        };
        let height = if self.source_height > 0 {
            self.source_height
        } else {
            image.height
        };

        if self.columns == 0 && self.rows == 0 {
            return (width, height);
        }

        let cell_width = nonzero_div(terminal_width_px, terminal_cols as u32);
        let cell_height = nonzero_div(terminal_height_px, terminal_rows as u32);

        if self.columns > 0 && self.rows > 0 {
            return (
                cell_width.wrapping_mul(self.columns),
                cell_height.wrapping_mul(self.rows),
            );
        }

        if self.columns > 0 {
            let calc_width = cell_width.wrapping_mul(self.columns);
            let aspect = (height as f64) / (width as f64);
            let calc_height = round_f64((calc_width as f64) * aspect);
            return (calc_width, calc_height);
        }

        let calc_height = cell_height.wrapping_mul(self.rows);
        let aspect = (width as f64) / (height as f64);
        let calc_width = round_f64((calc_height as f64) * aspect);
        (calc_width, calc_height)
    }

    pub(crate) fn grid_size(
        &self,
        image: &Image,
        terminal_width_px: u32,
        terminal_height_px: u32,
        terminal_cols: u16,
        terminal_rows: u16,
    ) -> (u32, u32) {
        if self.columns > 0 && self.rows > 0 {
            return (self.columns, self.rows);
        }

        let (pixel_width, pixel_height) = self.pixel_size(
            image,
            terminal_width_px,
            terminal_height_px,
            terminal_cols,
            terminal_rows,
        );
        let cell_width = nonzero_div(terminal_width_px, terminal_cols as u32);
        let cell_height = nonzero_div(terminal_height_px, terminal_rows as u32);

        (
            div_ceil(pixel_width.wrapping_add(self.x_offset), cell_width),
            div_ceil(pixel_height.wrapping_add(self.y_offset), cell_height),
        )
    }
}

fn nonzero_div(num: u32, den: u32) -> u32 {
    if den == 0 {
        return 0;
    }
    num / den
}

fn div_ceil(num: u32, den: u32) -> u32 {
    if den == 0 {
        return 0;
    }
    let q = num / den;
    let r = num % den;
    if r == 0 {
        q
    } else {
        q + 1
    }
}

fn round_f64(v: f64) -> u32 {
    if v <= 0.0 {
        0
    } else if v >= u32::MAX as f64 {
        u32::MAX
    } else {
        (v + 0.5) as u32
    }
}

#[derive(Clone, Copy)]
struct ImageEntry {
    pub used: bool,
    pub image: Image,
}

#[derive(Clone, Copy)]
struct PlacementEntry {
    pub used: bool,
    pub key: PlacementKey,
    pub placement: Placement,
}

pub(crate) struct ImageStorage {
    pub dirty: bool,
    pub next_image_id: u32,
    pub next_internal_placement_id: u32,

    images: [ImageEntry; MAX_IMAGES],
    image_count: usize,

    placements: [PlacementEntry; MAX_PLACEMENTS],
    placement_count: usize,

    loading_image: Option<LoadingImage>,

    pub image_limits: LoadingLimits,
    pub total_bytes: usize,
    pub total_limit: usize,

    scratch: *mut u8,
    scratch_len: usize,
    scratch_cap_val: usize,
}

impl ImageStorage {
    pub(crate) fn new(scratch_buf: *mut u8, scratch_cap: usize) -> Self {
        Self {
            dirty: false,
            next_image_id: DEFAULT_NEXT_IMAGE_ID,
            next_internal_placement_id: 0,
            images: [ImageEntry {
                used: false,
                image: Image::new(),
            }; MAX_IMAGES],
            image_count: 0,
            placements: [PlacementEntry {
                used: false,
                key: PlacementKey {
                    image_id: 0,
                    is_internal: false,
                    placement_id: 0,
                },
                placement: Placement::new(),
            }; MAX_PLACEMENTS],
            placement_count: 0,
            loading_image: None,
            image_limits: LoadingLimits::direct_only(),
            total_bytes: 0,
            total_limit: DEFAULT_TOTAL_LIMIT,
            scratch: scratch_buf,
            scratch_len: 0,
            scratch_cap_val: scratch_cap,
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.total_limit != 0
    }

    pub(crate) fn scratch_buf_mut(&mut self) -> *mut u8 {
        self.scratch
    }

    pub(crate) fn scratch_cap(&self) -> usize {
        self.scratch_cap_val
    }

    pub(crate) fn loading_mut(&mut self) -> Option<&mut LoadingImage> {
        match &mut self.loading_image {
            Some(l) => Some(l),
            None => None,
        }
    }

    pub(crate) fn set_loading(&mut self, loading: LoadingImage) {
        self.loading_image = Some(loading);
    }

    pub(crate) fn clear_loading(&mut self) {
        self.loading_image = None;
    }

    pub(crate) fn image_by_id(&self, image_id: u32) -> Option<Image> {
        let mut i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked(i) };
            if entry.used && entry.image.id == image_id {
                return Some(entry.image);
            }
            i += 1;
        }
        None
    }

    pub(crate) fn image_ref_by_id(&self, image_id: u32) -> *const Image {
        let mut i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked(i) };
            if entry.used && entry.image.id == image_id {
                return &entry.image as *const Image;
            }
            i += 1;
        }
        ptr::null()
    }

    pub(crate) fn image_by_number(&self, image_number: u32) -> Option<Image> {
        let mut newest: Option<Image> = None;
        let mut i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked(i) };
            if entry.used && entry.image.number == image_number {
                match newest {
                    None => newest = Some(entry.image),
                    Some(ref n) => {
                        if entry.image.transmit_time_ns > n.transmit_time_ns {
                            newest = Some(entry.image);
                        }
                    }
                }
            }
            i += 1;
        }
        newest
    }

    pub(crate) fn add_image(&mut self, img: Image) -> Result<(), ImageError> {
        if img.data_len > self.total_limit {
            return Err(ImageError::OutOfMemory);
        }

        let total = self.total_bytes.wrapping_add(img.data_len);
        if total > self.total_limit {
            let req = total.wrapping_sub(self.total_limit);
            if !self.evict_images(req) {
                return Err(ImageError::OutOfMemory);
            }
        }

        let mut i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked(i) };
            if entry.used && entry.image.id == img.id {
                let entry_mut = unsafe { self.images.get_unchecked_mut(i) };
                self.total_bytes = self.total_bytes.saturating_sub(entry_mut.image.data_len);
                entry_mut.image = img;
                self.total_bytes = self.total_bytes.wrapping_add(img.data_len);
                self.dirty = true;
                return Ok(());
            }
            i += 1;
        }

        let mut free_idx = MAX_IMAGES;
        i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked(i) };
            if !entry.used {
                free_idx = i;
                break;
            }
            i += 1;
        }

        if free_idx >= MAX_IMAGES {
            return Err(ImageError::OutOfMemory);
        }

        let entry = unsafe { self.images.get_unchecked_mut(free_idx) };
        entry.used = true;
        entry.image = img;
        self.image_count += 1;
        self.total_bytes = self.total_bytes.wrapping_add(img.data_len);
        self.dirty = true;
        Ok(())
    }

    pub(crate) fn add_placement(
        &mut self,
        image_id: u32,
        placement_id: u32,
        p: Placement,
    ) -> Result<(), ImageError> {
        let key = PlacementKey {
            image_id,
            is_internal: placement_id == 0,
            placement_id: if placement_id == 0 {
                let id = self.next_internal_placement_id;
                self.next_internal_placement_id = self.next_internal_placement_id.wrapping_add(1);
                id
            } else {
                placement_id
            },
        };

        let mut i = 0;
        while i < MAX_PLACEMENTS {
            let entry = unsafe { self.placements.get_unchecked(i) };
            if entry.used && entry.key.eq(&key) {
                let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                entry_mut.placement = p;
                self.dirty = true;
                return Ok(());
            }
            i += 1;
        }

        let mut free_idx = MAX_PLACEMENTS;
        i = 0;
        while i < MAX_PLACEMENTS {
            let entry = unsafe { self.placements.get_unchecked(i) };
            if !entry.used {
                free_idx = i;
                break;
            }
            i += 1;
        }

        if free_idx >= MAX_PLACEMENTS {
            return Err(ImageError::OutOfMemory);
        }

        let entry = unsafe { self.placements.get_unchecked_mut(free_idx) };
        entry.used = true;
        entry.key = key;
        entry.placement = p;
        self.placement_count += 1;
        self.dirty = true;
        Ok(())
    }

    pub(crate) fn delete(&mut self, cmd: Delete) {
        match cmd {
            Delete::All(delete_images) => {
                let mut i = 0;
                while i < MAX_PLACEMENTS {
                    let entry = unsafe { self.placements.get_unchecked(i) };
                    if entry.used && !entry.placement.location_is_virtual {
                        let image_id = entry.key.image_id;
                        let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                        entry_mut.used = false;
                        self.placement_count = self.placement_count.saturating_sub(1);
                        if delete_images {
                            self.delete_if_unused(image_id);
                        }
                    }
                    i += 1;
                }

                if delete_images {
                    i = 0;
                    while i < MAX_IMAGES {
                        let entry = unsafe { self.images.get_unchecked(i) };
                        if entry.used {
                            self.delete_if_unused(entry.image.id);
                        }
                        i += 1;
                    }
                }

                self.dirty = true;
            }

            Delete::Id(v) => {
                self.delete_by_id(v.image_id, v.placement_id, v.delete);
            }

            Delete::Newest(v) => {
                if let Some(img) = self.image_by_number(v.image_number) {
                    self.delete_by_id(img.id, v.placement_id, v.delete);
                }
            }

            Delete::IntersectCursor(_delete_images) => {
                self.dirty = true;
            }

            Delete::IntersectCell(v) => {
                if v.x == 0 || v.y == 0 {
                    return;
                }
                self.dirty = true;
            }

            Delete::IntersectCellZ(v) => {
                if v.x == 0 || v.y == 0 {
                    return;
                }
                let mut i = 0;
                while i < MAX_PLACEMENTS {
                    let entry = unsafe { self.placements.get_unchecked(i) };
                    if entry.used && entry.placement.z == v.z {
                        let image_id = entry.key.image_id;
                        let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                        entry_mut.used = false;
                        self.placement_count = self.placement_count.saturating_sub(1);
                        if v.delete {
                            self.delete_if_unused(image_id);
                        }
                    }
                    i += 1;
                }
                self.dirty = true;
            }

            Delete::Column(v) => {
                if v.x == 0 {
                    return;
                }
                self.dirty = true;
            }

            Delete::Row(v) => {
                if v.y == 0 {
                    return;
                }
                self.dirty = true;
            }

            Delete::Z(v) => {
                let mut i = 0;
                while i < MAX_PLACEMENTS {
                    let entry = unsafe { self.placements.get_unchecked(i) };
                    if entry.used
                        && !entry.placement.location_is_virtual
                        && entry.placement.z == v.z
                    {
                        let image_id = entry.key.image_id;
                        let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                        entry_mut.used = false;
                        self.placement_count = self.placement_count.saturating_sub(1);
                        if v.delete {
                            self.delete_if_unused(image_id);
                        }
                    }
                    i += 1;
                }
                self.dirty = true;
            }

            Delete::Range(v) => {
                if v.first == 0 || v.last == 0 || v.first > v.last {
                    return;
                }
                let mut i = 0;
                while i < MAX_PLACEMENTS {
                    let entry = unsafe { self.placements.get_unchecked(i) };
                    if entry.used && entry.key.image_id >= v.first && entry.key.image_id <= v.last {
                        let image_id = entry.key.image_id;
                        let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                        entry_mut.used = false;
                        self.placement_count = self.placement_count.saturating_sub(1);
                        if v.delete {
                            self.delete_if_unused(image_id);
                        }
                    }
                    i += 1;
                }
                self.dirty = true;
            }

            Delete::AnimationFrames(_) => {}
        }
    }

    fn delete_by_id(&mut self, image_id: u32, placement_id: u32, delete_unused: bool) {
        if placement_id == 0 {
            let mut i = 0;
            while i < MAX_PLACEMENTS {
                let entry = unsafe { self.placements.get_unchecked(i) };
                if entry.used && entry.key.image_id == image_id {
                    let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                    entry_mut.used = false;
                    self.placement_count = self.placement_count.saturating_sub(1);
                }
                i += 1;
            }
        } else {
            let target_key = PlacementKey {
                image_id,
                is_internal: false,
                placement_id,
            };
            let mut i = 0;
            while i < MAX_PLACEMENTS {
                let entry = unsafe { self.placements.get_unchecked(i) };
                if entry.used && entry.key.eq(&target_key) {
                    let entry_mut = unsafe { self.placements.get_unchecked_mut(i) };
                    entry_mut.used = false;
                    self.placement_count = self.placement_count.saturating_sub(1);
                    break;
                }
                i += 1;
            }
        }

        if delete_unused {
            self.delete_if_unused(image_id);
        }

        self.dirty = true;
    }

    fn delete_if_unused(&mut self, image_id: u32) {
        let mut has_placement = false;
        let mut i = 0;
        while i < MAX_PLACEMENTS {
            let entry = unsafe { self.placements.get_unchecked(i) };
            if entry.used && entry.key.image_id == image_id {
                has_placement = true;
                break;
            }
            i += 1;
        }

        if has_placement {
            return;
        }

        i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked(i) };
            if entry.used && entry.image.id == image_id {
                let entry_mut = unsafe { self.images.get_unchecked_mut(i) };
                self.total_bytes = self.total_bytes.saturating_sub(entry_mut.image.data_len);
                entry_mut.used = false;
                self.image_count = self.image_count.saturating_sub(1);
                break;
            }
            i += 1;
        }
    }

    fn evict_images(&mut self, req: usize) -> bool {
        let mut evicted: usize = 0;

        loop {
            let mut best_idx = MAX_IMAGES;
            let mut best_used = true;
            let mut best_time = u64::MAX;

            let mut i = 0;
            while i < MAX_IMAGES {
                let entry = unsafe { self.images.get_unchecked(i) };
                if !entry.used {
                    i += 1;
                    continue;
                }

                let used = self.image_has_placements(entry.image.id);

                if !used && best_used {
                    best_idx = i;
                    best_used = false;
                    best_time = entry.image.transmit_time_ns;
                } else if !used && !best_used {
                    if entry.image.transmit_time_ns < best_time {
                        best_idx = i;
                        best_time = entry.image.transmit_time_ns;
                    }
                } else if used && best_used {
                    if entry.image.transmit_time_ns < best_time {
                        best_idx = i;
                        best_time = entry.image.transmit_time_ns;
                    }
                }

                i += 1;
            }

            if best_idx >= MAX_IMAGES {
                break;
            }

            let entry = unsafe { self.images.get_unchecked(best_idx) };
            let img_id = entry.image.id;
            let img_bytes = entry.image.data_len;

            let mut j = 0;
            while j < MAX_PLACEMENTS {
                let p_entry = unsafe { self.placements.get_unchecked(j) };
                if p_entry.used && p_entry.key.image_id == img_id {
                    let p_mut = unsafe { self.placements.get_unchecked_mut(j) };
                    p_mut.used = false;
                    self.placement_count = self.placement_count.saturating_sub(1);
                }
                j += 1;
            }

            let entry_mut = unsafe { self.images.get_unchecked_mut(best_idx) };
            evicted = evicted.wrapping_add(img_bytes);
            self.total_bytes = self.total_bytes.saturating_sub(img_bytes);
            entry_mut.used = false;
            self.image_count = self.image_count.saturating_sub(1);

            if evicted >= req {
                return true;
            }
        }

        evicted >= req
    }

    fn image_has_placements(&self, image_id: u32) -> bool {
        let mut i = 0;
        while i < MAX_PLACEMENTS {
            let entry = unsafe { self.placements.get_unchecked(i) };
            if entry.used && entry.key.image_id == image_id {
                return true;
            }
            i += 1;
        }
        false
    }

    pub(crate) fn set_limit(&mut self, limit: usize) {
        if limit == 0 {
            let limits = self.image_limits;
            self.clear_all();
            self.image_limits = limits;
            self.total_limit = 0;
            return;
        }

        if limit < self.total_bytes {
            let req = self.total_bytes.wrapping_sub(limit);
            let _ = self.evict_images(req);
        }

        self.total_limit = limit;
    }

    fn clear_all(&mut self) {
        let mut i = 0;
        while i < MAX_IMAGES {
            let entry = unsafe { self.images.get_unchecked_mut(i) };
            entry.used = false;
            i += 1;
        }
        self.image_count = 0;

        i = 0;
        while i < MAX_PLACEMENTS {
            let entry = unsafe { self.placements.get_unchecked_mut(i) };
            entry.used = false;
            i += 1;
        }
        self.placement_count = 0;

        self.loading_image = None;
        self.total_bytes = 0;
        self.dirty = true;
    }

    pub(crate) fn image_count(&self) -> usize {
        self.image_count
    }

    pub(crate) fn placement_count(&self) -> usize {
        self.placement_count
    }
}

fn image_snapshot(img: &Image) -> KittyImageSnapshot {
    KittyImageSnapshot {
        id: img.id,
        number: img.number,
        width: img.width,
        height: img.height,
        format: img.format as c_int,
        compression: img.compression as c_int,
        data_ptr: img.data_ptr,
        data_len: img.data_len,
    }
}

fn placement_snapshot(key: PlacementKey, placement: Placement) -> KittyPlacementSnapshot {
    let (pin_node, pin_x, pin_y) = if placement.pin.is_null() {
        (placement.pin_node, placement.pin_x, placement.pin_y)
    } else {
        let pin = unsafe { &*(placement.pin as *const Pin) };
        (pin.node as *mut c_void, pin.x, pin.y)
    };

    KittyPlacementSnapshot {
        image_id: key.image_id,
        placement_id: key.placement_id,
        is_virtual: placement.location_is_virtual,
        pin_node,
        pin_x,
        pin_y,
        x_offset: placement.x_offset,
        y_offset: placement.y_offset,
        source_x: placement.source_x,
        source_y: placement.source_y,
        source_width: placement.source_width,
        source_height: placement.source_height,
        columns: placement.columns,
        rows: placement.rows,
        z: placement.z,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_image_get_handle(
    storage: *const c_void,
    image_id: u32,
) -> *const c_void {
    if storage.is_null() {
        return ptr::null();
    }
    let storage = unsafe { &*(storage as *const ImageStorage) };
    storage.image_ref_by_id(image_id) as *const c_void
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_image_snapshot(
    image: *const c_void,
    out: *mut KittyImageSnapshot,
) -> c_int {
    if image.is_null() || out.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }
    let img = unsafe { &*(image as *const Image) };
    unsafe {
        ptr::write(out, image_snapshot(img));
    }
    GHOSTTY_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_kitty_placement_iterator_next(
    storage: *const c_void,
    layer: c_int,
    index: *mut usize,
    out: *mut KittyPlacementSnapshot,
) -> bool {
    if storage.is_null() || index.is_null() || out.is_null() {
        return false;
    }

    let storage = unsafe { &*(storage as *const ImageStorage) };
    let mut i = unsafe { ptr::read(index) };
    while i < MAX_PLACEMENTS {
        let entry = unsafe { storage.placements.get_unchecked(i) };
        i = i.wrapping_add(1);
        unsafe {
            ptr::write(index, i);
        }

        if !entry.used {
            continue;
        }
        if !kitty_placement_layer_matches_impl(layer, entry.placement.z) {
            continue;
        }

        unsafe {
            ptr::write(out, placement_snapshot(entry.key, entry.placement));
        }
        return true;
    }

    false
}
