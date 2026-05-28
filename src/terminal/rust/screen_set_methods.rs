use core::ffi::c_void;
use core::ptr;

use crate::allocator::{alloc_alloc_impl, alloc_free_impl, GhosttyAllocator};
use crate::early::*;
use crate::highlight::Pin;
use crate::kitty_graphics_storage::{ImageStorage, DEFAULT_SCRATCH_CAP};
use crate::page_list_types::PageList;
use crate::screen_methods::pin_row_and_cell;
use crate::screen_set::ScreenKey;
use crate::screen_set::ScreenSet;
use crate::screen_types::Screen;
use crate::size_types::CellCountInt;
use crate::tabstops::Tabstops;
use crate::terminal_byte_list::{byte_list_from_void, ByteList};
use crate::terminal_types::{Terminal, TerminalOptions, TABSTOP_INTERVAL};

unsafe fn kitty_images_init(alloc: *const GhosttyAllocator) -> *mut c_void {
    unsafe {
        let scratch = alloc_alloc_impl(alloc, DEFAULT_SCRATCH_CAP) as *mut u8;
        if scratch.is_null() {
            return ptr::null_mut();
        }
        let storage_ptr =
            alloc_alloc_impl(alloc, core::mem::size_of::<ImageStorage>()) as *mut ImageStorage;
        if storage_ptr.is_null() {
            alloc_free_impl(alloc, scratch, DEFAULT_SCRATCH_CAP);
            return ptr::null_mut();
        }
        ptr::write(storage_ptr, ImageStorage::new(scratch, DEFAULT_SCRATCH_CAP));
        storage_ptr as *mut c_void
    }
}

unsafe fn kitty_images_deinit(alloc: *const GhosttyAllocator, storage: *mut c_void) {
    if storage.is_null() {
        return;
    }
    unsafe {
        let storage = storage as *mut ImageStorage;
        let scratch = (*storage).scratch_buf_mut();
        let scratch_cap = (*storage).scratch_cap();
        alloc_free_impl(
            alloc,
            storage as *mut u8,
            core::mem::size_of::<ImageStorage>(),
        );
        if !scratch.is_null() {
            alloc_free_impl(alloc, scratch, scratch_cap);
        }
    }
}

impl ScreenSet {
    pub unsafe fn bootstrap_init(
        alloc: *const GhosttyAllocator,
        cols: CellCountInt,
        rows: CellCountInt,
        max_scrollback: usize,
    ) -> Option<ScreenSet> {
        unsafe {
            let screen = Screen::bootstrap_init(alloc, cols, rows, max_scrollback)?;
            let screen_ptr = alloc_box(alloc, screen)? as *mut c_void;

            let mut set = ScreenSet::default();
            set.set_screen(ScreenKey::Primary, screen_ptr);
            set.set_active(ScreenKey::Primary, screen_ptr);
            Some(set)
        }
    }

    pub unsafe fn bootstrap_deinit(&mut self, alloc: *const GhosttyAllocator) {
        unsafe {
            for i in 0..2 {
                let screen = *self.screens.get_unchecked(i);
                if !screen.is_null() {
                    let s = screen as *mut Screen;
                    (*s).bootstrap_deinit(alloc);
                    alloc_free_impl(alloc, s as *mut u8, core::mem::size_of::<Screen>());
                    *self.screens.get_unchecked_mut(i) = ptr::null_mut();
                }
            }
            self.active = ptr::null_mut();
        }
    }
}

impl Screen {
    pub unsafe fn bootstrap_init(
        alloc: *const GhosttyAllocator,
        cols: CellCountInt,
        rows: CellCountInt,
        max_scrollback: usize,
    ) -> Option<Screen> {
        unsafe {
            let pages = PageList::init_full(alloc, cols, rows, max_scrollback)?;
            let pages_ptr = alloc_box(alloc, pages)?;

            let first = (*pages_ptr).pages.first;
            if first.is_null() {
                alloc_free_impl(
                    alloc,
                    pages_ptr as *mut u8,
                    core::mem::size_of::<PageList>(),
                );
                return None;
            }

            let page_pin = (*pages_ptr).track_pin(Pin {
                node: first,
                y: 0,
                x: 0,
                garbage: false,
            });
            if page_pin.is_null() {
                (*pages_ptr).deinit_full(alloc);
                alloc_free_impl(
                    alloc,
                    pages_ptr as *mut u8,
                    core::mem::size_of::<PageList>(),
                );
                return None;
            }

            let mut cursor = crate::screen_types::ScreenCursor::default();
            cursor.page_pin = page_pin;
            let (row, cell) = pin_row_and_cell(page_pin);
            cursor.page_row = row;
            cursor.page_cell = cell;

            let mut screen = Screen {
                alloc: core::ptr::read(alloc),
                pages: pages_ptr,
                no_scrollback: max_scrollback == 0,
                cursor,
                saved_cursor: None,
                selection: None,
                charset: crate::screen_types::ScreenCharsetState::default(),
                protected_mode: crate::ansi::ProtectedMode::OFF,
                kitty_keyboard: crate::kitty_key::KittyKeyFlagStack::default(),
                kitty_images: kitty_images_init(alloc),
                semantic_prompt: crate::screen_types::ScreenSemanticPrompt::default(),
                dirty: crate::screen_types::ScreenDirty::default(),
            };
            screen.cursor_reload();
            Some(screen)
        }
    }

    pub unsafe fn bootstrap_deinit(&mut self, alloc: *const GhosttyAllocator) {
        unsafe {
            kitty_images_deinit(alloc, self.kitty_images);
            self.kitty_images = ptr::null_mut();
            if !self.pages.is_null() {
                (*self.pages).deinit_full(alloc);
                alloc_free_impl(
                    alloc,
                    self.pages as *mut u8,
                    core::mem::size_of::<PageList>(),
                );
                self.pages = ptr::null_mut();
            }
        }
    }
}

impl Terminal {
    pub unsafe fn init_full(
        alloc: *const GhosttyAllocator,
        cols: CellCountInt,
        rows: CellCountInt,
        max_scrollback: usize,
    ) -> Option<Terminal> {
        unsafe {
            let screens = ScreenSet::bootstrap_init(alloc, cols, rows, max_scrollback)?;
            let tabstops = Tabstops::init(alloc, cols as usize, TABSTOP_INTERVAL as usize)?;

            let opts = TerminalOptions::new(cols, rows);
            let mut term = Terminal::init(&opts);
            term.screens = screens;
            term.tabstops = tabstops;
            term.cols = cols;
            term.rows = rows;
            term.bootstrap_alloc = alloc;
            term.apc_max_bytes = crate::apc::ApcMaxBytes::init_full();
            term.kitty_image_storage_limit = opts.kitty_image_storage_limit;
            term.title = ByteList::create(alloc)
                .map(|p| p as *mut core::ffi::c_void)
                .unwrap_or(core::ptr::null_mut());
            term.pwd = ByteList::create(alloc)
                .map(|p| p as *mut core::ffi::c_void)
                .unwrap_or(core::ptr::null_mut());
            Some(term)
        }
    }

    pub unsafe fn deinit_full(&mut self, alloc: *const GhosttyAllocator) {
        unsafe {
            if !self.title.is_null() {
                ByteList::destroy(alloc, byte_list_from_void(self.title));
                self.title = ptr::null_mut();
            }
            if !self.pwd.is_null() {
                ByteList::destroy(alloc, byte_list_from_void(self.pwd));
                self.pwd = ptr::null_mut();
            }
            self.screens.bootstrap_deinit(alloc);
            self.tabstops.deinit(alloc);
            self.deinit(alloc);
        }
    }
}

unsafe fn alloc_box<T>(alloc: *const GhosttyAllocator, value: T) -> Option<*mut T> {
    unsafe {
        let size = core::mem::size_of::<T>();
        let ptr = alloc_alloc_impl(alloc, size);
        if ptr.is_null() {
            return None;
        }
        (ptr as *mut T).write(value);
        Some(ptr as *mut T)
    }
}
