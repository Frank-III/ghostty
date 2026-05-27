use core::ffi::c_void;

use crate::formatter_types::{Format, Options, PinMap, ScreenContent, ScreenExtra, Writer};
use crate::mode_def::{mode_tag_from_index, MODE_COUNT, MODE_ENTRIES};
use crate::terminal_types::Terminal;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Extra {
    pub palette: bool,
    pub modes: bool,
    pub scrolling_region: bool,
    pub tabstops: bool,
    pub pwd: bool,
    pub keyboard: bool,
    pub screen: ScreenExtra,
}

impl Extra {
    pub const NONE: Extra = Extra {
        palette: false,
        modes: false,
        scrolling_region: false,
        tabstops: false,
        pwd: false,
        keyboard: false,
        screen: ScreenExtra::NONE,
    };

    pub const STYLES: Extra = Extra {
        palette: true,
        modes: false,
        scrolling_region: false,
        tabstops: false,
        pwd: false,
        keyboard: false,
        screen: ScreenExtra::STYLES,
    };

    pub const ALL: Extra = Extra {
        palette: true,
        modes: true,
        scrolling_region: true,
        tabstops: true,
        pwd: true,
        keyboard: true,
        screen: ScreenExtra::ALL,
    };
}

#[derive(Clone, Copy)]
pub struct TerminalFormatter {
    pub terminal: *const c_void,
    pub opts: Options,
    pub content: ScreenContent,
    pub extra: Extra,
    pub pin_map: Option<PinMap>,
}

impl TerminalFormatter {
    pub fn init(terminal: *const c_void, opts: Options) -> Self {
        TerminalFormatter {
            terminal,
            opts,
            content: ScreenContent::ALL,
            extra: Extra::STYLES,
            pin_map: None,
        }
    }

    pub fn format(&self, writer: &mut dyn Writer) {
        let term: &Terminal = unsafe { &*self.terminal.cast::<Terminal>() };

        if self.extra.palette {
            self.emit_palette(term, writer);
        }

        if self.opts.format == Format::Vt && self.extra.modes {
            self.emit_modes(term, writer);
        }

        if !self.content.is_none {
            crate::formatter_screen::screen_formatter_format(
                term.screens.active,
                self.opts,
                self.content,
                self.extra.screen,
                self.pin_map,
                writer,
            );
        }

        if self.opts.format == Format::Vt {
            self.emit_post_screen(term, writer);
        }
    }

    fn emit_palette(&self, term: &Terminal, writer: &mut dyn Writer) {
        match self.opts.format {
            Format::Plain => return,
            Format::Vt => {
                let palette = term.colors.palette.current();
                let mut i = 0usize;
                while i < 256 {
                    let rgb = palette[i];
                    writer.write(b"\x1b]4;");
                    write_usize(writer, i);
                    writer.write(b";rgb:");
                    write_hex2(writer, rgb.r);
                    writer.write(b"/");
                    write_hex2(writer, rgb.g);
                    writer.write(b"/");
                    write_hex2(writer, rgb.b);
                    writer.write(b"\x1b\\");
                    i += 1;
                }
            }
            Format::Html => {
                writer.write(b"<style>:root{");
                let palette = term.colors.palette.current();
                let mut i = 0usize;
                while i < 256 {
                    let rgb = palette[i];
                    writer.write(b"--vt-palette-");
                    write_usize(writer, i);
                    writer.write(b": #");
                    write_hex2(writer, rgb.r);
                    write_hex2(writer, rgb.g);
                    write_hex2(writer, rgb.b);
                    writer.write(b";");
                    i += 1;
                }
                writer.write(b"}</style>");
            }
        }

        if let Some(m) = self.pin_map {
            let mut counter = CountingWriter { count: 0 };
            let mut extra = *self;
            extra.content = ScreenContent::NONE;
            extra.pin_map = None;
            extra.extra = Extra::NONE;
            extra.extra.palette = true;
            extra.format(&mut counter);
            let pin: *const c_void = core::ptr::null();
            let _ = (m.append_fn)(m.ctx, pin, counter.count);
        }
    }

    fn emit_modes(&self, term: &Terminal, writer: &mut dyn Writer) {
        for i in 0..MODE_COUNT {
            let entry = &MODE_ENTRIES[i];
            if entry.disabled {
                continue;
            }
            let tag = mode_tag_from_index(i as u8);
            let current = term.modes.get_by_tag(tag);
            if current == entry.default {
                continue;
            }
            if tag.ansi {
                writer.write(b"\x1b[");
            } else {
                writer.write(b"\x1b[?");
            }
            write_u16(writer, tag.value);
            if current {
                writer.write(b"h");
            } else {
                writer.write(b"l");
            }
        }

        if let Some(m) = self.pin_map {
            let mut counter = CountingWriter { count: 0 };
            let mut extra = *self;
            extra.content = ScreenContent::NONE;
            extra.pin_map = None;
            extra.extra = Extra::NONE;
            extra.extra.modes = true;
            extra.format(&mut counter);
            let pin: *const c_void = core::ptr::null();
            let _ = (m.append_fn)(m.ctx, pin, counter.count);
        }
    }

    fn emit_post_screen(&self, term: &Terminal, writer: &mut dyn Writer) {
        if self.extra.scrolling_region {
            let region = &term.scrolling_region;
            if region.top != 0 || region.bottom != term.rows.wrapping_sub(1) {
                writer.write(b"\x1b[");
                write_usize(writer, (region.top + 1) as usize);
                writer.write(b";");
                write_usize(writer, (region.bottom + 1) as usize);
                writer.write(b"r");
            }
            if region.left != 0 || region.right != term.cols.wrapping_sub(1) {
                writer.write(b"\x1b[");
                write_usize(writer, (region.left + 1) as usize);
                writer.write(b";");
                write_usize(writer, (region.right + 1) as usize);
                writer.write(b"s");
            }
        }

        if self.extra.tabstops {
            writer.write(b"\x1b[3g");
            let cols = term.cols as usize;
            let mut col = 0usize;
            while col < cols {
                if term.tabstops.get(col) {
                    writer.write(b"\x1b[");
                    write_usize(writer, col + 1);
                    writer.write(b"G");
                    writer.write(b"\x1bH");
                }
                col += 1;
            }
        }

        if self.extra.keyboard && term.flags.modify_other_keys_2 {
            writer.write(b"\x1b[>4;2m");
        }

        if self.extra.pwd {
            let mut ptr: *const u8 = core::ptr::null();
            let mut len: usize = 0;
            unsafe {
                ghostty_terminal_pwd_items(self.terminal, &mut ptr, &mut len);
            }
            if !ptr.is_null() && len > 0 {
                let items = unsafe { core::slice::from_raw_parts(ptr, len) };
                writer.write(b"\x1b]7;");
                writer.write(items);
                writer.write(b"\x1b\\");
            }
        }

        if let Some(m) = self.pin_map {
            let mut counter = CountingWriter { count: 0 };
            let mut extra = *self;
            extra.content = ScreenContent::NONE;
            extra.pin_map = None;
            extra.extra = Extra::NONE;
            extra.extra.scrolling_region = self.extra.scrolling_region;
            extra.extra.tabstops = self.extra.tabstops;
            extra.extra.keyboard = self.extra.keyboard;
            extra.extra.pwd = self.extra.pwd;
            extra.format(&mut counter);
            let pin: *const c_void = core::ptr::null();
            let _ = (m.append_fn)(m.ctx, pin, counter.count);
        }
    }
}

extern "C" {
    fn ghostty_terminal_pwd_items(
        terminal: *const c_void,
        out_ptr: *mut *const u8,
        out_len: *mut usize,
    );
}



struct CountingWriter {
    count: usize,
}

impl Writer for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> usize {
        self.count = self.count.wrapping_add(bytes.len());
        bytes.len()
    }
}

fn hex_digit(v: u8) -> u8 {
    if v < 10 {
        b'0' + v
    } else {
        b'a' + (v - 10)
    }
}

fn write_hex2(writer: &mut dyn Writer, value: u8) {
    let buf = [hex_digit(value >> 4), hex_digit(value & 0x0f)];
    writer.write(&buf);
}

fn write_usize(writer: &mut dyn Writer, value: usize) {
    let mut tmp = [0u8; 20];
    let len = usize_to_buf(value, &mut tmp);
    writer.write(&tmp[..len]);
}

fn write_u16(writer: &mut dyn Writer, value: u16) {
    write_usize(writer, value as usize);
}

fn usize_to_buf(value: usize, buf: &mut [u8; 20]) -> usize {
    if value == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 20];
    let mut i = 20usize;
    let mut v = value;
    while v > 0 {
        i -= 1;
        tmp[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    let len = 20 - i;
    let mut k = 0usize;
    while k < len {
        buf[k] = tmp[i + k];
        k += 1;
    }
    len
}
