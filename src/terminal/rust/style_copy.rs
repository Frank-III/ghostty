use core::ffi::{c_int, c_void};
use core::ptr;

use crate::constants::*;
use crate::early::*;
use crate::selection::*;
use crate::style::*;

pub(crate) unsafe fn write_style_color_rgb(
    color: *const GhosttyStyleColor,
    palette_color: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    if color.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        match ptr::read(core::ptr::addr_of!((*color).tag)) {
            STYLE_COLOR_NONE => GHOSTTY_INVALID_VALUE,
            STYLE_COLOR_PALETTE => {
                write_rgb(out, palette_color);
                GHOSTTY_SUCCESS
            }
            STYLE_COLOR_RGB => {
                let rgb = core::ptr::addr_of!((*color).value.rgb);
                write_rgb(out, rgb);
                GHOSTTY_SUCCESS
            }
            _ => GHOSTTY_INVALID_VALUE,
        }
    }
}

pub(crate) unsafe fn copy_style_color(
    dst: *mut GhosttyStyleColor,
    src: *const GhosttyStyleColor,
) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        let tag = ptr::read(core::ptr::addr_of!((*src).tag));
        ptr::write(core::ptr::addr_of_mut!((*dst).tag), tag);
        ptr::write(core::ptr::addr_of_mut!((*dst).value.padding), 0);

        match tag {
            STYLE_COLOR_NONE => {}
            STYLE_COLOR_PALETTE => {
                ptr::write(
                    core::ptr::addr_of_mut!((*dst).value.palette),
                    ptr::read(core::ptr::addr_of!((*src).value.palette)),
                );
            }
            STYLE_COLOR_RGB => {
                write_rgb(
                    core::ptr::addr_of_mut!((*dst).value.rgb).cast::<c_void>(),
                    core::ptr::addr_of!((*src).value.rgb),
                );
            }
            _ => return GHOSTTY_INVALID_VALUE,
        }
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn copy_style(dst: *mut GhosttyStyle, src: *const GhosttyStyle) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*dst).size),
            ptr::read(core::ptr::addr_of!((*src).size)),
        );
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).fg_color),
            core::ptr::addr_of!((*src).fg_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).bg_color),
            core::ptr::addr_of!((*src).bg_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        let result = copy_style_color(
            core::ptr::addr_of_mut!((*dst).underline_color),
            core::ptr::addr_of!((*src).underline_color),
        );
        if result != GHOSTTY_SUCCESS {
            return result;
        }
        ptr::write(
            core::ptr::addr_of_mut!((*dst).bold),
            ptr::read(core::ptr::addr_of!((*src).bold)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).italic),
            ptr::read(core::ptr::addr_of!((*src).italic)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).faint),
            ptr::read(core::ptr::addr_of!((*src).faint)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).blink),
            ptr::read(core::ptr::addr_of!((*src).blink)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).inverse),
            ptr::read(core::ptr::addr_of!((*src).inverse)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).invisible),
            ptr::read(core::ptr::addr_of!((*src).invisible)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).strikethrough),
            ptr::read(core::ptr::addr_of!((*src).strikethrough)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).overline),
            ptr::read(core::ptr::addr_of!((*src).overline)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*dst).underline),
            ptr::read(core::ptr::addr_of!((*src).underline)),
        );
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn write_scrollbar(
    out: *mut GhosttyTerminalScrollbar,
    total: u64,
    offset: u64,
    len: u64,
) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).total), total);
        ptr::write(core::ptr::addr_of_mut!((*out).offset), offset);
        ptr::write(core::ptr::addr_of_mut!((*out).len), len);
    }
}

pub(crate) unsafe fn write_rgb_value(out: *mut GhosttyColorRgb, r: u8, g: u8, b: u8) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).r), r);
        ptr::write(core::ptr::addr_of_mut!((*out).g), g);
        ptr::write(core::ptr::addr_of_mut!((*out).b), b);
    }
}

pub(crate) unsafe fn copy_palette(dst: *mut GhosttyColorRgb, src: *const GhosttyColorRgb) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < 256 {
        unsafe {
            let src_rgb = src.add(i);
            write_rgb_value(
                dst.add(i),
                ptr::read(core::ptr::addr_of!((*src_rgb).r)),
                ptr::read(core::ptr::addr_of!((*src_rgb).g)),
                ptr::read(core::ptr::addr_of!((*src_rgb).b)),
            );
        }
        i += 1;
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn write_rgb(out: *mut c_void, src: *const GhosttyColorRgb) {
    let rgb = out.cast::<GhosttyColorRgb>();
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).r),
            ptr::read(core::ptr::addr_of!((*src).r)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).g),
            ptr::read(core::ptr::addr_of!((*src).g)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).b),
            ptr::read(core::ptr::addr_of!((*src).b)),
        );
    }
}

pub(crate) unsafe fn write_out<T>(out: *mut c_void, value: T) {
    unsafe {
        ptr::write(out.cast::<T>(), value);
    }
}
