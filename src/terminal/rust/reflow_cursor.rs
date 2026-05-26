use core::ffi::c_void;
use core::ptr;
use core::mem;

use crate::highlight::Pin;
use crate::hyperlink::HyperlinkPageEntry;
use crate::page_core::{std_capacity, Page};
use crate::page_list_types::PageListNode;
use crate::page_types::*;
use crate::ref_counted_set::{RefCountedSet, RefCountedSetContext, DEFAULT_ID};
use crate::size_types::*;
use crate::style_types::Style;

const KITTY_GRAPHICS_UNICODE_PLACEHOLDER: u32 = 0xE0B6;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    Success = 0,
    Repeat = 1,
    SkipNext = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReflowError {
    OutOfMemory,
    OutOfSpace,
}

pub struct StyleContext;

impl RefCountedSetContext<Style> for StyleContext {
    #[inline]
    fn hash(value: &Style) -> u64 {
        value.hash()
    }
    #[inline]
    fn eql(a: &Style, b: &Style) -> bool {
        a.eql(b)
    }
}

#[repr(C)]
struct TrackedPinArray {
    keys: *mut *mut Pin,
    len: usize,
}

fn tracked_pins_slice(list: *const PageListType) -> (*mut *mut Pin, usize) {
    unsafe {
        let raw: *mut c_void = (*list).tracked_pins;
        if raw.is_null() {
            return (ptr::null_mut(), 0);
        }
        let arr = &*(raw as *const TrackedPinArray);
        if arr.keys.is_null() || arr.len == 0 {
            return (ptr::null_mut(), 0);
        }
        (arr.keys, arr.len)
    }
}

type PageListType = crate::page_list_types::PageList;

#[repr(C)]
pub struct ReflowCursor {
    pub x: CellCountInt,
    pub y: CellCountInt,
    pub pending_wrap: bool,
    pub node: *mut PageListNode,
    pub page: *mut Page,
    pub page_row: *mut Row,
    pub page_cell: *mut Cell,
    pub new_rows: usize,
    pub total_rows: usize,
}

impl ReflowCursor {
    pub fn init(node: *mut PageListNode) -> Self {
        unsafe {
            let page: *mut Page = &mut (*node).data;
            let rows_ptr: *mut Row = (*page).rows.ptr_mut((*page).memory);
            let first_row: *mut Row = rows_ptr;
            let cells_offset = (*first_row).cells();
            let cells_ptr: *mut Cell = cells_offset.ptr_mut((*page).memory);
            let total_rows = (*page).size.rows as usize;
            ReflowCursor {
                x: 0,
                y: 0,
                pending_wrap: false,
                node,
                page,
                page_row: first_row,
                page_cell: cells_ptr,
                new_rows: 0,
                total_rows,
            }
        }
    }

    #[inline]
    pub fn bottom(&self) -> bool {
        unsafe { self.y == (*self.page).capacity.rows - 1 }
    }

    pub fn cursor_forward(&mut self) {
        unsafe {
            if self.x == (*self.page).size.cols - 1 {
                self.pending_wrap = true;
            } else {
                self.page_cell = self.page_cell.add(1);
                self.x += 1;
            }
        }
    }

    pub fn cursor_scroll(&mut self) {
        unsafe {
            debug_assert!(self.y == (*self.page).size.rows - 1);
            debug_assert!((*self.page).size.rows < (*self.page).capacity.rows);

            (*self.page).size.rows += 1;

            let next_row = self.page_row.add(1);
            self.page_row = next_row;

            let cells_offset = (*next_row).cells();
            let cells_ptr: *mut Cell = cells_offset.ptr_mut((*self.page).memory);
            self.page_cell = cells_ptr;
            self.pending_wrap = false;
            self.x = 0;
            self.y += 1;
        }
    }

    pub fn cursor_new_page(
        &mut self,
        list: *mut PageListType,
        cap: PageCapacity,
    ) -> Result<(), ReflowError> {
        unsafe {
            let new_rows = self.new_rows;
            let node = create_page_node(list, cap)?;
            (*node).data.size.rows = 1;
            let self_node = self.node;
            let pages = &mut (*list).pages;
            pages.insert_after(self_node, node);
            *self = Self::init(node);
            self.new_rows = new_rows;
            Ok(())
        }
    }

    pub fn cursor_scroll_or_new_page(
        &mut self,
        list: *mut PageListType,
        cap: PageCapacity,
    ) -> Result<(), ReflowError> {
        let new_total_rows = self.total_rows + 1;
        if self.bottom() {
            self.cursor_new_page(list, cap)?;
        } else {
            self.cursor_scroll();
        }
        self.total_rows = new_total_rows;
        Ok(())
    }

    pub fn cursor_absolute(&mut self, x: CellCountInt, y: CellCountInt) {
        unsafe {
            debug_assert!((x as usize) < (*self.page).size.cols as usize);
            debug_assert!((y as usize) < (*self.page).size.rows as usize);

            let rows: *mut Row = self.page_row;
            let row: *mut Row = if y == self.y {
                self.page_row
            } else if y < self.y {
                rows.sub((self.y - y) as usize)
            } else {
                rows.add((y - self.y) as usize)
            };

            self.page_row = row;

            let cells_offset = (*row).cells();
            let cells_ptr: *mut Cell = cells_offset.ptr_mut((*self.page).memory);
            self.page_cell = cells_ptr.add(x as usize);
            self.pending_wrap = false;
            self.x = x;
            self.y = y;
        }
    }

    pub fn count_trailing_empty_cells(&self) -> usize {
        unsafe {
            if (*self.page_row).wrap() {
                return 0;
            }

            let cells: *mut Cell = self.page_cell;
            let len = (*self.page).size.cols as usize - self.x as usize;
            let mut i = 0usize;
            while i < len {
                let rev_i = len - i - 1;
                if !(*cells.add(rev_i)).is_empty() {
                    return i;
                }
                i += 1;
            }

            if (*self.page_row).semantic_prompt() != SemanticPrompt::None {
                return len - 1;
            }
            len
        }
    }

    pub fn copy_row_metadata(&self, other: *const Row) {
        unsafe {
            let sp = (*other).semantic_prompt();
            (*self.page_row).set_semantic_prompt(sp);
        }
    }

    pub fn write_cell(
        &mut self,
        list: *mut PageListType,
        cell: *const Cell,
        src_page: *const Page,
    ) -> Result<WriteOutcome, ReflowError> {
        unsafe {
            let cell_val = ptr::read(cell);
            let page = &mut *self.page;
            let dst_cell = self.page_cell;
            let dst_row = self.page_row;

            match cell_val.content_tag() {
                ContentTag::Codepoint | ContentTag::CodepointGrapheme => {
                    match cell_val.wide() {
                        Wide::Narrow => {
                            ptr::write(dst_cell, cell_val);
                        }

                        Wide::Wide => {
                            if page.size.cols > 1 {
                                if self.x == page.size.cols - 1 {
                                    let mut spacer = Cell(0);
                                    spacer.set_content_tag(ContentTag::Codepoint);
                                    spacer.set_content_codepoint(0);
                                    spacer.set_wide(Wide::SpacerHead);
                                    ptr::write(dst_cell, spacer);
                                    self.cursor_forward();
                                    return Ok(WriteOutcome::Repeat);
                                } else {
                                    ptr::write(dst_cell, cell_val);
                                }
                            } else {
                                let mut empty = Cell(0);
                                empty.set_content_tag(ContentTag::Codepoint);
                                empty.set_content_codepoint(0);
                                empty.set_wide(Wide::Narrow);
                                ptr::write(dst_cell, empty);
                                self.cursor_forward();
                                return Ok(WriteOutcome::SkipNext);
                            }
                        }

                        Wide::SpacerTail => {
                            if page.size.cols > 1 {
                                ptr::write(dst_cell, cell_val);
                            } else {
                                return Ok(WriteOutcome::Success);
                            }
                        }

                        Wide::SpacerHead => {
                            return Ok(WriteOutcome::Success);
                        }
                    }
                }

                ContentTag::BgColorPalette | ContentTag::BgColorRgb => {
                    ptr::write(dst_cell, cell_val);
                    self.cursor_forward();
                    return Ok(WriteOutcome::Success);
                }
            }

            ptr::write(
                dst_cell,
                {
                    let mut c = Cell(0);
                    c.set_content_tag(ContentTag::Codepoint);
                    c.set_content_codepoint(cell_val.codepoint());
                    c.set_wide(cell_val.wide());
                    c.set_protected(cell_val.protected());
                    c.set_semantic_content(cell_val.semantic_content());
                    c
                },
            );

            if cell_val.codepoint() == KITTY_GRAPHICS_UNICODE_PLACEHOLDER {
                (*dst_row).set_kitty_virtual_placeholder(true);
            }

            if cell_val.has_grapheme() {
                if let Some((cps_ptr, cps_len)) = (*src_page).lookup_grapheme(cell) {
                    if page.grapheme_count() >= page.grapheme_capacity() {
                        return Err(ReflowError::OutOfSpace);
                    }

                    match page.set_graphemes(dst_row, dst_cell, cps_ptr, cps_len) {
                        Ok(()) => {}
                        Err(_) => {
                            (*dst_cell).set_content_tag(ContentTag::Codepoint);
                            (*dst_cell).set_content_codepoint(0xFFFD);
                        }
                    }
                }
            }

            if cell_val.hyperlink() {
                if let Some(src_id) = (*src_page).lookup_hyperlink(cell) {
                    if page.hyperlink_count() >= page.hyperlink_capacity() {
                        return Err(ReflowError::OutOfSpace);
                    }

                    let src_base = (*src_page).memory;
                    let our_base = page.memory;

                    let src_set: *const RefCountedSet =
                        &(*src_page).hyperlink_set;
                    let our_set: *mut RefCountedSet =
                        &mut page.hyperlink_set;

                    if (*our_set).capacity() > 0 && (*src_set).capacity() > 0 {
                        let rc = (*our_set).ref_count(our_base as *const u8, src_id);
                        let dst_id = if rc > 0 {
                            (*our_set).add_id(our_base, src_id);
                            src_id
                        } else {
                            let entry: HyperlinkPageEntry =
                                (*src_set).get::<HyperlinkPageEntry>(src_base, src_id);
                            match (*our_set).next_id::<HyperlinkPageEntry>(our_base, entry) {
                                Some(id) => id,
                                None => return Err(ReflowError::OutOfSpace),
                            }
                        };

                        match page.set_hyperlink(dst_row, dst_cell, dst_id) {
                            Ok(()) => {}
                            Err(_) => {
                                if (*our_set).capacity() > 0 {
                                    (*our_set).release(our_base, dst_id);
                                }
                                (*dst_cell).set_hyperlink(false);
                            }
                        }
                    }
                }
            }

            if cell_val.style_id() != DEFAULT_ID {
                let src_base = (*src_page).memory;
                let our_base = page.memory;
                let src_styles: *const RefCountedSet =
                    &(*src_page).styles;
                let our_styles: *mut RefCountedSet =
                    &mut page.styles;

                if (*our_styles).capacity() > 0 && (*src_styles).capacity() > 0 {
                    let style: Style =
                        (*src_styles).get::<Style>(src_base as *mut u8, cell_val.style_id());
                    let src_style_id = cell_val.style_id();

                    let result = (*our_styles).add_with_id::<Style, StyleContext>(
                        our_base,
                        style,
                        src_style_id,
                    );

                    match result {
                        Ok(maybe_id) => {
                            let id = match maybe_id {
                                Some(new_id) => new_id,
                                None => src_style_id,
                            };
                            (*dst_row).set_styled(true);
                            (*dst_cell).set_style_id(id);
                        }
                        Err(_) => {
                            return Err(ReflowError::OutOfSpace);
                        }
                    }
                }
            }

            self.cursor_forward();
            Ok(WriteOutcome::Success)
        }
    }

    pub fn move_last_row_to_new_page(
        &mut self,
        list: *mut PageListType,
        cap: PageCapacity,
    ) -> Result<(), ReflowError> {
        unsafe {
            debug_assert!(self.y == (*self.page).size.rows - 1);
            debug_assert!(!self.pending_wrap);

            let old_node = self.node;
            let old_page = self.page;
            let old_row = self.page_row;
            let old_x = self.x;
            let old_total_rows = self.total_rows;

            self.cursor_new_page(list, cap)?;
            debug_assert!(self.node != old_node);
            debug_assert!(self.y == 0);

            self.cursor_absolute(old_x, 0);

            match (*self.page).clone_row_from(
                old_page as *const Page,
                self.page_row,
                old_row as *const Row,
            ) {
                Ok(()) => {}
                Err(_) => return Err(ReflowError::OutOfMemory),
            }
            self.total_rows = old_total_rows;

            {
                let (keys, len) = tracked_pins_slice(list as *const PageListType);
                if !keys.is_null() {
                    let old_page_ref: *const Page = &(*old_node).data;
                    let old_last_y = (*old_page).size.rows - 1;
                    let mut i = 0usize;
                    while i < len {
                        let p = *keys.add(i);
                        if !p.is_null() {
                            let pin = &mut *p;
                            let pin_page: *const Page = &(*pin.node).data;
                            if pin_page == old_page_ref && pin.y == old_last_y {
                                pin.node = self.node;
                                pin.y = self.y;
                            }
                        }
                        i += 1;
                    }
                }
            }

            (*old_page).clear_cells(old_row, 0, (*old_page).size.cols as usize);
            (*old_page).size.rows -= 1;

            if (*old_page).size.rows == 0 {
                (*list).pages.remove(old_node);
                destroy_node(list, old_node);
            }

            Ok(())
        }
    }

    pub fn reflow_row(
        &mut self,
        list: *mut PageListType,
        row: Pin,
        cursor_pin: Option<*mut Pin>,
    ) -> Result<(), ReflowError> {
        unsafe {
            let src_node = row.node;
            let src_page: *mut Page = &mut (*src_node).data;
            let src_page_const: *const Page = src_page as *const Page;
            let src_y = row.y;

            let src_rows_base: *mut Row = (*src_page).rows.ptr_mut((*src_page).memory);
            let src_row: *mut Row = src_rows_base.add(src_y as usize);
            let src_cells_offset = (*src_row).cells();
            let src_cells: *const Cell = src_cells_offset.ptr((*src_page).memory as *const u8);
            let src_cols = (*src_page).size.cols as usize;

            let mut cols_len = src_cols;
            if !(*src_row).wrap() {
                while cols_len > 0 {
                    if !(*src_cells.add(cols_len - 1)).is_empty() {
                        break;
                    }
                    cols_len -= 1;
                }
                if cols_len == 0 && (*src_row).semantic_prompt() != SemanticPrompt::None {
                    cols_len = 1;
                }
            }

            {
                let (keys, len) = tracked_pins_slice(list as *const PageListType);
                if !keys.is_null() {
                    let src_page_ref_val: *const Page = &(*src_node).data;
                    let mut i = 0usize;
                    while i < len {
                        let p = *keys.add(i);
                        if !p.is_null() {
                            let pin = &mut *p;
                            let pin_page: *const Page = &(*pin.node).data;
                            if pin_page != src_page_ref_val || pin.y != src_y {
                                i += 1;
                                continue;
                            }

                            if let Some(cp) = cursor_pin {
                                if p == cp {
                                    i += 1;
                                    continue;
                                }
                            }

                            if pin.x as usize >= cols_len {
                                let available = (*self.page).size.cols as usize - 1 - self.x as usize;
                                let clamped = if (pin.x as usize) < available {
                                    pin.x
                                } else {
                                    available as CellCountInt
                                };
                                pin.x = clamped;
                            }

                            let pin_extent = pin.x as usize + 1;
                            if pin_extent > cols_len {
                                cols_len = pin_extent;
                            }
                        }
                        i += 1;
                    }
                }
            }

            if let Some(cp) = cursor_pin {
                let pin = &*cp;
                let pin_page: *const Page = &(*pin.node).data;
                let src_page_ref_val: *const Page = &(*src_node).data;
                if pin_page == src_page_ref_val && pin.y == src_y {
                    let pin_extent = pin.x as usize + 1;
                    if pin_extent > cols_len {
                        cols_len = pin_extent;
                    }
                }
            }

            if cols_len == 0 {
                if !(*src_row).wrap_continuation() {
                    self.new_rows += 1;
                }
                return Ok(());
            }

            let cap = match (*src_page).capacity.adjust(&CapacityAdjustment {
                cols: Some((*self.page).size.cols),
            }) {
                Ok(c) => c,
                Err(_) => {
                    let mut fallback = (*src_page).capacity;
                    fallback.cols = (*self.page).size.cols;
                    let std_rows = std_capacity().rows;
                    fallback.rows = if (*src_page).size.rows < std_rows {
                        (*src_page).size.rows
                    } else {
                        std_rows
                    };
                    fallback
                }
            };

            while self.new_rows > 0 {
                self.cursor_scroll_or_new_page(list, cap)?;
                self.new_rows -= 1;
            }

            self.copy_row_metadata(src_row as *const Row);

            let mut x: usize = 0;
            while x < cols_len {
                if self.pending_wrap {
                    (*self.page_row).set_wrap(true);
                    self.cursor_scroll_or_new_page(list, cap)?;
                    self.copy_row_metadata(src_row as *const Row);
                    (*self.page_row).set_wrap_continuation(true);
                }

                {
                    let (keys, len) = tracked_pins_slice(list as *const PageListType);
                    if !keys.is_null() {
                        let src_page_ref_val: *const Page = &(*src_node).data;
                        let mut i = 0usize;
                        while i < len {
                            let p = *keys.add(i);
                            if !p.is_null() {
                                let pin = &mut *p;
                                let pin_page: *const Page = &(*pin.node).data;
                                if pin_page == src_page_ref_val
                                    && pin.y == src_y
                                    && pin.x as usize == x
                                {
                                    pin.node = self.node;
                                    pin.x = self.x;
                                    pin.y = self.y;
                                }
                            }
                            i += 1;
                        }
                    }
                }

                let src_cell = src_cells.add(x);
                match self.write_cell(list, src_cell, src_page_const) {
                    Ok(WriteOutcome::Success) => {
                        x += 1;
                    }
                    Ok(WriteOutcome::SkipNext) => {
                        {
                            let (keys, len) = tracked_pins_slice(
                                list as *const PageListType,
                            );
                            if !keys.is_null() {
                                let src_page_ref_val: *const Page = &(*src_node).data;
                                let mut i = 0usize;
                                while i < len {
                                    let p = *keys.add(i);
                                    if !p.is_null() {
                                        let pin = &mut *p;
                                        let pin_page: *const Page = &(*pin.node).data;
                                        if pin_page == src_page_ref_val
                                            && pin.y == src_y
                                            && pin.x as usize == x + 1
                                        {
                                            pin.node = self.node;
                                            pin.x = self.x;
                                            pin.y = self.y;
                                        }
                                    }
                                    i += 1;
                                }
                            }
                        }
                        x += 2;
                    }
                    Ok(WriteOutcome::Repeat) => {}
                    Err(ReflowError::OutOfMemory) => return Err(ReflowError::OutOfMemory),
                    Err(ReflowError::OutOfSpace) => {
                        if self.y == 0 {
                            x += 1;
                            self.cursor_forward();
                        } else {
                            self.move_last_row_to_new_page(list, cap)?;
                        }
                    }
                }
            }

            if !(*src_row).wrap() {
                self.new_rows += 1;
            }
            Ok(())
        }
    }
}

unsafe fn create_page_node(
    list: *mut PageListType,
    cap: PageCapacity,
) -> Result<*mut PageListNode, ReflowError> {
    unsafe {
        let page = match Page::init(cap) {
            Ok(p) => p,
            Err(_) => return Err(ReflowError::OutOfMemory),
        };
        let node_size = mem::size_of::<PageListNode>();
        let aligned = (node_size + 7) & !7;
        match page_alloc(aligned) {
            Ok(node_mem) => {
                let node_ptr = node_mem.as_mut_ptr() as *mut PageListNode;
                ptr::write(
                    node_ptr,
                    PageListNode {
                        prev: ptr::null_mut(),
                        next: ptr::null_mut(),
                        data: page,
                        serial: (*list).page_serial,
                    },
                );
                (*list).page_serial += 1;
                (*list).page_size += (*node_ptr).data.memory_len;
                Ok(node_ptr)
            }
            Err(_) => return Err(ReflowError::OutOfMemory),
        }
    }
}

unsafe fn destroy_node(list: *mut PageListType, node: *mut PageListNode) {
    unsafe {
        let node_page: *mut Page = &mut (*node).data;
        let page_mem_len = (*node_page).memory_len;
        (*list).page_size = (*list).page_size.saturating_sub(page_mem_len);
        (*node_page).deinit();

        let node_size = mem::size_of::<PageListNode>();
        let aligned = (node_size + 7) & !7;
        let node_slice = core::slice::from_raw_parts_mut(node as *mut u8, aligned);
        page_free(node_slice);
    }
}
