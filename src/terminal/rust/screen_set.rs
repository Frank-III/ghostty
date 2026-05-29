use crate::allocator::{alloc_alloc_impl, GhosttyAllocator};
use crate::constants::*;
use crate::early::*;
use crate::screen_types::Screen;
use crate::size_types::CellCountInt;
use core::ffi::c_void;
use core::mem;
use core::ptr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenKey {
    Primary = 0,
    Alternate = 1,
}

pub const SCREEN_KEY_COUNT: usize = 2;

impl ScreenKey {
    pub fn as_index(self) -> usize {
        match self {
            ScreenKey::Primary => 0,
            ScreenKey::Alternate => 1,
        }
    }

    pub fn is_primary(self) -> bool {
        self == ScreenKey::Primary
    }

    pub fn is_alternate(self) -> bool {
        self == ScreenKey::Alternate
    }
}

#[repr(C)]
pub struct ScreenSet {
    pub active_key: ScreenKey,
    pub active: *mut c_void,
    pub screens: [*mut c_void; SCREEN_KEY_COUNT],
    pub generations: [usize; SCREEN_KEY_COUNT],
}

impl Default for ScreenSet {
    fn default() -> Self {
        Self {
            active_key: ScreenKey::Primary,
            active: ptr::null_mut(),
            screens: [ptr::null_mut(); SCREEN_KEY_COUNT],
            generations: [0usize; SCREEN_KEY_COUNT],
        }
    }
}

impl ScreenSet {
    pub fn get(&self, key: ScreenKey) -> *mut c_void {
        self.screens[key.as_index()]
    }

    pub fn is_initialized(&self, key: ScreenKey) -> bool {
        !self.screens[key.as_index()].is_null()
    }

    pub fn set_screen(&mut self, key: ScreenKey, screen: *mut c_void) {
        self.screens[key.as_index()] = screen;
    }

    pub fn generation(&self, key: ScreenKey) -> usize {
        self.generations[key.as_index()]
    }

    pub fn bump_generation(&mut self, key: ScreenKey) {
        let idx = key.as_index();
        self.generations[idx] = self.generations[idx].wrapping_add(1);
    }

    pub fn switch_to(&mut self, key: ScreenKey) {
        self.active_key = key;
        self.active = self.screens[key.as_index()];
        debug_assert!(!self.active.is_null());
    }

    pub fn remove_screen(&mut self, key: ScreenKey) -> *mut c_void {
        debug_assert!(!key.is_primary());
        let idx = key.as_index();
        let screen = self.screens[idx];
        if !screen.is_null() {
            self.screens[idx] = ptr::null_mut();
            self.generations[idx] = self.generations[idx].wrapping_add(1);
        }
        screen
    }

    pub fn set_active(&mut self, key: ScreenKey, screen: *mut c_void) {
        self.active_key = key;
        self.active = screen;
        self.screens[key.as_index()] = screen;
    }

    /// Get the screen for `key`, initializing it if needed.
    ///
    /// Safety: `alloc` must be the terminal bootstrap allocator.
    #[cfg(ghostty_vt_terminal_owned)]
    pub unsafe fn get_or_init_screen(
        &mut self,
        alloc: *const GhosttyAllocator,
        key: ScreenKey,
        cols: CellCountInt,
        rows: CellCountInt,
        primary_scrollback: usize,
    ) -> Option<*mut Screen> {
        unsafe {
            let existing = self.get(key);
            if !existing.is_null() {
                return Some(existing as *mut Screen);
            }
            if alloc.is_null() {
                return None;
            }
            let scrollback = if key.is_primary() {
                primary_scrollback
            } else {
                0
            };
            let screen = Screen::bootstrap_init(alloc, cols, rows, scrollback)?;
            let size = mem::size_of::<Screen>();
            let ptr = alloc_alloc_impl(alloc, size);
            if ptr.is_null() {
                return None;
            }
            (ptr as *mut Screen).write(screen);
            self.set_screen(key, ptr as *mut c_void);
            Some(ptr as *mut Screen)
        }
    }

    /// Non-owned builds rely on Zig to allocate screens; only return existing screens.
    #[cfg(not(ghostty_vt_terminal_owned))]
    pub unsafe fn get_or_init_screen(
        &mut self,
        _alloc: *const GhosttyAllocator,
        key: ScreenKey,
        _cols: CellCountInt,
        _rows: CellCountInt,
        _primary_scrollback: usize,
    ) -> Option<*mut Screen> {
        let existing = self.get(key);
        if existing.is_null() {
            None
        } else {
            Some(existing as *mut Screen)
        }
    }
}
