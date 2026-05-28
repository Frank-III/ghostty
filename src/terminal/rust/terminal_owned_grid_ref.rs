use core::ffi::{c_int, c_void};
use core::ptr;

use crate::allocator::{alloc_alloc_impl, alloc_free_impl, GhosttyAllocator};
use crate::early::*;
use crate::highlight::Pin;
use crate::page_list_types::PageList;
use crate::point::{Point, PointC, PointTag};
use crate::screen_set::ScreenKey;
use crate::screen_types::Screen;
use crate::selection::GhosttyGridRef;
use crate::terminal_owned::RustTerminalOwned;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_grid_ref_track(
    handle: *mut c_void,
    pt: *const PointC,
    out_pin: *mut *mut Pin,
    out_screen_key: *mut u8,
    out_screen_generation: *mut usize,
) -> c_int {
    unsafe {
        if handle.is_null() || pt.is_null() || out_pin.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        let point = Point::from_c(ptr::read(pt));
        let (tag, x, y) = match point {
            Point::Active(c) => (PointTag::ACTIVE, c.x, c.y),
            Point::Viewport(c) => (PointTag::VIEWPORT, c.x, c.y),
            Point::Screen(c) => (PointTag::SCREEN, c.x, c.y),
            Point::History(c) => (PointTag::HISTORY, c.x, c.y),
        };

        let screen = owned.terminal.active();
        if screen.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let pages = (*screen).pages;
        if pages.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let p = match (*pages).pin(tag, x, y) {
            Some(pin) => pin,
            None => return GHOSTTY_INVALID_VALUE,
        };

        let tracked = (*pages).track_pin(p);
        if tracked.is_null() {
            return GHOSTTY_OUT_OF_MEMORY;
        }

        ptr::write(out_pin, tracked);
        (*tracked).garbage = false;
        if !out_screen_key.is_null() {
            ptr::write(out_screen_key, owned.terminal.screens.active_key as u8);
        }
        if !out_screen_generation.is_null() {
            let key = owned.terminal.screens.active_key;
            ptr::write(
                out_screen_generation,
                owned.terminal.screens.generation(key),
            );
        }

        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_tracked_page_list(
    handle: *mut c_void,
    screen_key: u8,
    generation: usize,
    out_pages: *mut *mut PageList,
) -> bool {
    unsafe {
        if handle.is_null() || out_pages.is_null() {
            return false;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        let key = match screen_key {
            0 => ScreenKey::Primary,
            1 => ScreenKey::Alternate,
            _ => return false,
        };
        if owned.terminal.screens.generation(key) != generation {
            ptr::write(out_pages, ptr::null_mut());
            return false;
        }
        let screen = owned.terminal.screens.get(key) as *mut Screen;
        if screen.is_null() {
            ptr::write(out_pages, ptr::null_mut());
            return false;
        }
        let pages = (*screen).pages;
        ptr::write(out_pages, pages);
        !pages.is_null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_tracked_pin_garbage(pin: *mut Pin) -> bool {
    if pin.is_null() {
        return true;
    }
    unsafe { (*pin).garbage }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_tracked_pin_to_grid_ref(
    pin: *mut Pin,
    out_ref: *mut GhosttyGridRef,
) -> c_int {
    unsafe {
        if pin.is_null() || out_ref.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let p = &*pin;
        ptr::write(
            core::ptr::addr_of_mut!((*out_ref).size),
            core::mem::size_of::<GhosttyGridRef>(),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*out_ref).node),
            p.node as *mut c_void,
        );
        ptr::write(core::ptr::addr_of_mut!((*out_ref).x), p.x);
        ptr::write(core::ptr::addr_of_mut!((*out_ref).y), p.y);
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_tracked_pin_point(
    pages: *mut PageList,
    tag: u8,
    pin: *mut Pin,
    out_x: *mut u16,
    out_y: *mut u32,
) -> c_int {
    unsafe {
        if pages.is_null() || pin.is_null() {
            return GHOSTTY_NO_VALUE;
        }
        let p = ptr::read(pin);
        if p.garbage {
            return GHOSTTY_NO_VALUE;
        }
        let pt = match (*pages).point_from_pin(PointTag::from_u8(tag), p) {
            Some((x, y)) => (x, y),
            None => return GHOSTTY_NO_VALUE,
        };
        if !out_x.is_null() {
            ptr::write(out_x, pt.0);
        }
        if !out_y.is_null() {
            ptr::write(out_y, pt.1);
        }
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_page_list_for_screen(
    handle: *mut c_void,
    screen_key: u8,
    out_pages: *mut *mut PageList,
) -> bool {
    unsafe {
        if handle.is_null() || out_pages.is_null() {
            return false;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        let key = match screen_key {
            0 => ScreenKey::Primary,
            1 => ScreenKey::Alternate,
            _ => return false,
        };
        let screen = owned.terminal.screens.get(key) as *mut Screen;
        if screen.is_null() {
            ptr::write(out_pages, ptr::null_mut());
            return false;
        }
        let pages = (*screen).pages;
        ptr::write(out_pages, pages);
        !pages.is_null()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_page_list_untrack_pin(
    pages: *mut PageList,
    pin: *mut Pin,
) {
    unsafe {
        if pages.is_null() || pin.is_null() {
            return;
        }
        (*pages).untrack_pin(pin);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_tracked_grid_ref_set(
    handle: *mut c_void,
    pt: *const PointC,
    _old_pin: *mut Pin,
    out_pin: *mut *mut Pin,
    out_screen_key: *mut u8,
    out_screen_generation: *mut usize,
) -> c_int {
    unsafe {
        if handle.is_null() || pt.is_null() || out_pin.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &mut *(handle as *mut RustTerminalOwned);
        let screen = owned.terminal.active();
        if screen.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let pages = (*screen).pages;
        if pages.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let point = Point::from_c(ptr::read(pt));
        let (tag, x, y) = match point {
            Point::Active(c) => (PointTag::ACTIVE, c.x, c.y),
            Point::Viewport(c) => (PointTag::VIEWPORT, c.x, c.y),
            Point::Screen(c) => (PointTag::SCREEN, c.x, c.y),
            Point::History(c) => (PointTag::HISTORY, c.x, c.y),
        };

        let p = match (*pages).pin(tag, x, y) {
            Some(pin) => pin,
            None => return GHOSTTY_INVALID_VALUE,
        };

        let tracked = (*pages).track_pin(p);
        if tracked.is_null() {
            return GHOSTTY_OUT_OF_MEMORY;
        }

        ptr::write(out_pin, tracked);
        (*tracked).garbage = false;
        if !out_screen_key.is_null() {
            ptr::write(out_screen_key, owned.terminal.screens.active_key as u8);
        }
        if !out_screen_generation.is_null() {
            let key = owned.terminal.screens.active_key;
            ptr::write(
                out_screen_generation,
                owned.terminal.screens.generation(key),
            );
        }

        GHOSTTY_SUCCESS
    }
}
