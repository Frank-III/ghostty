use core::ffi::c_void;
use core::ptr;
use crate::early::*;
use crate::constants::*;

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
}
