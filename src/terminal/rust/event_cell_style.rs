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
use crate::simple::*;

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

pub(crate) unsafe fn copy_style_color(dst: *mut GhosttyStyleColor, src: *const GhosttyStyleColor) -> c_int {
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

pub(crate) unsafe fn write_scrollbar(out: *mut GhosttyTerminalScrollbar, total: u64, offset: u64, len: u64) {
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyStyleColor {
    pub(crate) tag: c_int,
    pub(crate) value: GhosttyStyleColorValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union GhosttyStyleColorValue {
    palette: u8,
    rgb: GhosttyColorRgb,
    padding: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyColorRgb {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyStyle {
    pub(crate) size: usize,
    pub(crate) fg_color: GhosttyStyleColor,
    pub(crate) bg_color: GhosttyStyleColor,
    pub(crate) underline_color: GhosttyStyleColor,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) faint: bool,
    pub(crate) blink: bool,
    pub(crate) inverse: bool,
    pub(crate) invisible: bool,
    pub(crate) strikethrough: bool,
    pub(crate) overline: bool,
    pub(crate) underline: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_style_default(result: *mut GhosttyStyle) {
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*result).size),
            mem::size_of::<GhosttyStyle>(),
        );
        write_style_color_none(core::ptr::addr_of_mut!((*result).fg_color));
        write_style_color_none(core::ptr::addr_of_mut!((*result).bg_color));
        write_style_color_none(core::ptr::addr_of_mut!((*result).underline_color));
        ptr::write(core::ptr::addr_of_mut!((*result).bold), false);
        ptr::write(core::ptr::addr_of_mut!((*result).italic), false);
        ptr::write(core::ptr::addr_of_mut!((*result).faint), false);
        ptr::write(core::ptr::addr_of_mut!((*result).blink), false);
        ptr::write(core::ptr::addr_of_mut!((*result).inverse), false);
        ptr::write(core::ptr::addr_of_mut!((*result).invisible), false);
        ptr::write(core::ptr::addr_of_mut!((*result).strikethrough), false);
        ptr::write(core::ptr::addr_of_mut!((*result).overline), false);
        ptr::write(core::ptr::addr_of_mut!((*result).underline), 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_style_is_default(style: *const GhosttyStyle) -> bool {
    unsafe {
        ptr::read(core::ptr::addr_of!((*style).size)) == mem::size_of::<GhosttyStyle>()
            && ptr::read(core::ptr::addr_of!((*style).fg_color.tag)) == STYLE_COLOR_NONE
            && ptr::read(core::ptr::addr_of!((*style).bg_color.tag)) == STYLE_COLOR_NONE
            && ptr::read(core::ptr::addr_of!((*style).underline_color.tag)) == STYLE_COLOR_NONE
            && !ptr::read(core::ptr::addr_of!((*style).bold))
            && !ptr::read(core::ptr::addr_of!((*style).italic))
            && !ptr::read(core::ptr::addr_of!((*style).faint))
            && !ptr::read(core::ptr::addr_of!((*style).blink))
            && !ptr::read(core::ptr::addr_of!((*style).inverse))
            && !ptr::read(core::ptr::addr_of!((*style).invisible))
            && !ptr::read(core::ptr::addr_of!((*style).strikethrough))
            && !ptr::read(core::ptr::addr_of!((*style).overline))
            && ptr::read(core::ptr::addr_of!((*style).underline)) == 0
    }
}

pub(crate) unsafe fn write_style_color_none(color: *mut GhosttyStyleColor) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*color).tag), STYLE_COLOR_NONE);
        ptr::write(core::ptr::addr_of_mut!((*color).value.padding), 0);
    }
}
