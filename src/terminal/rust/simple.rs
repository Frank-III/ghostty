use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::style::*;

pub(crate) fn decimal_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 10 {
        value /= 10;
        len += 1;
    }
    len
}

pub(crate) fn signed_decimal_len(value: i32) -> usize {
    if value < 0 {
        1 + decimal_len((-i64::from(value)) as u64)
    } else {
        decimal_len(value as u64)
    }
}

pub(crate) fn utf8_len(codepoint: u32) -> Option<usize> {
    match codepoint {
        0x0000..=0x007f => Some(1),
        0x0080..=0x07ff => Some(2),
        0x0800..=0xffff => Some(3),
        0x1_0000..=0x10_ffff => Some(4),
        _ => None,
    }
}

pub(crate) unsafe fn write_bytes(out: *mut u8, offset: &mut usize, bytes: &[u8]) {
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = unsafe { ptr::read(bytes.as_ptr().add(i)) };
        unsafe {
            ptr::write(out.add(*offset + i), byte);
        }
        i += 1;
    }
    *offset += bytes.len();
}

pub(crate) unsafe fn write_byte(out: *mut u8, offset: &mut usize, byte: u8) {
    unsafe {
        ptr::write(out.add(*offset), byte);
    }
    *offset += 1;
}

pub(crate) unsafe fn write_decimal(out: *mut u8, offset: &mut usize, mut value: u64) {
    let mut reversed = [0u8; 20];
    let mut len = 0usize;

    loop {
        let digit = (value % 10) as u8;
        unsafe {
            ptr::write(reversed.as_mut_ptr().add(len), b'0' + digit);
        }
        len += 1;
        value /= 10;

        if value == 0 {
            break;
        }
    }

    while len > 0 {
        len -= 1;
        let byte = unsafe { ptr::read(reversed.as_ptr().add(len)) };
        unsafe {
            ptr::write(out.add(*offset), byte);
        }
        *offset += 1;
    }
}

pub(crate) unsafe fn write_signed_decimal(out: *mut u8, offset: &mut usize, value: i32) {
    if value < 0 {
        unsafe {
            write_byte(out, offset, b'-');
            write_decimal(out, offset, (-i64::from(value)) as u64);
        }
    } else {
        unsafe {
            write_decimal(out, offset, value as u64);
        }
    }
}

pub(crate) unsafe fn write_utf8(out: *mut u8, offset: &mut usize, codepoint: u32) {
    if codepoint <= 0x7f {
        unsafe {
            write_byte(out, offset, codepoint as u8);
        }
    } else if codepoint <= 0x7ff {
        unsafe {
            write_byte(out, offset, 0xc0 | ((codepoint >> 6) as u8));
            write_byte(out, offset, 0x80 | ((codepoint & 0x3f) as u8));
        }
    } else if codepoint <= 0xffff {
        unsafe {
            write_byte(out, offset, 0xe0 | ((codepoint >> 12) as u8));
            write_byte(out, offset, 0x80 | (((codepoint >> 6) & 0x3f) as u8));
            write_byte(out, offset, 0x80 | ((codepoint & 0x3f) as u8));
        }
    } else {
        unsafe {
            write_byte(out, offset, 0xf0 | ((codepoint >> 18) as u8));
            write_byte(out, offset, 0x80 | (((codepoint >> 12) & 0x3f) as u8));
            write_byte(out, offset, 0x80 | (((codepoint >> 6) & 0x3f) as u8));
            write_byte(out, offset, 0x80 | ((codepoint & 0x3f) as u8));
        }
    }
}

pub(crate) unsafe fn write_mouse_action_suffix(out: *mut u8, offset: &mut usize, action: c_int) {
    unsafe {
        write_byte(
            out,
            offset,
            if action == MOUSE_ACTION_RELEASE {
                b'm'
            } else {
                b'M'
            },
        );
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyString {
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,
}

pub(crate) fn struct_sized_field_fits<T>(size: usize, offset: usize) -> bool {
    size >= offset.saturating_add(mem::size_of::<T>())
}

pub(crate) unsafe fn write_string(out: *mut c_void, bytes: &'static [u8]) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), bytes.as_ptr());
        ptr::write(core::ptr::addr_of_mut!((*string).len), bytes.len());
    }
}

pub(crate) unsafe fn write_borrowed_string(out: *mut c_void, ptr: *const u8, len: usize) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), ptr);
        ptr::write(core::ptr::addr_of_mut!((*string).len), len);
    }
}
