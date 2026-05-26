use core::ffi::{c_int, c_void};
use core::{mem, ptr};

use crate::constants::*;
use crate::palette_copy::*;
use crate::simple::*;
use crate::style::*;
use crate::style_copy::*;
use crate::style_write::*;

#[repr(C)]
pub struct GhosttyRenderStateColors {
    pub(crate) size: usize,
    pub(crate) background: GhosttyColorRgb,
    pub(crate) foreground: GhosttyColorRgb,
    pub(crate) cursor: GhosttyColorRgb,
    pub(crate) cursor_has_value: bool,
    pub(crate) palette: [GhosttyColorRgb; 256],
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_get_color(
    data: c_int,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor_present: bool,
    cursor: GhosttyColorRgb,
    palette: *const GhosttyColorRgb,
    out: *mut c_void,
) -> c_int {
    if out.is_null() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        match data {
            RENDER_DATA_COLOR_BACKGROUND => {
                write_rgb(out, core::ptr::addr_of!(background));
            }
            RENDER_DATA_COLOR_FOREGROUND => {
                write_rgb(out, core::ptr::addr_of!(foreground));
            }
            RENDER_DATA_COLOR_CURSOR => {
                if !cursor_present {
                    return RENDER_RESULT_INVALID_VALUE;
                }
                write_rgb(out, core::ptr::addr_of!(cursor));
            }
            RENDER_DATA_COLOR_CURSOR_HAS_VALUE => {
                ptr::write(out.cast::<bool>(), cursor_present);
            }
            RENDER_DATA_COLOR_PALETTE => {
                return copy_palette(out.cast::<GhosttyColorRgb>(), palette);
            }
            _ => return RENDER_RESULT_INVALID_VALUE,
        }
    }

    RENDER_RESULT_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_render_state_colors_get(
    out_size: usize,
    out: *mut GhosttyRenderStateColors,
    background: GhosttyColorRgb,
    foreground: GhosttyColorRgb,
    cursor_present: bool,
    cursor: GhosttyColorRgb,
    palette: *const GhosttyColorRgb,
) -> c_int {
    if out.is_null() || palette.is_null() || out_size < mem::size_of::<usize>() {
        return RENDER_RESULT_INVALID_VALUE;
    }

    unsafe {
        if struct_sized_field_fits::<GhosttyColorRgb>(
            out_size,
            mem::offset_of!(GhosttyRenderStateColors, background),
        ) {
            write_rgb(
                core::ptr::addr_of_mut!((*out).background).cast::<c_void>(),
                core::ptr::addr_of!(background),
            );
        }

        if struct_sized_field_fits::<GhosttyColorRgb>(
            out_size,
            mem::offset_of!(GhosttyRenderStateColors, foreground),
        ) {
            write_rgb(
                core::ptr::addr_of_mut!((*out).foreground).cast::<c_void>(),
                core::ptr::addr_of!(foreground),
            );
        }

        if cursor_present
            && struct_sized_field_fits::<GhosttyColorRgb>(
                out_size,
                mem::offset_of!(GhosttyRenderStateColors, cursor),
            )
        {
            write_rgb(
                core::ptr::addr_of_mut!((*out).cursor).cast::<c_void>(),
                core::ptr::addr_of!(cursor),
            );
        }

        if struct_sized_field_fits::<bool>(
            out_size,
            mem::offset_of!(GhosttyRenderStateColors, cursor_has_value),
        ) {
            ptr::write(
                core::ptr::addr_of_mut!((*out).cursor_has_value),
                cursor_present,
            );
        }

        let palette_offset = mem::offset_of!(GhosttyRenderStateColors, palette);
        if out_size > palette_offset {
            let mut available = out_size - palette_offset;
            let out_palette = core::ptr::addr_of_mut!((*out).palette).cast::<GhosttyColorRgb>();
            let mut i = 0usize;
            while i < 256 && available >= mem::size_of::<GhosttyColorRgb>() {
                write_rgb(out_palette.add(i).cast::<c_void>(), palette.add(i));
                available -= mem::size_of::<GhosttyColorRgb>();
                i += 1;
            }
        }
    }

    RENDER_RESULT_SUCCESS
}
