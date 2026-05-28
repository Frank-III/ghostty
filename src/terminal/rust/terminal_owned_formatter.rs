use core::ffi::{c_int, c_void};
use core::ptr;

use crate::allocator::{alloc_alloc_impl, alloc_free_impl, GhosttyAllocator};
use crate::early::*;
use crate::formatter_terminal::{Extra, TerminalFormatter};
use crate::formatter_types::{Format, Options, ScreenContent, ScreenExtra, Writer};
use crate::selection::GhosttySelection;
use crate::selection_copy::selection_from_ghostty;
use crate::selection_types::Selection;
use crate::terminal_owned::RustTerminalOwned;

#[repr(C)]
pub struct OwnedFormatterTerminalExtra {
    pub size: usize,
    pub palette: bool,
    pub modes: bool,
    pub scrolling_region: bool,
    pub tabstops: bool,
    pub pwd: bool,
    pub keyboard: bool,
    pub screen_size: usize,
    pub screen_cursor: bool,
    pub screen_style: bool,
    pub screen_hyperlink: bool,
    pub screen_protection: bool,
    pub screen_kitty_keyboard: bool,
    pub screen_charsets: bool,
}

#[repr(C)]
pub struct OwnedFormatterOptions {
    pub size: usize,
    pub emit: u8,
    pub unwrap: bool,
    pub trim: bool,
    pub extra: OwnedFormatterTerminalExtra,
    pub selection: *const GhosttySelection,
}

pub struct OwnedFormatter {
    pub formatter: TerminalFormatter,
}

impl OwnedFormatter {
    fn extra_from_c(extra: &OwnedFormatterTerminalExtra) -> Extra {
        let screen = if extra.screen_size >= core::mem::size_of::<OwnedFormatterTerminalExtra>() {
            ScreenExtra {
                cursor: extra.screen_cursor,
                style: extra.screen_style,
                hyperlink: extra.screen_hyperlink,
                protection: extra.screen_protection,
                kitty_keyboard: extra.screen_kitty_keyboard,
                charsets: extra.screen_charsets,
            }
        } else {
            ScreenExtra::NONE
        };
        Extra {
            palette: extra.palette,
            modes: extra.modes,
            scrolling_region: extra.scrolling_region,
            tabstops: extra.tabstops,
            pwd: extra.pwd,
            keyboard: extra.keyboard,
            screen,
        }
    }

    unsafe fn content_from_selection(
        alloc: *const GhosttyAllocator,
        sel: *const GhosttySelection,
    ) -> Result<ScreenContent, c_int> {
        unsafe {
            if sel.is_null() {
                return Ok(ScreenContent::ALL);
            }
            let selection = match selection_from_ghostty(&*sel) {
                Some(s) => s,
                None => return Err(GHOSTTY_INVALID_VALUE),
            };
            let boxed =
                alloc_alloc_impl(alloc, core::mem::size_of::<Selection>()) as *mut Selection;
            if boxed.is_null() {
                return Err(GHOSTTY_OUT_OF_MEMORY);
            }
            ptr::write(boxed, selection);
            Ok(ScreenContent {
                is_none: false,
                selection: boxed as *const c_void,
            })
        }
    }
}

struct FixedBufWriter {
    buf: *mut u8,
    cap: usize,
    pos: usize,
    failed: bool,
}

impl Writer for FixedBufWriter {
    fn write(&mut self, bytes: &[u8]) -> usize {
        if self.failed || self.buf.is_null() {
            self.pos = self.pos.wrapping_add(bytes.len());
            return bytes.len();
        }
        let remain = self.cap.saturating_sub(self.pos);
        if bytes.len() > remain {
            self.failed = true;
            self.pos = self.pos.wrapping_add(bytes.len());
            return bytes.len();
        }
        let mut i = 0usize;
        while i < bytes.len() {
            unsafe {
                ptr::write(self.buf.add(self.pos + i), *bytes.get_unchecked(i));
            }
            i += 1;
        }
        self.pos += bytes.len();
        bytes.len()
    }
}

struct AllocWriter {
    alloc: *const GhosttyAllocator,
    buf: *mut u8,
    len: usize,
    cap: usize,
}

impl AllocWriter {
    unsafe fn to_owned_slice(self) -> Result<(*mut u8, usize), c_int> {
        if self.buf.is_null() && self.len == 0 {
            return Ok((ptr::null_mut(), 0));
        }
        if self.buf.is_null() {
            return Err(GHOSTTY_OUT_OF_MEMORY);
        }
        Ok((self.buf, self.len))
    }
}

impl Writer for AllocWriter {
    fn write(&mut self, bytes: &[u8]) -> usize {
        let need = self.len + bytes.len();
        if need > self.cap {
            let new_cap = need;
            let new_buf = unsafe { alloc_alloc_impl(self.alloc, new_cap) as *mut u8 };
            if new_buf.is_null() {
                return 0;
            }
            if !self.buf.is_null() && self.len > 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(self.buf, new_buf, self.len);
                    alloc_free_impl(self.alloc, self.buf as *mut u8, self.cap);
                }
            }
            self.buf = new_buf;
            self.cap = new_cap;
        }
        let mut i = 0usize;
        while i < bytes.len() {
            unsafe {
                ptr::write(self.buf.add(self.len + i), *bytes.get_unchecked(i));
            }
            i += 1;
        }
        self.len += bytes.len();
        bytes.len()
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_formatter_new(
    handle: *mut c_void,
    alloc: *const GhosttyAllocator,
    opts: *const OwnedFormatterOptions,
) -> *mut OwnedFormatter {
    unsafe {
        if handle.is_null() || opts.is_null() {
            return ptr::null_mut();
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        let opts = ptr::read(opts);
        if opts.size < core::mem::size_of::<OwnedFormatterOptions>() {
            return ptr::null_mut();
        }

        let emit = match opts.emit {
            0 => Format::Plain,
            1 => Format::Vt,
            2 => Format::Html,
            _ => return ptr::null_mut(),
        };

        let mut formatter = TerminalFormatter::init(
            ptr::from_ref(&owned.terminal) as *const c_void,
            Options {
                format: emit,
                unwrap: opts.unwrap,
                trim: opts.trim,
                codepoint_map: crate::formatter_types::CodepointMap::EMPTY,
                background: None,
                foreground: None,
                palette: None,
            },
        );
        formatter.extra = OwnedFormatter::extra_from_c(&opts.extra);

        match OwnedFormatter::content_from_selection(alloc, opts.selection) {
            Ok(content) => formatter.content = content,
            Err(_) => return ptr::null_mut(),
        }

        let mem =
            alloc_alloc_impl(alloc, core::mem::size_of::<OwnedFormatter>()) as *mut OwnedFormatter;
        if mem.is_null() {
            if !opts.selection.is_null() && !formatter.content.is_none {
                let sel_ptr = formatter.content.selection as *mut Selection;
                if !sel_ptr.is_null() {
                    alloc_free_impl(alloc, sel_ptr as *mut u8, core::mem::size_of::<Selection>());
                }
            }
            return ptr::null_mut();
        }
        ptr::write(mem, OwnedFormatter { formatter });
        mem
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_formatter_free(
    alloc: *const GhosttyAllocator,
    fmt: *mut OwnedFormatter,
) {
    unsafe {
        if fmt.is_null() {
            return;
        }
        let f = &*fmt;
        if !f.formatter.content.is_none && !f.formatter.content.selection.is_null() {
            let sel_ptr = f.formatter.content.selection as *mut Selection;
            if !sel_ptr.is_null() {
                alloc_free_impl(alloc, sel_ptr as *mut u8, core::mem::size_of::<Selection>());
            }
        }
        alloc_free_impl(
            alloc,
            fmt as *mut u8,
            core::mem::size_of::<OwnedFormatter>(),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_formatter_format_buf(
    fmt: *mut OwnedFormatter,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe {
        if fmt.is_null() || out_written.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }

        let mut discarding = FixedBufWriter {
            buf: ptr::null_mut(),
            cap: 0,
            pos: 0,
            failed: false,
        };
        if out.is_null() || out_len == 0 {
            (&*fmt).formatter.format(&mut discarding);
            ptr::write(out_written, discarding.pos);
            return GHOSTTY_OUT_OF_SPACE;
        }

        let mut writer = FixedBufWriter {
            buf: out,
            cap: out_len,
            pos: 0,
            failed: false,
        };
        (&*fmt).formatter.format(&mut writer);
        ptr::write(out_written, writer.pos);
        if writer.failed {
            return GHOSTTY_OUT_OF_SPACE;
        }
        GHOSTTY_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_formatter_format_alloc(
    alloc: *const GhosttyAllocator,
    fmt: *mut OwnedFormatter,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        if fmt.is_null() || out_ptr.is_null() || out_len.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        ptr::write(out_ptr, ptr::null_mut());
        ptr::write(out_len, 0);

        let mut writer = AllocWriter {
            alloc,
            buf: ptr::null_mut(),
            len: 0,
            cap: 0,
        };
        (&*fmt).formatter.format(&mut writer);
        match writer.to_owned_slice() {
            Ok((ptr, len)) => {
                ptr::write(out_ptr, ptr);
                ptr::write(out_len, len);
                GHOSTTY_SUCCESS
            }
            Err(code) => code,
        }
    }
}

#[repr(C)]
pub struct OwnedSelectionFormatOptions {
    pub size: usize,
    pub emit: u8,
    pub unwrap: bool,
    pub trim: bool,
    pub selection: *const GhosttySelection,
}

fn owned_formatter_options_from_selection(
    opts: &OwnedSelectionFormatOptions,
) -> OwnedFormatterOptions {
    OwnedFormatterOptions {
        size: core::mem::size_of::<OwnedFormatterOptions>(),
        emit: opts.emit,
        unwrap: opts.unwrap,
        trim: opts.trim,
        extra: OwnedFormatterTerminalExtra {
            size: core::mem::size_of::<OwnedFormatterTerminalExtra>(),
            palette: false,
            modes: false,
            scrolling_region: false,
            tabstops: false,
            pwd: false,
            keyboard: false,
            screen_size: core::mem::size_of::<OwnedFormatterTerminalExtra>(),
            screen_cursor: false,
            screen_style: false,
            screen_hyperlink: false,
            screen_protection: false,
            screen_kitty_keyboard: false,
            screen_charsets: false,
        },
        selection: opts.selection,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_selection_format_buf(
    handle: *mut c_void,
    alloc: *const GhosttyAllocator,
    opts: *const OwnedSelectionFormatOptions,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> c_int {
    unsafe {
        if handle.is_null() || opts.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let opts_read = ptr::read(opts);
        if opts_read.size < core::mem::size_of::<OwnedSelectionFormatOptions>() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        let mut fmt_opts = owned_formatter_options_from_selection(&opts_read);
        let mut active_selection_storage: GhosttySelection;
        if fmt_opts.selection.is_null() {
            let screen = owned.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let sel = match (*screen).selection {
                Some(ref s) => s,
                None => return GHOSTTY_NO_VALUE,
            };
            active_selection_storage = crate::selection_copy::selection_to_ghostty(sel);
            fmt_opts.selection = &mut active_selection_storage as *mut GhosttySelection;
        }
        let fmt = ghostty_rust_terminal_owned_formatter_new(handle, alloc, &fmt_opts);
        if fmt.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let result =
            ghostty_rust_terminal_owned_formatter_format_buf(fmt, out, out_len, out_written);
        ghostty_rust_terminal_owned_formatter_free(alloc, fmt);
        result
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_terminal_owned_selection_format_alloc(
    handle: *mut c_void,
    alloc: *const GhosttyAllocator,
    opts: *const OwnedSelectionFormatOptions,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    unsafe {
        if handle.is_null() || opts.is_null() {
            return GHOSTTY_INVALID_VALUE;
        }
        let owned = &*(handle as *mut RustTerminalOwned);
        let opts_read = ptr::read(opts);
        if opts_read.size < core::mem::size_of::<OwnedSelectionFormatOptions>() {
            return GHOSTTY_INVALID_VALUE;
        }

        let mut fmt_opts = owned_formatter_options_from_selection(&opts_read);
        let mut active_selection_storage: GhosttySelection;
        if fmt_opts.selection.is_null() {
            let screen = owned.terminal.active();
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let sel = match (*screen).selection {
                Some(ref s) => s,
                None => return GHOSTTY_NO_VALUE,
            };
            active_selection_storage = crate::selection_copy::selection_to_ghostty(sel);
            fmt_opts.selection = &mut active_selection_storage as *mut GhosttySelection;
        }

        let fmt = ghostty_rust_terminal_owned_formatter_new(handle, alloc, &fmt_opts);
        if fmt.is_null() {
            return if opts_read.selection.is_null() {
                GHOSTTY_NO_VALUE
            } else {
                GHOSTTY_INVALID_VALUE
            };
        }
        let result =
            ghostty_rust_terminal_owned_formatter_format_alloc(alloc, fmt, out_ptr, out_len);
        ghostty_rust_terminal_owned_formatter_free(alloc, fmt);
        result
    }
}
