use crate::color_palette::Palette;
use crate::formatter_types::{Format, format_styled, Options};
use crate::page_core::Page;
use crate::page_types::*;
use crate::point::Coordinate;
use crate::CellCountInt;
use crate::style::GhosttyColorRgb;
use crate::style_types::{Color, RGB, Style};

pub trait FormatterWriter {
    fn write_bytes(&mut self, bytes: &[u8]) -> bool;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TrailingState {
    pub rows: usize,
    pub cells: usize,
}

impl TrailingState {
    pub const EMPTY: TrailingState = TrailingState { rows: 0, cells: 0 };
}

impl Default for TrailingState {
    fn default() -> Self {
        Self::EMPTY
    }
}

pub struct PageFormatter<'a> {
    page: &'a Page,
    opts: Options,
    start_x: CellCountInt,
    start_y: CellCountInt,
    end_x: Option<CellCountInt>,
    end_y: Option<CellCountInt>,
    rectangle: bool,
    trailing_state: Option<TrailingState>,
}

impl<'a> PageFormatter<'a> {
    pub fn new(page: &'a Page, opts: Options) -> Self {
        Self {
            page,
            opts,
            start_x: 0,
            start_y: 0,
            end_x: None,
            end_y: None,
            rectangle: false,
            trailing_state: None,
        }
    }

    pub fn set_start_x(&mut self, v: CellCountInt) { self.start_x = v; }
    pub fn set_start_y(&mut self, v: CellCountInt) { self.start_y = v; }
    pub fn set_end_x(&mut self, v: Option<CellCountInt>) { self.end_x = v; }
    pub fn set_end_y(&mut self, v: Option<CellCountInt>) { self.end_y = v; }
    pub fn set_rectangle(&mut self, v: bool) { self.rectangle = v; }
    pub fn set_trailing_state(&mut self, v: Option<TrailingState>) { self.trailing_state = v; }
    pub fn opts_mut(&mut self) -> &mut Options { &mut self.opts }
    pub fn opts(&self) -> &Options { &self.opts }

    pub fn format(&self, writer: &mut dyn FormatterWriter) -> bool {
        let _state = self.format_with_state(writer);
        true
    }

    pub fn format_with_state(&self, writer: &mut dyn FormatterWriter) -> TrailingState {
        let mut blank_rows: usize = 0;
        let mut blank_cells: usize = 0;

        if let Some(state) = self.trailing_state {
            if self.start_y == 0 && self.start_x == 0 {
                blank_rows = state.rows;
                blank_cells = state.cells;
            }
        }

        let start_x = self.start_x;
        if start_x >= self.page.size.cols {
            return TrailingState { rows: blank_rows, cells: blank_cells };
        }
        let end_x_unclamped = self.end_x.unwrap_or(self.page.size.cols - 1);
        let mut end_x = end_x_unclamped.min(self.page.size.cols - 1);

        let start_y = self.start_y;
        if start_y >= self.page.size.rows {
            return TrailingState { rows: blank_rows, cells: blank_cells };
        }
        let end_y_unclamped = self.end_y.unwrap_or(self.page.size.rows - 1);
        if start_y > end_y_unclamped {
            return TrailingState { rows: blank_rows, cells: blank_cells };
        }
        let mut end_y = end_y_unclamped.min(self.page.size.rows - 1);

        if self.opts.unwrap && !self.rectangle {
            let final_row = self.page.get_row(end_y as usize);
            let cells = self.page.get_cells(final_row);
            if unsafe { (*cells.as_ptr().add(end_x as usize)).wide() } == Wide::SpacerHead {
                if end_y < self.page.size.rows - 1 {
                    end_y += 1;
                    end_x = 0;
                }
            }
        }

        if start_y == end_y && start_x > end_x {
            return TrailingState { rows: blank_rows, cells: blank_cells };
        }

        if !self.emit_header(writer) {
            return TrailingState { rows: blank_rows, cells: blank_cells };
        }

        let mut style = Style::default();
        let mut current_hyperlink_id: Option<u16> = None;

        let mut y = start_y as usize;
        while y <= end_y as usize {
            let row_ptr = self.page.get_row(y);
            let cells = self.page.get_cells(row_ptr);
            let row_ref = unsafe { &*row_ptr };

            let row_end_x: CellCountInt = if self.rectangle || y == end_y as usize {
                end_x + 1
            } else {
                self.page.size.cols
            };

            let row_start_x: CellCountInt = if start_x > 0
                && (self.rectangle || y == start_y as usize)
            {
                let cell_at_start = unsafe { *cells.as_ptr().add(start_x as usize) };
                match cell_at_start.wide() {
                    Wide::SpacerTail => start_x - 1,
                    Wide::SpacerHead => {
                        y += 1;
                        continue;
                    }
                    _ => start_x,
                }
            } else {
                0
            };

            let subset =
                unsafe { core::slice::from_raw_parts(cells.as_ptr().add(row_start_x as usize), (row_end_x - row_start_x) as usize) };

            if !Cell::has_text_any(subset) {
                blank_rows += 1;
                y += 1;
                continue;
            }

            if blank_rows > 0 {
                if !style.is_default() {
                    self.format_style_close(writer);
                    style = Style::default();
                }

                let newline = match self.opts.format {
                    Format::Plain | Format::Html => b"\n" as &[u8],
                    Format::Vt => b"\r\n" as &[u8],
                };

                let mut i = 0usize;
                while i < blank_rows {
                    if !writer.write_bytes(newline) {
                        return TrailingState { rows: blank_rows, cells: blank_cells };
                    }
                    i += 1;
                }

                blank_rows = 0;
            }

            if !row_ref.wrap() || !self.opts.unwrap {
                blank_rows += 1;
            }

            if !row_ref.wrap_continuation() || !self.opts.unwrap {
                blank_cells = 0;
            }

            let mut ci = 0usize;
            while ci < subset.len() {
                let cell = unsafe { *subset.as_ptr().add(ci) };
                let x = row_start_x + ci as CellCountInt;

                match cell.wide() {
                    Wide::SpacerHead | Wide::SpacerTail => {
                        ci += 1;
                        continue;
                    }
                    _ => {}
                }

                if self.is_cell_blank(cell) {
                    blank_cells += 1;
                    ci += 1;
                    continue;
                }

                if blank_cells > 0 {
                    self.write_spaces(writer, blank_cells);
                    blank_cells = 0;
                }

                if format_styled(self.opts.format) {
                    let cell_style = self.cell_style(cell);

                    if !cell_style.eql(&style) {
                        if !style.is_default() {
                            match self.opts.format {
                                Format::Html => {
                                    self.format_style_close(writer);
                                }
                                Format::Vt => {
                                    if cell_style.is_default() {
                                        self.format_style_close(writer);
                                    }
                                }
                                Format::Plain => {}
                            }
                        }

                        style = cell_style;

                        if !cell_style.is_default() {
                            self.format_style_open(writer, &style);
                        }
                    }
                }

                if self.opts.format == Format::Html {
                    let link_id = if cell.hyperlink() {
                        unsafe { self.page.lookup_hyperlink(&cell as *const Cell) }
                    } else {
                        None
                    };

                    if current_hyperlink_id != link_id {
                        if current_hyperlink_id.is_some() {
                            self.format_hyperlink_close(writer);
                            current_hyperlink_id = None;
                        }

                        if let Some(lid) = link_id {
                            current_hyperlink_id = Some(lid);
                            // TODO: lookup URI from page hyperlink_set and emit
                            self.format_hyperlink_open(writer, b"#");
                        }
                    }
                }

                match cell.content_tag() {
                    ContentTag::Codepoint | ContentTag::CodepointGrapheme => {
                        self.write_cell(writer, cell);
                    }
                    ContentTag::BgColorPalette | ContentTag::BgColorRgb => {
                        writer.write_bytes(b" ");
                    }
                }

                ci += 1;
            }

            y += 1;
        }

        if !style.is_default() {
            self.format_style_close(writer);
        }

        if current_hyperlink_id.is_some() {
            self.format_hyperlink_close(writer);
        }

        self.emit_footer(writer);

        TrailingState { rows: blank_rows, cells: blank_cells }
    }

    fn is_cell_blank(&self, cell: Cell) -> bool {
        if format_styled(self.opts.format) && (!cell.is_empty() || cell.has_styling()) {
            return false;
        }

        if !cell.has_text() {
            return true;
        }

        if cell.codepoint() == b' ' as u32 && self.opts.trim {
            return true;
        }

        false
    }

    fn write_cell(&self, writer: &mut dyn FormatterWriter, cell: Cell) {
        if !cell.has_text() {
            writer.write_bytes(b" ");
            return;
        }

        let cp = cell.content_codepoint();
        self.write_codepoint_with_replacement(writer, cp);

        if cell.content_tag() == ContentTag::CodepointGrapheme {
            let grapheme =
                unsafe { self.page.lookup_grapheme(&cell as *const Cell) };
            if let Some((ptr, len)) = grapheme {
                let mut i = 0usize;
                while i < len {
                    let gcp = unsafe { *ptr.add(i) };
                    self.write_codepoint_with_replacement(writer, gcp);
                    i += 1;
                }
            }
        }
    }

    fn write_codepoint_with_replacement(&self, writer: &mut dyn FormatterWriter, cp: u32) {
        // TODO: codepoint_map support requires Options to carry codepoint map fields
        // (currently absent from formatter_types::Options). When added, search the
        // map in reverse for a matching range and write the replacement codepoint
        // or string before falling back to write_codepoint.
        self.write_codepoint(writer, cp);
    }

    fn write_codepoint(&self, writer: &mut dyn FormatterWriter, cp: u32) {
        match self.opts.format {
            Format::Plain | Format::Vt => {
                let mut buf = [0u8; 4];
                let len = encode_utf8(cp, &mut buf);
                writer.write_bytes(&buf[..len]);
            }
            Format::Html => {
                match cp {
                    0x3C => { writer.write_bytes(b"&lt;"); }
                    0x3E => { writer.write_bytes(b"&gt;"); }
                    0x26 => { writer.write_bytes(b"&amp;"); }
                    0x22 => { writer.write_bytes(b"&quot;"); }
                    0x27 => { writer.write_bytes(b"&#39;"); }
                    _ => {
                        if cp < 0x80 {
                            writer.write_bytes(&[cp as u8]);
                        } else {
                            let mut buf = [0u8; 16];
                            let n = write_numeric_entity(cp, &mut buf);
                            writer.write_bytes(&buf[..n]);
                        }
                    }
                }
            }
        }
    }

    fn cell_style(&self, cell: Cell) -> Style {
        match cell.content_tag() {
            ContentTag::Codepoint | ContentTag::CodepointGrapheme => {
                if !cell.has_styling() {
                    return Style::default();
                }
                // TODO: lookup style from page.styles ref-counted set by cell.style_id()
                // For now returns default since style set isn't ported
                Style::default()
            }
            ContentTag::BgColorPalette => Style {
                bg_color: Color::Palette(cell.content_color_palette()),
                ..Style::default()
            },
            ContentTag::BgColorRgb => {
                let rgb_val = cell.content_color_rgb();
                Style {
                    bg_color: Color::Rgb(RGB::new(rgb_val.r, rgb_val.g, rgb_val.b)),
                    ..Style::default()
                }
            }
        }
    }

    fn format_style_open(&self, writer: &mut dyn FormatterWriter, style: &Style) {
        match self.opts.format {
            Format::Plain => {}
            Format::Vt => {
                let mut buf = [0u8; 256];
                let n = style.fmt_vt(&mut buf);
                writer.write_bytes(&buf[..n]);
            }
            Format::Html => {
                let mut buf = [0u8; 512];
                let mut pos = 0usize;
                let prefix = b"<div style=\"display: inline;";
                buf[pos..pos + prefix.len()].copy_from_slice(prefix);
                pos += prefix.len();
                pos = append_html_style(&mut buf, pos, style, self.palette_ref());
                if pos < buf.len() {
                    buf[pos] = b'"';
                    pos += 1;
                }
                if pos < buf.len() {
                    buf[pos] = b'>';
                    pos += 1;
                }
                writer.write_bytes(&buf[..pos]);
            }
        }
    }

    fn format_style_close(&self, writer: &mut dyn FormatterWriter) {
        match self.opts.format {
            Format::Plain => {}
            Format::Vt => {
                writer.write_bytes(b"\x1b[0m");
            }
            Format::Html => {
                writer.write_bytes(b"</div>");
            }
        }
    }

    fn format_hyperlink_open(&self, writer: &mut dyn FormatterWriter, uri: &[u8]) {
        if self.opts.format != Format::Html {
            return;
        }
        writer.write_bytes(b"<a href=\"");
        let mut i = 0usize;
        while i < uri.len() {
            self.write_codepoint(writer, uri[i] as u32);
            i += 1;
        }
        writer.write_bytes(b"\">");
    }

    fn format_hyperlink_close(&self, writer: &mut dyn FormatterWriter) {
        if self.opts.format == Format::Html {
            writer.write_bytes(b"</a>");
        }
    }

    fn emit_header(&self, writer: &mut dyn FormatterWriter) -> bool {
        match self.opts.format {
            Format::Plain => true,
            Format::Html => {
                let mut buf = [0u8; 1024];
                let mut pos = 0usize;
                let header = b"<div style=\"font-family: monospace; white-space: pre;";
                buf[pos..pos + header.len()].copy_from_slice(header);
                pos += header.len();

                if let Some(bg) = self.opts.background {
                    let s = b"background-color: #";
                    buf[pos..pos + s.len()].copy_from_slice(s);
                    pos += s.len();
                    pos = write_hex_byte(&mut buf, pos, bg.r);
                    pos = write_hex_byte(&mut buf, pos, bg.g);
                    pos = write_hex_byte(&mut buf, pos, bg.b);
                    if pos < buf.len() { buf[pos] = b';'; pos += 1; }
                }
                if let Some(fg) = self.opts.foreground {
                    let s = b"color: #";
                    buf[pos..pos + s.len()].copy_from_slice(s);
                    pos += s.len();
                    pos = write_hex_byte(&mut buf, pos, fg.r);
                    pos = write_hex_byte(&mut buf, pos, fg.g);
                    pos = write_hex_byte(&mut buf, pos, fg.b);
                    if pos < buf.len() { buf[pos] = b';'; pos += 1; }
                }
                if pos + 2 <= buf.len() {
                    buf[pos] = b'"'; pos += 1;
                    buf[pos] = b'>'; pos += 1;
                }
                writer.write_bytes(&buf[..pos])
            }
            Format::Vt => {
                let mut buf = [0u8; 512];
                let mut pos = 0usize;
                if let Some(fg) = self.opts.foreground {
                    pos = write_osc_color(&mut buf, pos, 10, fg.r, fg.g, fg.b);
                }
                if let Some(bg) = self.opts.background {
                    pos = write_osc_color(&mut buf, pos, 11, bg.r, bg.g, bg.b);
                }
                if pos > 0 {
                    writer.write_bytes(&buf[..pos])
                } else {
                    true
                }
            }
        }
    }

    fn emit_footer(&self, writer: &mut dyn FormatterWriter) {
        if self.opts.format == Format::Html {
            writer.write_bytes(b"</div>");
        }
    }

    fn write_spaces(&self, writer: &mut dyn FormatterWriter, count: usize) {
        let buf = [b' '; 64];
        let mut remaining = count;
        while remaining > 0 {
            let chunk = remaining.min(64);
            writer.write_bytes(&buf[..chunk]);
            remaining -= chunk;
        }
    }

    fn palette_ref(&self) -> Option<&[GhosttyColorRgb; 256]> {
        self.opts.palette.map(|p| unsafe { &*p })
    }
}

fn write_hex_byte(buf: &mut [u8], pos: usize, v: u8) -> usize {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    if pos + 2 <= buf.len() {
        buf[pos] = HEX[(v >> 4) as usize];
        buf[pos + 1] = HEX[(v & 0xf) as usize];
    }
    pos + 2
}

fn write_osc_color(buf: &mut [u8], mut pos: usize, cmd: u8, r: u8, g: u8, b: u8) -> usize {
    let prefix = b"\x1b]";
    if pos + prefix.len() <= buf.len() {
        buf[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();
    }
    pos = decimal_to_buf_u8(buf, pos, cmd);
    if pos < buf.len() { buf[pos] = b';'; pos += 1; }
    let rgb_prefix = b"rgb:";
    if pos + rgb_prefix.len() <= buf.len() {
        buf[pos..pos + rgb_prefix.len()].copy_from_slice(rgb_prefix);
        pos += rgb_prefix.len();
    }
    pos = write_hex_byte(buf, pos, r);
    if pos < buf.len() { buf[pos] = b'/'; pos += 1; }
    pos = write_hex_byte(buf, pos, g);
    if pos < buf.len() { buf[pos] = b'/'; pos += 1; }
    pos = write_hex_byte(buf, pos, b);
    let st = b"\x1b\\";
    if pos + st.len() <= buf.len() {
        buf[pos..pos + st.len()].copy_from_slice(st);
        pos += st.len();
    }
    pos
}

fn decimal_to_buf_u8(buf: &mut [u8], mut pos: usize, v: u8) -> usize {
    if v == 0 {
        if pos < buf.len() { buf[pos] = b'0'; }
        return pos + 1;
    }
    let mut tmp = [0u8; 3];
    let mut n = 0usize;
    let mut val = v as u16;
    while val > 0 {
        tmp[n] = b'0' + (val % 10) as u8;
        n += 1;
        val /= 10;
    }
    for i in (0..n).rev() {
        if pos < buf.len() { buf[pos] = tmp[i]; }
        pos += 1;
    }
    pos
}

fn write_numeric_entity(cp: u32, buf: &mut [u8]) -> usize {
    buf[0] = b'&';
    buf[1] = b'#';
    let mut tmp = [0u8; 10];
    let mut n = 0usize;
    let mut val = cp;
    while val > 0 {
        tmp[n] = b'0' + (val % 10) as u8;
        n += 1;
        val /= 10;
    }
    let mut pos = 2usize;
    for i in (0..n).rev() {
        buf[pos] = tmp[i];
        pos += 1;
    }
    buf[pos] = b';';
    pos + 1
}

fn encode_utf8(cp: u32, buf: &mut [u8; 4]) -> usize {
    if cp < 0x80 {
        buf[0] = cp as u8;
        1
    } else if cp < 0x800 {
        buf[0] = 0xC0 | (cp >> 6) as u8;
        buf[1] = 0x80 | (cp & 0x3F) as u8;
        2
    } else if cp < 0x10000 {
        buf[0] = 0xE0 | (cp >> 12) as u8;
        buf[1] = 0x80 | ((cp >> 6) & 0x3F) as u8;
        buf[2] = 0x80 | (cp & 0x3F) as u8;
        3
    } else {
        buf[0] = 0xF0 | (cp >> 18) as u8;
        buf[1] = 0x80 | ((cp >> 12) & 0x3F) as u8;
        buf[2] = 0x80 | ((cp >> 6) & 0x3F) as u8;
        buf[3] = 0x80 | (cp & 0x3F) as u8;
        4
    }
}

fn decode_utf8(s: &[u8]) -> (u32, usize) {
    if s.is_empty() {
        return (0, 1);
    }
    let b0 = s[0];
    if b0 < 0x80 {
        (b0 as u32, 1)
    } else if b0 & 0xE0 == 0xC0 {
        if s.len() < 2 { return (0, 1); }
        let cp = ((b0 as u32 & 0x1F) << 6) | (s[1] as u32 & 0x3F);
        (cp, 2)
    } else if b0 & 0xF0 == 0xE0 {
        if s.len() < 3 { return (0, 1); }
        let cp = ((b0 as u32 & 0x0F) << 12)
            | ((s[1] as u32 & 0x3F) << 6)
            | (s[2] as u32 & 0x3F);
        (cp, 3)
    } else if b0 & 0xF8 == 0xF0 {
        if s.len() < 4 { return (0, 1); }
        let cp = ((b0 as u32 & 0x07) << 18)
            | ((s[1] as u32 & 0x3F) << 12)
            | ((s[2] as u32 & 0x3F) << 6)
            | (s[3] as u32 & 0x3F);
        (cp, 4)
    } else {
        (0, 1)
    }
}

fn append_html_style(
    buf: &mut [u8],
    mut pos: usize,
    style: &Style,
    _palette: Option<&[GhosttyColorRgb; 256]>,
) -> usize {
    if style.flags.bold() {
        let s = b"font-weight: bold;";
        if pos + s.len() <= buf.len() {
            buf[pos..pos + s.len()].copy_from_slice(s);
        }
        pos += s.len();
    }
    if style.flags.italic() {
        let s = b"font-style: italic;";
        if pos + s.len() <= buf.len() {
            buf[pos..pos + s.len()].copy_from_slice(s);
        }
        pos += s.len();
    }
    if style.flags.strikethrough() {
        let s = b"text-decoration: line-through;";
        if pos + s.len() <= buf.len() {
            buf[pos..pos + s.len()].copy_from_slice(s);
        }
        pos += s.len();
    }
    if style.flags.overline() {
        let s = b"text-decoration: overline;";
        if pos + s.len() <= buf.len() {
            buf[pos..pos + s.len()].copy_from_slice(s);
        }
        pos += s.len();
    }

    match style.fg_color {
        Color::None => {}
        Color::Palette(idx) => {
            let s = b"color: var(--vt-palette-";
            if pos + s.len() <= buf.len() {
                buf[pos..pos + s.len()].copy_from_slice(s);
            }
            pos += s.len();
            pos = decimal_to_buf_u8(buf, pos, idx);
            if pos + 2 <= buf.len() {
                buf[pos] = b')';
                pos += 1;
                buf[pos] = b';';
                pos += 1;
            }
        }
        Color::Rgb(rgb) => {
            let s = b"color: rgb(";
            if pos + s.len() <= buf.len() {
                buf[pos..pos + s.len()].copy_from_slice(s);
            }
            pos += s.len();
            pos = decimal_to_buf_u8(buf, pos, rgb.r);
            if pos + 2 <= buf.len() { buf[pos] = b','; pos += 1; buf[pos] = b' '; pos += 1; }
            pos = decimal_to_buf_u8(buf, pos, rgb.g);
            if pos + 2 <= buf.len() { buf[pos] = b','; pos += 1; buf[pos] = b' '; pos += 1; }
            pos = decimal_to_buf_u8(buf, pos, rgb.b);
            if pos + 2 <= buf.len() { buf[pos] = b')'; pos += 1; buf[pos] = b';'; pos += 1; }
        }
    }

    match style.bg_color {
        Color::None => {}
        Color::Palette(idx) => {
            let s = b"background-color: var(--vt-palette-";
            if pos + s.len() <= buf.len() {
                buf[pos..pos + s.len()].copy_from_slice(s);
            }
            pos += s.len();
            pos = decimal_to_buf_u8(buf, pos, idx);
            if pos + 2 <= buf.len() {
                buf[pos] = b')';
                pos += 1;
                buf[pos] = b';';
                pos += 1;
            }
        }
        Color::Rgb(rgb) => {
            let s = b"background-color: rgb(";
            if pos + s.len() <= buf.len() {
                buf[pos..pos + s.len()].copy_from_slice(s);
            }
            pos += s.len();
            pos = decimal_to_buf_u8(buf, pos, rgb.r);
            if pos + 2 <= buf.len() { buf[pos] = b','; pos += 1; buf[pos] = b' '; pos += 1; }
            pos = decimal_to_buf_u8(buf, pos, rgb.g);
            if pos + 2 <= buf.len() { buf[pos] = b','; pos += 1; buf[pos] = b' '; pos += 1; }
            pos = decimal_to_buf_u8(buf, pos, rgb.b);
            if pos + 2 <= buf.len() { buf[pos] = b')'; pos += 1; buf[pos] = b';'; pos += 1; }
        }
    }

    pos
}
