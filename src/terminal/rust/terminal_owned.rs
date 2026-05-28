use core::ffi::{c_int, c_void};
use core::mem;
use core::ptr;

use crate::allocator::{GhosttyAllocator, alloc_alloc_impl, alloc_free_impl};
use crate::apc::{ApcMaxBytes, ApcProtocol};
use crate::color_palette::{default_palette, Palette};
use crate::early::*;
use crate::highlight::Pin;
use crate::selection::GhosttySelection;
use crate::selection_types::Selection;
use crate::constants::{
    TERMINAL_DATA_COLOR_BACKGROUND, TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT,
    TERMINAL_DATA_COLOR_CURSOR, TERMINAL_DATA_COLOR_CURSOR_DEFAULT,
    TERMINAL_DATA_COLOR_FOREGROUND, TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT,
    TERMINAL_DATA_COLOR_PALETTE, TERMINAL_DATA_COLOR_PALETTE_DEFAULT,
    TERMINAL_DATA_CURSOR_STYLE, TERMINAL_DATA_KITTY_IMAGE_MEDIUM_FILE,
    TERMINAL_DATA_KITTY_IMAGE_MEDIUM_SHARED_MEM, TERMINAL_DATA_KITTY_IMAGE_MEDIUM_TEMP_FILE,
    TERMINAL_DATA_KITTY_IMAGE_STORAGE_LIMIT, TERMINAL_DATA_PWD, TERMINAL_DATA_SCROLLBAR,
    TERMINAL_DATA_SELECTION, TERMINAL_DATA_TITLE, TERMINAL_OPTION_APC_MAX_BYTES,
    TERMINAL_OPTION_APC_MAX_BYTES_KITTY, TERMINAL_OPTION_COLOR_PALETTE,
    TERMINAL_OPTION_KITTY_IMAGE_MEDIUM_FILE, TERMINAL_OPTION_KITTY_IMAGE_MEDIUM_SHARED_MEM,
    TERMINAL_OPTION_KITTY_IMAGE_MEDIUM_TEMP_FILE, TERMINAL_OPTION_KITTY_IMAGE_STORAGE_LIMIT,
    TERMINAL_OPTION_SELECTION, STYLE_COLOR_NONE, STYLE_COLOR_PALETTE, STYLE_COLOR_RGB,
};
use crate::mode_def::{mode_find_index, ModeTag as ModeTagType};
use crate::style::{
    GhosttyColorRgb, GhosttyStyle, GhosttyStyleColor, GhosttyStyleColorValue,
};
use crate::style_types::{Color, Style, rgb_to_ghostty};
use crate::terminal_get_color::{terminal_get_color_impl, terminal_get_palette_impl};
use crate::terminal_get_kitty_image::terminal_get_kitty_image_impl;
use crate::terminal_get_selection::terminal_get_selection_impl;
use crate::terminal_get_style::{terminal_get_scrollbar_impl, terminal_get_style_impl};
use crate::terminal_set_color::{terminal_set_palette_impl, terminal_set_rgb_impl};
use crate::terminal_set_scalar::{terminal_set_bool_optional_impl, terminal_set_u64_zero_impl, terminal_set_usize_optional_impl};
use crate::terminal_get_string::terminal_get_string_impl;
use crate::terminal_set_string::terminal_set_string_impl;
use crate::point::{Point, PointC, PointTag};
use crate::selection::GhosttyGridRef;
use crate::terminal_get_scalar::{
    terminal_get_scalar_impl, terminal_get_scalar_multi_impl,
};
use crate::terminal_point::{
    terminal_point_from_grid_ref_impl, GhosttyPointCoordinate,
};
use crate::terminal_types::{DynamicRgb, Terminal};

/// Rust-owned terminal state for the C ABI path.
#[repr(C)]
pub struct RustTerminalOwned {
    pub terminal: Terminal,
}

#[cfg(ghostty_vt_terminal_owned)]
impl RustTerminalOwned {
    pub unsafe fn set_wrapper(&mut self, wrapper: *mut c_void) {
        self.terminal.effects_wrapper = wrapper;
    }
}

impl RustTerminalOwned {
    pub unsafe fn new(
        alloc: *const GhosttyAllocator,
        cols: u16,
        rows: u16,
        max_scrollback: usize,
    ) -> Option<*mut Self> {
        unsafe {
            let term = Terminal::init_full(alloc, cols, rows, max_scrollback)?;
            let mut owned = RustTerminalOwned { terminal: term };

            let size = core::mem::size_of::<RustTerminalOwned>();
            let mem = alloc_alloc_impl(alloc, size);
            if mem.is_null() {
                owned.terminal.deinit_full(alloc);
                return None;
            }
            (mem as *mut RustTerminalOwned).write(owned);
            Some(mem as *mut RustTerminalOwned)
        }
    }

    pub unsafe fn write(&mut self, data: &[u8]) {
        self.terminal.write(data);
    }

    pub unsafe fn grid_ref(&self, pt: PointC, out_ref: *mut GhosttyGridRef) -> c_int {
        unsafe {
            let point = Point::from_c(pt);
            let (tag, x, y) = match point {
                Point::Active(c) => (PointTag::ACTIVE, c.x, c.y),
                Point::Viewport(c) => (PointTag::VIEWPORT, c.x, c.y),
                Point::Screen(c) => (PointTag::SCREEN, c.x, c.y),
                Point::History(c) => (PointTag::HISTORY, c.x, c.y),
            };

            let screen = self.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let pages = (*screen).pages;
            if pages.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            let pin = match (*pages).pin(tag, x, y) {
                Some(p) => p,
                None => return GHOSTTY_INVALID_VALUE,
            };

            if out_ref.is_null() {
                return GHOSTTY_SUCCESS;
            }

            ptr::write(
                core::ptr::addr_of_mut!((*out_ref).size),
                mem::size_of::<GhosttyGridRef>(),
            );
            ptr::write(
                core::ptr::addr_of_mut!((*out_ref).node),
                pin.node as *mut c_void,
            );
            ptr::write(core::ptr::addr_of_mut!((*out_ref).x), pin.x);
            ptr::write(core::ptr::addr_of_mut!((*out_ref).y), pin.y);

            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn resize(
        &mut self,
        alloc: *const GhosttyAllocator,
        cols: u16,
        rows: u16,
        cell_width_px: u32,
        cell_height_px: u32,
        out_width_px: *mut u32,
        out_height_px: *mut u32,
    ) -> c_int {
        unsafe {
            if cols == 0 || rows == 0 || out_width_px.is_null() || out_height_px.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            let width =
                (u64::from(cols) * u64::from(cell_width_px)).min(u64::from(u32::MAX)) as u32;
            let height =
                (u64::from(rows) * u64::from(cell_height_px)).min(u64::from(u32::MAX)) as u32;

            ptr::write(out_width_px, width);
            ptr::write(out_height_px, height);

            self.terminal.resize(alloc, cols, rows);
            self.terminal.width_px = width;
            self.terminal.height_px = height;

            GHOSTTY_SUCCESS
        }
    }

    pub fn reset(&mut self) {
        self.terminal.full_reset();
    }

    pub unsafe fn mode_get(&self, tag_backing: u16, out: *mut bool) -> c_int {
        unsafe {
            if out.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let tag = ModeTagType::from_u16(tag_backing);
            if mode_find_index(tag.value, tag.ansi).is_none() {
                return GHOSTTY_INVALID_VALUE;
            }
            ptr::write(out, self.terminal.mode_get(tag));
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn mode_set(&mut self, tag_backing: u16, value: bool) -> c_int {
        let tag = ModeTagType::from_u16(tag_backing);
        if mode_find_index(tag.value, tag.ansi).is_none() {
            return GHOSTTY_INVALID_VALUE;
        }
        self.terminal.mode_set(tag, value);
        GHOSTTY_SUCCESS
    }

    pub unsafe fn set_string(
        &mut self,
        alloc: *const GhosttyAllocator,
        data: c_int,
        value: *const crate::simple::GhosttyString,
    ) -> c_int {
        unsafe {
            let mut ptr: *const u8 = core::ptr::null();
            let mut len: usize = 0;
            if terminal_set_string_impl(value, &mut ptr, &mut len) != GHOSTTY_SUCCESS {
                return GHOSTTY_INVALID_VALUE;
            }
            let slice = core::slice::from_raw_parts(ptr, len);
            let ok = match data {
                TERMINAL_DATA_TITLE => self.terminal.set_title_slice(alloc, slice),
                TERMINAL_DATA_PWD => self.terminal.set_pwd_slice(alloc, slice),
                _ => return GHOSTTY_INVALID_VALUE,
            };
            if ok {
                GHOSTTY_SUCCESS
            } else {
                GHOSTTY_OUT_OF_MEMORY
            }
        }
    }

    pub unsafe fn get_string(&self, data: c_int, out: *mut c_void) -> c_int {
        unsafe {
            let (ptr, len) = match data {
                TERMINAL_DATA_TITLE => match self.terminal.get_title_slice() {
                    Some(s) => (s.as_ptr(), s.len()),
                    None => (crate::constants::EMPTY_UTF8.as_ptr(), 0),
                },
                TERMINAL_DATA_PWD => match self.terminal.get_pwd_slice() {
                    Some(s) => (s.as_ptr(), s.len()),
                    None => (crate::constants::EMPTY_UTF8.as_ptr(), 0),
                },
                _ => return GHOSTTY_INVALID_VALUE,
            };
            terminal_get_string_impl(data, ptr, len, out)
        }
    }

    pub unsafe fn get_style(&self, data: c_int, out: *mut c_void) -> c_int {
        unsafe {
            if data != TERMINAL_DATA_CURSOR_STYLE {
                return GHOSTTY_INVALID_VALUE;
            }
            let screen = self.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let style = (*screen).cursor.style;
            let ghostty = style_to_ghostty_style(&style);
            terminal_get_style_impl(data, &ghostty, out)
        }
    }

    pub unsafe fn get_scrollbar(&self, data: c_int, out: *mut c_void) -> c_int {
        unsafe {
            let screen = self.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let pages = (*screen).pages;
            if pages.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let sb = (*pages).scrollbar();
            terminal_get_scrollbar_impl(
                data,
                sb.total as u64,
                sb.offset as u64,
                sb.len as u64,
                out,
            )
        }
    }

    pub unsafe fn get_color(&self, data: c_int, out: *mut c_void) -> c_int {
        unsafe {
            let (has_value, rgb) = match data {
                TERMINAL_DATA_COLOR_FOREGROUND => color_snapshot(&self.terminal.colors.foreground),
                TERMINAL_DATA_COLOR_BACKGROUND => color_snapshot(&self.terminal.colors.background),
                TERMINAL_DATA_COLOR_CURSOR => color_snapshot(&self.terminal.colors.cursor),
                TERMINAL_DATA_COLOR_FOREGROUND_DEFAULT => {
                    default_color_snapshot(&self.terminal.colors.foreground)
                }
                TERMINAL_DATA_COLOR_BACKGROUND_DEFAULT => {
                    default_color_snapshot(&self.terminal.colors.background)
                }
                TERMINAL_DATA_COLOR_CURSOR_DEFAULT => {
                    default_color_snapshot(&self.terminal.colors.cursor)
                }
                _ => return GHOSTTY_INVALID_VALUE,
            };
            terminal_get_color_impl(
                data,
                has_value,
                rgb.r,
                rgb.g,
                rgb.b,
                out,
            )
        }
    }

    pub unsafe fn set_color(
        &mut self,
        data: c_int,
        value: *const GhosttyColorRgb,
    ) -> c_int {
        unsafe {
            let mut has_value = false;
            let mut rgb = GhosttyColorRgb { r: 0, g: 0, b: 0 };
            if terminal_set_rgb_impl(value, &mut has_value, &mut rgb) != GHOSTTY_SUCCESS {
                return GHOSTTY_INVALID_VALUE;
            }

            let slot = match data {
                TERMINAL_DATA_COLOR_FOREGROUND => Some(&mut self.terminal.colors.foreground),
                TERMINAL_DATA_COLOR_BACKGROUND => Some(&mut self.terminal.colors.background),
                TERMINAL_DATA_COLOR_CURSOR => Some(&mut self.terminal.colors.cursor),
                _ => None,
            };
            let Some(target) = slot else {
                return GHOSTTY_INVALID_VALUE;
            };

            target.set_default(if has_value { Some(rgb) } else { None });
            self.terminal.flags.dirty.palette = true;
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn set_palette(&mut self, value: *const GhosttyColorRgb) -> c_int {
        unsafe {
            let mut has_value = false;
            let mut palette_ptr: *const GhosttyColorRgb = ptr::null();
            if terminal_set_palette_impl(value, &mut has_value, &mut palette_ptr) != GHOSTTY_SUCCESS {
                return GHOSTTY_INVALID_VALUE;
            }

            let palette = if has_value {
                palette_from_c(palette_ptr)
            } else {
                default_palette()
            };
            self.terminal.colors.palette.change_default(palette);
            self.terminal.flags.dirty.palette = true;
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn get_palette(&self, data: c_int, out: *mut c_void) -> c_int {
        unsafe {
            let palette = match data {
                TERMINAL_DATA_COLOR_PALETTE => self.terminal.colors.palette.current(),
                TERMINAL_DATA_COLOR_PALETTE_DEFAULT => self.terminal.colors.palette.original(),
                _ => return GHOSTTY_INVALID_VALUE,
            };
            terminal_get_palette_impl(data, palette.as_ptr(), out)
        }
    }

    pub unsafe fn set_selection(&mut self, value: *const GhosttySelection) -> c_int {
        unsafe {
            let screen = self.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            if value.is_null() {
                (*screen).clear_selection();
                return GHOSTTY_SUCCESS;
            }

            let sel_c = ptr::read(value);
            let Some(start) = grid_ref_to_pin(sel_c.start) else {
                return GHOSTTY_INVALID_VALUE;
            };
            let Some(end) = grid_ref_to_pin(sel_c.end) else {
                return GHOSTTY_INVALID_VALUE;
            };
            let sel = Selection::init(start, end, sel_c.rectangle);
            (*screen).select(Some(sel));
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn get_selection(&self, out: *mut c_void) -> c_int {
        unsafe {
            let screen = self.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            let Some(sel) = (*screen).selection.as_ref() else {
                return terminal_get_selection_impl(
                    TERMINAL_DATA_SELECTION,
                    false,
                    ptr::null(),
                    out,
                );
            };

            let ghostty = selection_to_ghostty(sel);
            terminal_get_selection_impl(TERMINAL_DATA_SELECTION, true, &ghostty, out)
        }
    }

    pub unsafe fn set_apc_max_bytes(&mut self, value: *const usize) -> c_int {
        unsafe {
            let mut has_value = false;
            let mut max_bytes = 0usize;
            if terminal_set_usize_optional_impl(value, &mut has_value, &mut max_bytes) != GHOSTTY_SUCCESS
            {
                return GHOSTTY_INVALID_VALUE;
            }

            if has_value {
                self.terminal.apc_max_bytes = ApcMaxBytes::init_full_with(max_bytes);
            } else {
                self.terminal.apc_max_bytes.set_all(None);
            }
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn set_apc_max_bytes_kitty(&mut self, value: *const usize) -> c_int {
        unsafe {
            let mut has_value = false;
            let mut max_bytes = 0usize;
            if terminal_set_usize_optional_impl(value, &mut has_value, &mut max_bytes) != GHOSTTY_SUCCESS
            {
                return GHOSTTY_INVALID_VALUE;
            }

            if has_value {
                self.terminal.apc_max_bytes.put(ApcProtocol::Kitty, max_bytes);
            } else {
                self.terminal.apc_max_bytes.remove(ApcProtocol::Kitty);
            }
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn set_kitty_image_storage_limit(&mut self, value: *const u64) -> c_int {
        unsafe {
            let mut limit = 0u64;
            if terminal_set_u64_zero_impl(value, &mut limit) != GHOSTTY_SUCCESS {
                return GHOSTTY_INVALID_VALUE;
            }
            self.terminal.kitty_image_storage_limit = limit as usize;
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn set_kitty_image_medium(&mut self, option: c_int, value: *const bool) -> c_int {
        unsafe {
            let mut has_value = false;
            let mut enabled = false;
            if terminal_set_bool_optional_impl(value, &mut has_value, &mut enabled) != GHOSTTY_SUCCESS
            {
                return GHOSTTY_INVALID_VALUE;
            }
            if !has_value {
                return GHOSTTY_SUCCESS;
            }

            match option {
                TERMINAL_OPTION_KITTY_IMAGE_MEDIUM_FILE => {
                    self.terminal.kitty_image_medium_file = enabled;
                }
                TERMINAL_OPTION_KITTY_IMAGE_MEDIUM_TEMP_FILE => {
                    self.terminal.kitty_image_medium_temp_file = enabled;
                }
                TERMINAL_OPTION_KITTY_IMAGE_MEDIUM_SHARED_MEM => {
                    self.terminal.kitty_image_medium_shared_mem = enabled;
                }
                _ => return GHOSTTY_INVALID_VALUE,
            }
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn get_kitty_image(&self, data: c_int, enabled: bool, out: *mut c_void) -> c_int {
        unsafe {
            terminal_get_kitty_image_impl(
                data,
                enabled,
                self.terminal.kitty_image_storage_limit as u64,
                self.terminal.kitty_image_medium_file,
                self.terminal.kitty_image_medium_temp_file,
                self.terminal.kitty_image_medium_shared_mem,
                out,
            )
        }
    }

    pub unsafe fn set_color_override(
        &mut self,
        data: c_int,
        value: *const GhosttyColorRgb,
    ) -> c_int {
        unsafe {
            let mut has_value = false;
            let mut rgb = GhosttyColorRgb { r: 0, g: 0, b: 0 };
            if terminal_set_rgb_impl(value, &mut has_value, &mut rgb) != GHOSTTY_SUCCESS {
                return GHOSTTY_INVALID_VALUE;
            }

            let slot = match data {
                TERMINAL_DATA_COLOR_FOREGROUND => Some(&mut self.terminal.colors.foreground),
                TERMINAL_DATA_COLOR_BACKGROUND => Some(&mut self.terminal.colors.background),
                TERMINAL_DATA_COLOR_CURSOR => Some(&mut self.terminal.colors.cursor),
                _ => None,
            };
            let Some(target) = slot else {
                return GHOSTTY_INVALID_VALUE;
            };

            target.set_override(if has_value { Some(rgb) } else { None });
            self.terminal.flags.dirty.palette = true;
            GHOSTTY_SUCCESS
        }
    }

    pub unsafe fn set_palette_index(&mut self, index: u8, value: *const GhosttyColorRgb) -> c_int {
        unsafe {
            if value.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let rgb = ptr::read(value);
            self.terminal.colors.palette.set(index, rgb);
            self.terminal.flags.dirty.palette = true;
            GHOSTTY_SUCCESS
        }
    }

    unsafe fn scalar_snapshot(&self) -> ScalarSnapshot {
        unsafe {
            let screen = self.terminal.active();
            let (cursor_x, cursor_y, cursor_pending_wrap, kitty_keyboard_flags, total_rows) =
                if screen.is_null() {
                    (0, 0, false, 0, 0)
                } else {
                    let s = &*screen;
                    let pages = &*s.pages;
                    (
                        s.cursor.x,
                        s.cursor.y,
                        s.cursor.pending_wrap,
                        s.kitty_keyboard.current().value(),
                        pages.total_rows,
                    )
                };

            ScalarSnapshot {
                cols: self.terminal.cols,
                rows: self.terminal.rows,
                cursor_x,
                cursor_y,
                cursor_pending_wrap,
                active_screen: self.terminal.active_key() as c_int,
                cursor_visible: self.terminal.mode_get(ModeTagType {
                    value: 25,
                    ansi: false,
                }),
                kitty_keyboard_flags,
                mouse_tracking: self.mouse_tracking(),
                total_rows,
                scrollback_rows: total_rows.saturating_sub(self.terminal.rows as usize),
                width_px: self.terminal.width_px,
                height_px: self.terminal.height_px,
            }
        }
    }

    fn mouse_tracking(&self) -> bool {
        const MOUSE_MODES: [(u16, bool); 4] = [(9, false), (1000, false), (1002, false), (1003, false)];
        MOUSE_MODES
            .iter()
            .any(|&(value, ansi)| self.terminal.mode_get(ModeTagType { value, ansi }))
    }

    pub unsafe fn get_scalar(&self, data: c_int, out: *mut c_void) -> c_int {
        unsafe {
            let s = self.scalar_snapshot();
            terminal_get_scalar_impl(
                data,
                s.cols,
                s.rows,
                s.cursor_x,
                s.cursor_y,
                s.cursor_pending_wrap,
                s.active_screen,
                s.cursor_visible,
                s.kitty_keyboard_flags,
                s.mouse_tracking,
                s.total_rows,
                s.scrollback_rows,
                s.width_px,
                s.height_px,
                out,
            )
        }
    }

    pub unsafe fn get_scalar_multi(
        &self,
        count: usize,
        keys: *const c_int,
        values: *const *mut c_void,
        out_written: *mut usize,
    ) -> c_int {
        unsafe {
            let s = self.scalar_snapshot();
            terminal_get_scalar_multi_impl(
                count,
                keys,
                values,
                out_written,
                s.cols,
                s.rows,
                s.cursor_x,
                s.cursor_y,
                s.cursor_pending_wrap,
                s.active_screen,
                s.cursor_visible,
                s.kitty_keyboard_flags,
                s.mouse_tracking,
                s.total_rows,
                s.scrollback_rows,
                s.width_px,
                s.height_px,
            )
        }
    }

    pub unsafe fn point_from_grid_ref(
        &self,
        ref_: *const GhosttyGridRef,
        tag: PointTag,
        out: *mut GhosttyPointCoordinate,
    ) -> c_int {
        unsafe {
            if ref_.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            let screen = self.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let pages = (*screen).pages;
            if pages.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            let grid = &*ref_;
            if grid.node.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }

            let pin = Pin {
                node: grid.node as *mut crate::page_list_types::PageListNode,
                y: grid.y,
                x: grid.x,
                garbage: false,
            };

            let pt = match (*pages).point_from_pin(tag, pin) {
                Some((x, y)) => GhosttyPointCoordinate { x, y },
                None => {
                    return terminal_point_from_grid_ref_impl(
                        false,
                        GhosttyPointCoordinate { x: 0, y: 0 },
                        out,
                    );
                }
            };

            terminal_point_from_grid_ref_impl(true, pt, out)
        }
    }

    pub unsafe fn destroy(alloc: *const GhosttyAllocator, handle: *mut Self) {
        unsafe {
            if handle.is_null() {
                return;
            }
            let owned = &mut *handle;
            owned.terminal.deinit_full(alloc);
            alloc_free_impl(alloc, handle as *mut u8, core::mem::size_of::<RustTerminalOwned>());
        }
    }
}

struct ScalarSnapshot {
    cols: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_pending_wrap: bool,
    active_screen: c_int,
    cursor_visible: bool,
    kitty_keyboard_flags: u8,
    mouse_tracking: bool,
    total_rows: usize,
    scrollback_rows: usize,
    width_px: u32,
    height_px: u32,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_create(
    alloc: *const GhosttyAllocator,
    cols: u16,
    rows: u16,
    max_scrollback: usize,
) -> *mut c_void {
    unsafe {
        if cols == 0 || rows == 0 || alloc.is_null() {
            return ptr::null_mut();
        }
        match RustTerminalOwned::new(alloc, cols, rows, max_scrollback) {
            Some(handle) => handle as *mut c_void,
            None => ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_destroy(
    alloc: *const GhosttyAllocator,
    handle: *mut c_void,
) {
    unsafe {
        if handle.is_null() {
            return;
        }
        RustTerminalOwned::destroy(alloc, handle as *mut RustTerminalOwned);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_wrapper(
    handle: *mut c_void,
    wrapper: *mut c_void,
) {
    unsafe {
        if handle.is_null() {
            return;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_wrapper(wrapper);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_write(
    handle: *mut c_void,
    ptr: *const u8,
    len: usize,
) {
    unsafe {
        if handle.is_null() || ptr.is_null() || len == 0 {
            return;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        let slice = core::slice::from_raw_parts(ptr, len);
        owned.write(slice);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_grid_ref(
    handle: *mut c_void,
    pt: *const PointC,
    out_ref: *mut GhosttyGridRef,
) -> c_int {
    unsafe {
        if handle.is_null() || pt.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.grid_ref(ptr::read(pt), out_ref)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_resize(
    handle: *mut c_void,
    alloc: *const GhosttyAllocator,
    cols: u16,
    rows: u16,
    cell_width_px: u32,
    cell_height_px: u32,
    out_width_px: *mut u32,
    out_height_px: *mut u32,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.resize(
            alloc,
            cols,
            rows,
            cell_width_px,
            cell_height_px,
            out_width_px,
            out_height_px,
        )
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_reset(handle: *mut c_void) {
    unsafe {
        if handle.is_null() {
            return;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.reset();
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_scalar(
    handle: *mut c_void,
    data: c_int,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_scalar(data, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_scalar_multi(
    handle: *mut c_void,
    count: usize,
    keys: *const c_int,
    values: *const *mut c_void,
    out_written: *mut usize,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_scalar_multi(count, keys, values, out_written)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_point_from_grid_ref(
    handle: *mut c_void,
    ref_: *const GhosttyGridRef,
    tag: u8,
    out: *mut GhosttyPointCoordinate,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.point_from_grid_ref(ref_, PointTag::from_u8(tag), out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_mode_get(
    handle: *mut c_void,
    tag: u16,
    out: *mut bool,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.mode_get(tag, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_mode_set(
    handle: *mut c_void,
    tag: u16,
    value: bool,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.mode_set(tag, value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_string(
    handle: *mut c_void,
    alloc: *const GhosttyAllocator,
    data: c_int,
    value: *const crate::simple::GhosttyString,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_string(alloc, data, value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_string(
    handle: *mut c_void,
    data: c_int,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_string(data, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_style(
    handle: *mut c_void,
    data: c_int,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_style(data, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_scrollbar(
    handle: *mut c_void,
    data: c_int,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_scrollbar(data, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_color(
    handle: *mut c_void,
    data: c_int,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_color(data, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_color(
    handle: *mut c_void,
    data: c_int,
    value: *const GhosttyColorRgb,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_color(data, value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_palette(
    handle: *mut c_void,
    value: *const GhosttyColorRgb,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_palette(value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_palette(
    handle: *mut c_void,
    data: c_int,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_palette(data, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_selection(
    handle: *mut c_void,
    value: *const GhosttySelection,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_selection(value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_selection(
    handle: *mut c_void,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_selection(out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_apc_max_bytes(
    handle: *mut c_void,
    value: *const usize,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_apc_max_bytes(value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_apc_max_bytes_kitty(
    handle: *mut c_void,
    value: *const usize,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_apc_max_bytes_kitty(value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_kitty_image_storage_limit(
    handle: *mut c_void,
    value: *const u64,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_kitty_image_storage_limit(value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_kitty_image_medium(
    handle: *mut c_void,
    option: c_int,
    value: *const bool,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_kitty_image_medium(option, value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_get_kitty_image(
    handle: *mut c_void,
    data: c_int,
    enabled: bool,
    out: *mut c_void,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        owned.get_kitty_image(data, enabled, out)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_color_override(
    handle: *mut c_void,
    data: c_int,
    value: *const GhosttyColorRgb,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_color_override(data, value)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_set_palette_index(
    handle: *mut c_void,
    index: u8,
    value: *const GhosttyColorRgb,
) -> c_int {
    unsafe {
        if handle.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        owned.set_palette_index(index, value)
    }
}

fn palette_from_c(ptr: *const GhosttyColorRgb) -> Palette {
    let mut palette = default_palette();
    let mut i = 0usize;
    while i < 256 {
        unsafe {
            palette[i] = ptr::read(ptr.add(i));
        }
        i += 1;
    }
    palette
}

fn grid_ref_to_pin(grid: GhosttyGridRef) -> Option<Pin> {
    if grid.node.is_null() {
        return None;
    }
    Some(Pin {
        node: grid.node as *mut crate::page_list_types::PageListNode,
        x: grid.x,
        y: grid.y,
        garbage: false,
    })
}

fn grid_ref_from_pin(pin: Pin) -> GhosttyGridRef {
    GhosttyGridRef {
        size: mem::size_of::<GhosttyGridRef>(),
        node: pin.node as *mut c_void,
        x: pin.x,
        y: pin.y,
    }
}

fn selection_to_ghostty(sel: &Selection) -> GhosttySelection {
    GhosttySelection {
        size: mem::size_of::<GhosttySelection>(),
        start: grid_ref_from_pin(sel.start()),
        end: grid_ref_from_pin(sel.end_pin()),
        rectangle: sel.rectangle,
    }
}

fn color_snapshot(slot: &DynamicRgb) -> (bool, GhosttyColorRgb) {
    match slot.get() {
        Some(rgb) => (true, rgb),
        None => (false, GhosttyColorRgb { r: 0, g: 0, b: 0 }),
    }
}

fn default_color_snapshot(slot: &DynamicRgb) -> (bool, GhosttyColorRgb) {
    match slot.default_color() {
        Some(rgb) => (true, rgb),
        None => (false, GhosttyColorRgb { r: 0, g: 0, b: 0 }),
    }
}

fn style_color_to_c(color: Color) -> GhosttyStyleColor {
    match color {
        Color::None => GhosttyStyleColor {
            tag: STYLE_COLOR_NONE,
            value: GhosttyStyleColorValue { padding: 0 },
        },
        Color::Palette(idx) => GhosttyStyleColor {
            tag: STYLE_COLOR_PALETTE,
            value: GhosttyStyleColorValue { palette: idx },
        },
        Color::Rgb(rgb) => GhosttyStyleColor {
            tag: STYLE_COLOR_RGB,
            value: GhosttyStyleColorValue {
                rgb: rgb_to_ghostty(rgb),
            },
        },
    }
}

fn style_to_ghostty_style(style: &Style) -> GhosttyStyle {
    GhosttyStyle {
        size: core::mem::size_of::<GhosttyStyle>(),
        fg_color: style_color_to_c(style.fg_color),
        bg_color: style_color_to_c(style.bg_color),
        underline_color: style_color_to_c(style.underline_color),
        bold: style.flags.bold(),
        italic: style.flags.italic(),
        faint: style.flags.faint(),
        blink: style.flags.blink(),
        inverse: style.flags.inverse(),
        invisible: style.flags.invisible(),
        strikethrough: style.flags.strikethrough(),
        overline: style.flags.overline(),
        underline: if style.flags.underline() { 1 } else { 0 },
    }
}
