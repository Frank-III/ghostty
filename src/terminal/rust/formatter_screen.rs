#![allow(unused)]

use core::ffi::c_void;
use core::ptr;

use crate::charsets::*;
use crate::CellCountInt;
use crate::formatter_types::{
    Format, Options, PinMap, ScreenContent, ScreenExtra, Writer,
};
use crate::highlight::Pin;
use crate::hyperlink::*;
use crate::kitty_key::*;
use crate::page_core::Page;
use crate::page_list_types::*;
use crate::page_types::*;
use crate::point::PointTag;
use crate::screen_types::*;
use crate::selection_types::Selection;
use crate::style_types::Style;

pub trait FormatterWriter {
    fn write_bytes(&mut self, bytes: &[u8]) -> bool;
}

struct WriterAdapter<'a> {
    inner: &'a mut dyn Writer,
}

impl<'a> FormatterWriter for WriterAdapter<'a> {
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.write(bytes) == bytes.len()
    }
}

struct CountingAdapter {
    count: usize,
}

impl FormatterWriter for CountingAdapter {
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        self.count = self.count.wrapping_add(bytes.len());
        true
    }
}

fn write_usize<W: FormatterWriter>(w: &mut W, v: usize) -> bool {
    if v == 0 {
        return w.write_bytes(b"0");
    }
    let mut tmp = [0u8; 20];
    let mut i = 20usize;
    let mut val = v;
    while val > 0 && i > 0 {
        i -= 1;
        unsafe {
            *tmp.get_unchecked_mut(i) = b'0' + (val % 10) as u8;
        }
        val /= 10;
    }
    w.write_bytes(crate::bytes_util::subslice(&tmp, i, 20))
}

fn write_u16<W: FormatterWriter>(w: &mut W, v: u16) -> bool {
    write_usize(w, v as usize)
}

fn encode_utf8(cp: u32, buf: &mut [u8; 4]) -> usize {
    if cp < 0x80 {
        buf[0] = cp as u8;
        1
    } else if cp < 0x800 {
        buf[0] = 0xC0 | ((cp >> 6) as u8);
        buf[1] = 0x80 | ((cp & 0x3F) as u8);
        2
    } else if cp < 0x10000 {
        buf[0] = 0xE0 | ((cp >> 12) as u8);
        buf[1] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        buf[2] = 0x80 | ((cp & 0x3F) as u8);
        3
    } else if cp < 0x110000 {
        buf[0] = 0xF0 | ((cp >> 18) as u8);
        buf[1] = 0x80 | (((cp >> 12) & 0x3F) as u8);
        buf[2] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        buf[3] = 0x80 | ((cp & 0x3F) as u8);
        4
    } else {
        0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PageFormatterTrailingState {
    wrap: bool,
}

impl Default for PageFormatterTrailingState {
    fn default() -> Self {
        Self { wrap: false }
    }
}

struct PageFormatter {
    page: *const Page,
    opts: Options,
    start_x: CellCountInt,
    end_x: CellCountInt,
    start_y: CellCountInt,
    end_y: CellCountInt,
    rectangle: bool,
    trailing_state: Option<PageFormatterTrailingState>,
}

impl PageFormatter {
    fn init(page: *const Page, opts: Options) -> Self {
        Self {
            page,
            opts,
            start_x: 0,
            end_x: CellCountInt::MAX,
            start_y: 0,
            end_y: CellCountInt::MAX,
            rectangle: false,
            trailing_state: None,
        }
    }

    fn format_with_state<W: FormatterWriter>(
        &mut self,
        writer: &mut W,
    ) -> Option<PageFormatterTrailingState> {
        if self.page.is_null() {
            return None;
        }
        let page = unsafe { &*self.page };
        let cols = page.size.cols;
        let rows = page.size.rows;

        let y_start = self.start_y as usize;
        let y_end = if self.end_y == CellCountInt::MAX {
            rows as usize
        } else {
            (self.end_y as usize) + 1
        };
        if y_end > rows as usize {
            return None;
        }

        let x_start = self.start_x as usize;
        let x_end = if self.end_x == CellCountInt::MAX {
            cols as usize
        } else {
            (self.end_x as usize) + 1
        };
        if x_end > cols as usize {
            return None;
        }

        let mut last_wrap = self.trailing_state.map_or(false, |s| s.wrap);
        let rows_ptr: *mut Row = unsafe { page.rows.ptr_mut(page.memory) };
        let mut blank_rows: usize = 0;

        for y in y_start..y_end {
            let row_ptr = unsafe { rows_ptr.add(y) };
            let row = unsafe { &*row_ptr };

            let x_lo = if y == y_start
                || (self.rectangle && x_start < cols as usize)
            {
                x_start
            } else {
                0
            };
            let x_hi = if y == y_end - 1
                || (self.rectangle && x_end <= cols as usize)
            {
                x_end
            } else {
                cols as usize
            };

            let cells_ptr = unsafe { row.cells().ptr(page.memory) } as *const Cell;
            let hi = if x_hi <= cols as usize {
                x_hi
            } else {
                cols as usize
            };

            if x_lo < hi {
                let cells_subset =
                    unsafe { core::slice::from_raw_parts(cells_ptr.add(x_lo), hi - x_lo) };
                if !Cell::has_text_any(cells_subset) {
                    blank_rows += 1;
                    last_wrap = row.wrap();
                    continue;
                }
            } else {
                blank_rows += 1;
                last_wrap = row.wrap();
                continue;
            }

            if blank_rows > 0 {
                let sequence: &[u8] = match self.opts.format {
                    Format::Vt => b"\r\n",
                    _ => b"\n",
                };
                for _ in 0..blank_rows {
                    if !writer.write_bytes(sequence) {
                        return None;
                    }
                }
                blank_rows = 0;
            }

            if y > y_start && !last_wrap {
                match self.opts.format {
                    Format::Vt => {
                        if !writer.write_bytes(b"\r\n") {
                            return None;
                        }
                    }
                    _ => {
                        if !writer.write_bytes(b"\n") {
                            return None;
                        }
                    }
                }
            }

            let format_styled = matches!(self.opts.format, Format::Vt | Format::Html);
            let mut blank_cells: usize = 0;

            for x in x_lo..hi {
                let cell = unsafe { *cells_ptr.add(x) };
                let mut is_blank = false;
                if format_styled && (!cell.is_empty() || cell.has_styling()) {
                    is_blank = false;
                } else if !cell.has_text() {
                    is_blank = true;
                } else if cell.codepoint() == 0x20 && self.opts.trim {
                    is_blank = true;
                }

                if is_blank {
                    blank_cells += 1;
                    continue;
                }

                if blank_cells > 0 {
                    for _ in 0..blank_cells {
                        if !writer.write_bytes(b" ") {
                            return None;
                        }
                    }
                    blank_cells = 0;
                }

                let cp = cell.codepoint();
                let ch = if cp == 0 { 0x20 } else { cp };
                let mut buf = [0u8; 4];
                let n = encode_utf8(ch, &mut buf);
                if n > 0 && !writer.write_bytes(crate::bytes_util::subslice(&buf, 0, n)) {
                    return None;
                }
            }

            last_wrap = row.wrap();
        }

        Some(PageFormatterTrailingState { wrap: last_wrap })
    }
}

pub struct ScreenFormatter {
    pub screen: *const Screen,
    pub opts: Options,
    pub content: ScreenContent,
    pub extra: ScreenExtra,
    pub pin_map: Option<PinMap>,
}

impl ScreenFormatter {
    pub fn init(screen: *const Screen, opts: Options) -> Self {
        Self {
            screen,
            opts,
            content: ScreenContent::ALL,
            extra: ScreenExtra::NONE,
            pin_map: None,
        }
    }

    pub fn format<W: FormatterWriter>(&mut self, writer: &mut W) -> bool {
        if !self.content.is_none {
            let selection_ptr = if self.content.selection.is_null() {
                None
            } else {
                Some(unsafe { &*(self.content.selection as *const Selection) })
            };

            unsafe {
                let screen = &*self.screen;
                let pages = screen.pages as *const PageList;
                if pages.is_null() {
                    return false;
                }
                let mut list_formatter =
                    PageListFormatter::init(pages, self.opts);
                list_formatter.pin_map = self.pin_map;
                if let Some(sel) = selection_ptr {
                    list_formatter.top_left =
                        Some(selection_top_left(sel, screen));
                    list_formatter.bottom_right =
                        Some(selection_bottom_right(sel, screen));
                    list_formatter.rectangle = sel.rectangle;
                }
                if !list_formatter.format(writer) {
                    return false;
                }
                self.pin_map = list_formatter.pin_map;
            }
        }

        match self.opts.format {
            Format::Plain => return true,
            Format::Vt => {
                if !self.extra.is_set() {
                    return true;
                }
            }
            Format::Html => return true,
            _ => return true,
        }

        if self.extra.style {
            let style_data = unsafe { (*self.screen).cursor.style };
            let mut buf = [0u8; 128];
            let len = style_data.fmt_vt(&mut buf);
            if !writer.write_bytes(crate::bytes_util::subslice(&buf, 0, len)) {
                return false;
            }
        }

        if self.extra.hyperlink {
            let link = unsafe { (*self.screen).cursor.hyperlink };
            if !link.is_null() {
                if !writer.write_bytes(b"\x1b]8;;\x1b\\") {
                    return false;
                }
            }
        }

        if self.extra.protection {
            let prot = unsafe { (*self.screen).cursor.protected };
            if prot && !writer.write_bytes(b"\x1b[1\"q") {
                return false;
            }
        }

        if self.extra.kitty_keyboard {
            let flags = unsafe { (*self.screen).kitty_keyboard.current() };
            if flags.value() != KittyKeyFlags::DISABLED.value() {
                if !writer.write_bytes(b"\x1b[=") {
                    return false;
                }
                if !write_u16(writer, flags.value() as u16) {
                    return false;
                }
                if !writer.write_bytes(b";1u") {
                    return false;
                }
            }
        }

        if self.extra.charsets {
            let charset = unsafe { (*self.screen).charset };
            let slots: [CharsetSlot; 4] = [
                CharsetSlot::G0,
                CharsetSlot::G1,
                CharsetSlot::G2,
                CharsetSlot::G3,
            ];
            for slot in slots.iter() {
                let cs = charset.charsets.get(*slot);
                if cs == CharsetId::Utf8 {
                    continue;
                }
                let intermediate: u8 = match slot {
                    CharsetSlot::G0 => b'(',
                    CharsetSlot::G1 => b')',
                    CharsetSlot::G2 => b'*',
                    CharsetSlot::G3 => b'+',
                };
                let final_byte: u8 = match cs {
                    CharsetId::Ascii => b'B',
                    CharsetId::British => b'A',
                    CharsetId::DecSpecial => b'0',
                    _ => continue,
                };
                if !writer.write_bytes(&[0x1b, intermediate, final_byte]) {
                    return false;
                }
            }

            if charset.gl != CharsetSlot::G0 {
                let seq: &[u8] = match charset.gl {
                    CharsetSlot::G1 => b"\x0e",
                    CharsetSlot::G2 => b"\x1bn",
                    CharsetSlot::G3 => b"\x1bo",
                    _ => b"",
                };
                if !seq.is_empty() && !writer.write_bytes(seq) {
                    return false;
                }
            }

            if charset.gr != CharsetSlot::G2 {
                let seq: &[u8] = match charset.gr {
                    CharsetSlot::G1 => b"\x1b~",
                    CharsetSlot::G3 => b"\x1b|",
                    _ => b"",
                };
                if !seq.is_empty() && !writer.write_bytes(seq) {
                    return false;
                }
            }
        }

        if self.extra.cursor {
            let (cy, cx) = unsafe {
                let c = &(*self.screen).cursor;
                (c.y, c.x)
            };
            if !writer.write_bytes(b"\x1b[") {
                return false;
            }
            if !write_usize(writer, (cy + 1) as usize) {
                return false;
            }
            if !writer.write_bytes(b";") {
                return false;
            }
            if !write_usize(writer, (cx + 1) as usize) {
                return false;
            }
            if !writer.write_bytes(b"H") {
                return false;
            }
        }

        if let Some(m) = self.pin_map {
            let mut counter = CountingAdapter { count: 0 };
            let mut extra_fmt = ScreenFormatter {
                screen: self.screen,
                opts: self.opts,
                content: ScreenContent::NONE,
                extra: self.extra,
                pin_map: None,
            };
            extra_fmt.format(&mut counter);
            let pin_ptr: *const c_void = ptr::null();
            let _ = (m.append_fn)(m.ctx, pin_ptr, counter.count);
        }

        true
    }
}

pub struct PageListFormatter {
    pub list: *const PageList,
    pub opts: Options,
    pub top_left: Option<Pin>,
    pub bottom_right: Option<Pin>,
    pub rectangle: bool,
    pub pin_map: Option<PinMap>,
}

impl PageListFormatter {
    pub fn init(list: *const PageList, opts: Options) -> Self {
        Self {
            list,
            opts,
            top_left: None,
            bottom_right: None,
            rectangle: false,
            pin_map: None,
        }
    }

    pub fn format<W: FormatterWriter>(&mut self, writer: &mut W) -> bool {
        if self.list.is_null() {
            return false;
        }
        let list = unsafe { &*self.list };

        let tl: Pin = match self.top_left {
            Some(p) => p,
            None => list.get_top_left(PointTag::SCREEN),
        };
        let br: Pin = match self.bottom_right {
            Some(p) => p,
            None => match list.get_bottom_right(PointTag::SCREEN) {
                Some(p) => p,
                None => return false,
            },
        };

        let mut page_state: Option<PageFormatterTrailingState> = None;
        let mut current_node = tl.node;

        loop {
            if current_node.is_null() {
                break;
            }
            let node = unsafe { &*current_node };

            let start: CellCountInt =
                if current_node == tl.node { tl.y } else { 0 };
            let end: CellCountInt =
                if current_node == br.node { br.y + 1 } else {
                    node.data.size.rows
                };
            if start >= end {
                if current_node == br.node {
                    break;
                }
                current_node = node.next;
                continue;
            }

            let mut formatter =
                PageFormatter::init(&node.data, self.opts);
            formatter.start_y = start;
            formatter.end_y = end - 1;
            formatter.trailing_state = page_state;
            formatter.rectangle = self.rectangle;

            if self.rectangle {
                formatter.start_x = tl.x;
                formatter.end_x = br.x;
            } else {
                if current_node == tl.node {
                    formatter.start_x = tl.x;
                }
                if current_node == br.node {
                    formatter.end_x = br.x;
                }
            }

            page_state = formatter.format_with_state(writer);
            if page_state.is_none() {
                return false;
            }

            if current_node == br.node {
                break;
            }
            current_node = node.next;
        }

        true
    }
}

pub fn screen_formatter_format(
    screen: *const c_void,
    opts: Options,
    content: ScreenContent,
    extra: ScreenExtra,
    pin_map: Option<PinMap>,
    writer: &mut dyn Writer,
) {
    if screen.is_null() {
        return;
    }
    let mut adapter = WriterAdapter { inner: writer };
    let mut f = ScreenFormatter {
        screen: screen as *const Screen,
        opts,
        content,
        extra,
        pin_map,
    };
    f.format(&mut adapter);
}

fn selection_top_left(sel: &Selection, _screen: &Screen) -> Pin {
    let start = sel.start();
    let end = sel.end_pin();
    if start.node == end.node {
        if start.y < end.y || (start.y == end.y && start.x <= end.x) {
            return start;
        }
        return end;
    }
    let mut cur = start.node;
    let mut seen_end = false;
    unsafe {
        while !cur.is_null() {
            if cur == end.node {
                seen_end = true;
                break;
            }
            if cur == start.node {
                return start;
            }
            cur = (*cur).next;
        }
    }
    if seen_end { start } else { end }
}

fn selection_bottom_right(sel: &Selection, _screen: &Screen) -> Pin {
    let start = sel.start();
    let end = sel.end_pin();
    if start.node == end.node {
        if start.y < end.y || (start.y == end.y && start.x <= end.x) {
            return end;
        }
        return start;
    }
    let mut cur = start.node;
    let mut seen_end = false;
    unsafe {
        while !cur.is_null() {
            if cur == end.node {
                seen_end = true;
                break;
            }
            cur = (*cur).next;
        }
    }
    if seen_end { end } else { start }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CountWriter {
        count: usize,
    }

    impl FormatterWriter for CountWriter {
        fn write_bytes(&mut self, bytes: &[u8]) -> bool {
            self.count += bytes.len();
            true
        }
    }

    #[test]
    fn extra_none_is_not_set() {
        assert!(!ScreenExtra::NONE.is_set());
    }

    #[test]
    fn extra_all_is_set() {
        assert!(ScreenExtra::ALL.is_set());
    }

    #[test]
    fn encode_utf8_ascii() {
        let mut buf = [0u8; 4];
        assert_eq!(encode_utf8(0x41, &mut buf), 1);
        assert_eq!(buf[0], b'A');
    }

    #[test]
    fn encode_utf8_2byte() {
        let mut buf = [0u8; 4];
        assert_eq!(encode_utf8(0xE9, &mut buf), 2);
        assert_eq!(buf[0], 0xC3);
        assert_eq!(buf[1], 0xA9);
    }

    #[test]
    fn encode_utf8_3byte() {
        let mut buf = [0u8; 4];
        assert_eq!(encode_utf8(0x4E16, &mut buf), 3);
        assert_eq!(buf[0], 0xE4);
        assert_eq!(buf[1], 0xB8);
        assert_eq!(buf[2], 0x96);
    }

    #[test]
    fn encode_utf8_4byte() {
        let mut buf = [0u8; 4];
        assert_eq!(encode_utf8(0x1F600, &mut buf), 4);
        assert_eq!(buf[0], 0xF0);
        assert_eq!(buf[1], 0x9F);
        assert_eq!(buf[2], 0x98);
        assert_eq!(buf[3], 0x80);
    }

    #[test]
    fn write_u16_values() {
        struct BufW {
            data: [u8; 8],
            n: usize,
        }
        impl FormatterWriter for BufW {
            fn write_bytes(&mut self, bytes: &[u8]) -> bool {
                for b in bytes {
                    self.data[self.n] = *b;
                    self.n += 1;
                }
                true
            }
        }
        let mut w = BufW { data: [0u8; 8], n: 0 };
        assert!(write_u16(&mut w, 12345));
        assert_eq!(&w.data[..w.n], b"12345");
    }

    #[test]
    fn write_usize_zero() {
        struct BufW {
            data: [u8; 8],
            n: usize,
        }
        impl FormatterWriter for BufW {
            fn write_bytes(&mut self, bytes: &[u8]) -> bool {
                for b in bytes {
                    self.data[self.n] = *b;
                    self.n += 1;
                }
                true
            }
        }
        let mut w = BufW { data: [0u8; 8], n: 0 };
        assert!(write_usize(&mut w, 0));
        assert_eq!(&w.data[..w.n], b"0");
    }

    #[test]
    fn page_list_formatter_init_defaults() {
        let fmt = PageListFormatter::init(ptr::null(), Options::PLAIN);
        assert!(fmt.list.is_null());
        assert!(fmt.top_left.is_none());
        assert!(fmt.bottom_right.is_none());
        assert!(!fmt.rectangle);
    }

    #[test]
    fn page_list_format_null_returns_false() {
        let mut fmt = PageListFormatter::init(ptr::null(), Options::PLAIN);
        let mut cw = CountWriter { count: 0 };
        assert!(!fmt.format(&mut cw));
    }

    #[test]
    fn counting_adapter_accumulates() {
        let mut c = CountingAdapter { count: 0 };
        assert!(c.write_bytes(b"hello"));
        assert!(c.write_bytes(b"world"));
        assert_eq!(c.count, 10);
    }
}
