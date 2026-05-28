use core::ffi::c_void;
use core::ptr;

use crate::allocator::{GhosttyAllocator, alloc_alloc_impl, alloc_free_impl};
use crate::early::*;
use crate::highlight::Pin;
use crate::page_list_types::PageList;
use crate::screen_set::ScreenKey;
use crate::screen_set::ScreenSet;
use crate::screen_types::Screen;
use crate::size_types::CellCountInt;
use crate::tabstops::Tabstops;
use crate::terminal_byte_list::{byte_list_from_void, ByteList};
use crate::terminal_types::{Terminal, TerminalOptions, TABSTOP_INTERVAL};

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
                let screen = self.screens[i];
                if !screen.is_null() {
                    let s = screen as *mut Screen;
                    (*s).bootstrap_deinit(alloc);
                    alloc_free_impl(alloc, s as *mut u8, core::mem::size_of::<Screen>());
                    self.screens[i] = ptr::null_mut();
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
                alloc_free_impl(alloc, pages_ptr as *mut u8, core::mem::size_of::<PageList>());
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
                alloc_free_impl(alloc, pages_ptr as *mut u8, core::mem::size_of::<PageList>());
                return None;
            }

            let mut cursor = crate::screen_types::ScreenCursor::default();
            cursor.page_pin = page_pin;

            Some(Screen {
                alloc: core::ptr::read(alloc),
                pages: pages_ptr,
                no_scrollback: max_scrollback == 0,
                cursor,
                saved_cursor: None,
                selection: None,
                charset: crate::screen_types::ScreenCharsetState::default(),
                protected_mode: crate::ansi::ProtectedMode::OFF,
                kitty_keyboard: crate::kitty_key::KittyKeyFlagStack::default(),
                kitty_images: ptr::null_mut(),
                semantic_prompt: crate::screen_types::ScreenSemanticPrompt::default(),
                dirty: crate::screen_types::ScreenDirty::default(),
            })
        }
    }

    pub unsafe fn bootstrap_deinit(&mut self, alloc: *const GhosttyAllocator) {
        unsafe {
            if !self.pages.is_null() {
                (*self.pages).deinit_full(alloc);
                alloc_free_impl(alloc, self.pages as *mut u8, core::mem::size_of::<PageList>());
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

            let mut term = Terminal::init(&TerminalOptions::new(cols, rows));
            term.screens = screens;
            term.tabstops = tabstops;
            term.cols = cols;
            term.rows = rows;
            term.bootstrap_alloc = alloc;
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
