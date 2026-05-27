#![allow(unused)]

use crate::stream_types::OscTerminator;
use crate::style_types::RGB;
use crate::kitty_color;
use crate::osc_encoding::is_safe_utf8;

// ─── Kitty Clipboard Protocol (OSC 5522) ───────────────────────────────────

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KittyClipboardLocation {
    Primary = 0,
}

impl KittyClipboardLocation {
    #[inline]
    pub fn parse(s: &[u8]) -> Option<Self> {
        if s.len() == 7
            && unsafe { *s.get_unchecked(0) } == b'p'
            && unsafe { *s.get_unchecked(1) } == b'r'
            && unsafe { *s.get_unchecked(2) } == b'i'
            && unsafe { *s.get_unchecked(3) } == b'm'
            && unsafe { *s.get_unchecked(4) } == b'a'
            && unsafe { *s.get_unchecked(5) } == b'r'
            && unsafe { *s.get_unchecked(6) } == b'y'
        {
            Some(KittyClipboardLocation::Primary)
        } else {
            None
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KittyClipboardOperation {
    Read = 0,
    Walias = 1,
    Wdata = 2,
    Write = 3,
}

impl KittyClipboardOperation {
    #[inline]
    pub fn parse(s: &[u8]) -> Option<Self> {
        match s {
            b"read" => Some(KittyClipboardOperation::Read),
            b"walias" => Some(KittyClipboardOperation::Walias),
            b"wdata" => Some(KittyClipboardOperation::Wdata),
            b"write" => Some(KittyClipboardOperation::Write),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KittyClipboardStatus {
    Data = 0,
    Done = 1,
    Ebusy = 2,
    Einval = 3,
    Eio = 4,
    Enosys = 5,
    Eperm = 6,
    Ok = 7,
}

impl KittyClipboardStatus {
    #[inline]
    pub fn parse(s: &[u8]) -> Option<Self> {
        match s {
            b"DATA" => Some(KittyClipboardStatus::Data),
            b"DONE" => Some(KittyClipboardStatus::Done),
            b"EBUSY" => Some(KittyClipboardStatus::Ebusy),
            b"EINVAL" => Some(KittyClipboardStatus::Einval),
            b"EIO" => Some(KittyClipboardStatus::Eio),
            b"ENOSYS" => Some(KittyClipboardStatus::Enosys),
            b"EPERM" => Some(KittyClipboardStatus::Eperm),
            b"OK" => Some(KittyClipboardStatus::Ok),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KittyClipboardOptionKey {
    Id,
    Loc,
    Mime,
    Name,
    Password,
    Pw,
    Status,
    Type,
}

fn is_ascii_whitespace(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r'
}

fn trim_ws(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && is_ascii_whitespace(unsafe { *s.get_unchecked(start) }) {
        start += 1;
    }
    while end > start && is_ascii_whitespace(unsafe { *s.get_unchecked(end - 1) }) {
        end -= 1;
    }
    unsafe { s.get_unchecked(start..end) }
}

fn find_byte_kitty(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
    let mut i = start;
    while i < haystack.len() {
        if unsafe { *haystack.get_unchecked(i) } == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn bytes_eq_kitty(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if unsafe { *a.get_unchecked(i) } != unsafe { *b.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

fn key_bytes(key: KittyClipboardOptionKey) -> &'static [u8] {
    match key {
        KittyClipboardOptionKey::Id => b"id",
        KittyClipboardOptionKey::Loc => b"loc",
        KittyClipboardOptionKey::Mime => b"mime",
        KittyClipboardOptionKey::Name => b"name",
        KittyClipboardOptionKey::Password => b"password",
        KittyClipboardOptionKey::Pw => b"pw",
        KittyClipboardOptionKey::Status => b"status",
        KittyClipboardOptionKey::Type => b"type",
    }
}

const VALID_ID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_+.";

fn is_valid_identifier(s: &[u8]) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < s.len() {
        let c = unsafe { *s.get_unchecked(i) };
        let mut found = false;
        let mut j = 0;
        while j < VALID_ID_CHARS.len() {
            if c == unsafe { *VALID_ID_CHARS.get_unchecked(j) } {
                found = true;
                break;
            }
            j += 1;
        }
        if !found {
            return false;
        }
        i += 1;
    }
    true
}

pub fn kitty_clipboard_read_option_str<'a>(
    key: KittyClipboardOptionKey,
    metadata: &'a [u8],
) -> Option<&'a [u8]> {
    let key_b = key_bytes(key);
    let mut pos: usize = 0;

    loop {
        while pos < metadata.len() && is_ascii_whitespace(unsafe { *metadata.get_unchecked(pos) }) {
            pos += 1;
        }
        if pos >= metadata.len() {
            return None;
        }

        let remaining = unsafe { metadata.get_unchecked(pos..) };
        if !starts_with(remaining, key_b) {
            let next = find_byte_kitty(metadata, b':', pos);
            match next {
                Some(n) => {
                    pos = n + 1;
                    continue;
                }
                None => return None,
            }
        }

        pos += key_b.len();

        while pos < metadata.len() && is_ascii_whitespace(unsafe { *metadata.get_unchecked(pos) }) {
            pos += 1;
        }
        if pos >= metadata.len() {
            return None;
        }
        if unsafe { *metadata.get_unchecked(pos) } != b'=' {
            return None;
        }

        let start = pos + 1;
        let end = find_byte_kitty(metadata, b':', start).unwrap_or(metadata.len());
        let value = trim_ws(unsafe { metadata.get_unchecked(start..end) });
        return Some(value);
    }
}

fn starts_with(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if unsafe { *haystack.get_unchecked(i) } != unsafe { *needle.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

pub fn kitty_clipboard_read_id<'a>(metadata: &'a [u8]) -> Option<&'a [u8]> {
    let value = kitty_clipboard_read_option_str(KittyClipboardOptionKey::Id, metadata)?;
    if is_valid_identifier(value) {
        Some(value)
    } else {
        None
    }
}

pub fn kitty_clipboard_read_loc(metadata: &[u8]) -> Option<KittyClipboardLocation> {
    let value = kitty_clipboard_read_option_str(KittyClipboardOptionKey::Loc, metadata)?;
    KittyClipboardLocation::parse(value)
}

pub fn kitty_clipboard_read_status(metadata: &[u8]) -> Option<KittyClipboardStatus> {
    let value = kitty_clipboard_read_option_str(KittyClipboardOptionKey::Status, metadata)?;
    KittyClipboardStatus::parse(value)
}

pub fn kitty_clipboard_read_type(metadata: &[u8]) -> Option<KittyClipboardOperation> {
    let value = kitty_clipboard_read_option_str(KittyClipboardOptionKey::Type, metadata)?;
    KittyClipboardOperation::parse(value)
}

pub fn kitty_clipboard_read_mime<'a>(metadata: &'a [u8]) -> Option<&'a [u8]> {
    kitty_clipboard_read_option_str(KittyClipboardOptionKey::Mime, metadata)
}

pub fn kitty_clipboard_read_name<'a>(metadata: &'a [u8]) -> Option<&'a [u8]> {
    kitty_clipboard_read_option_str(KittyClipboardOptionKey::Name, metadata)
}

pub fn kitty_clipboard_read_password<'a>(metadata: &'a [u8]) -> Option<&'a [u8]> {
    kitty_clipboard_read_option_str(KittyClipboardOptionKey::Password, metadata)
}

pub fn kitty_clipboard_read_pw<'a>(metadata: &'a [u8]) -> Option<&'a [u8]> {
    kitty_clipboard_read_option_str(KittyClipboardOptionKey::Pw, metadata)
}

#[derive(Clone, Copy)]
pub struct KittyClipboardOsc<'a> {
    pub metadata: &'a [u8],
    pub payload: Option<&'a [u8]>,
    pub terminator: OscTerminator,
}

pub fn parse_kitty_clipboard<'a>(data: &'a [u8], terminator: OscTerminator) -> KittyClipboardOsc<'a> {
    let semi = find_byte_kitty(data, b';', 0);
    match semi {
        Some(pos) => KittyClipboardOsc {
            metadata: unsafe { data.get_unchecked(..pos) },
            payload: Some(unsafe { data.get_unchecked(pos + 1..) }),
            terminator,
        },
        None => KittyClipboardOsc {
            metadata: data,
            payload: None,
            terminator,
        },
    }
}

// ─── Kitty Color Protocol (OSC 21) ─────────────────────────────────────────

pub const KITTY_COLOR_MAX_REQUESTS: usize = 64;

pub struct KittyColorOscResult {
    pub requests: [kitty_color::Request; KITTY_COLOR_MAX_REQUESTS],
    pub len: usize,
    pub terminator: OscTerminator,
}

impl KittyColorOscResult {
    pub fn new(terminator: OscTerminator) -> Self {
        Self {
            requests: [kitty_color::Request::Reset(kitty_color::Kind::Palette(0)); KITTY_COLOR_MAX_REQUESTS],
            len: 0,
            terminator,
        }
    }

    #[inline]
    pub fn push(&mut self, req: kitty_color::Request) -> bool {
        if self.len >= KITTY_COLOR_MAX_REQUESTS {
            return false;
        }
        unsafe {
            *self.requests.get_unchecked_mut(self.len) = req;
        }
        self.len += 1;
        true
    }

    #[inline]
    pub fn as_slice(&self) -> &[kitty_color::Request] {
        unsafe { self.requests.get_unchecked(..self.len) }
    }
}

fn find_byte_kc(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
    let mut i = start;
    while i < haystack.len() {
        if unsafe { *haystack.get_unchecked(i) } == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

use crate::osc_parser_color::rgb_parse;

pub fn parse_kitty_color(data: &[u8], terminator: OscTerminator) -> KittyColorOscResult {
    let mut result = KittyColorOscResult::new(terminator);
    let mut pos: usize = 0;

    loop {
        if pos > data.len() {
            break;
        }

        let kv_end = find_byte_kc(data, b';', pos).unwrap_or(data.len());
        let kv = unsafe { data.get_unchecked(pos..kv_end) };
        pos = kv_end + 1;

        if kv.is_empty() {
            if pos > data.len() {
                break;
            }
            continue;
        }

        let eq_pos = find_byte_kc(kv, b'=', 0);
        let (key, value) = match eq_pos {
            Some(ep) => {
                let k = unsafe { kv.get_unchecked(..ep) };
                let v = trim_ws(unsafe { kv.get_unchecked(ep + 1..) });
                (k, v)
            }
            None => {
                let k = kv;
                (k, &b""[..])
            }
        };

        if key.is_empty() {
            continue;
        }

        let kind = match kitty_color::Kind::parse(key) {
            Some(k) => k,
            None => continue,
        };

        if value.is_empty() {
            if !result.push(kitty_color::Request::Reset(kind)) {
                break;
            }
        } else if value.len() == 1 && unsafe { *value.get_unchecked(0) } == b'?' {
            if !result.push(kitty_color::Request::Query(kind)) {
                break;
            }
        } else {
            let rgb = match rgb_parse(value) {
                Some(c) => c,
                None => continue,
            };
            if !result.push(kitty_color::Request::Set { key: kind, color: rgb }) {
                break;
            }
        }

        if pos > data.len() {
            break;
        }
    }

    result
}

// ─── Kitty Text Sizing (OSC 66) ────────────────────────────────────────────

pub const KITTY_TEXT_MAX_PAYLOAD: usize = 4096;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KittyTextVAlign {
    Top = 0,
    Bottom = 1,
    Center = 2,
}

impl KittyTextVAlign {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(KittyTextVAlign::Top),
            1 => Some(KittyTextVAlign::Bottom),
            2 => Some(KittyTextVAlign::Center),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KittyTextHAlign {
    Left = 0,
    Right = 1,
    Center = 2,
}

impl KittyTextHAlign {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(KittyTextHAlign::Left),
            1 => Some(KittyTextHAlign::Right),
            2 => Some(KittyTextHAlign::Center),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KittyTextSizingOsc<'a> {
    pub scale: u8,
    pub width: u8,
    pub numerator: u8,
    pub denominator: u8,
    pub valign: KittyTextVAlign,
    pub halign: KittyTextHAlign,
    pub text: &'a [u8],
}

impl<'a> KittyTextSizingOsc<'a> {
    pub fn new(text: &'a [u8]) -> Self {
        Self {
            scale: 1,
            width: 0,
            numerator: 0,
            denominator: 0,
            valign: KittyTextVAlign::Top,
            halign: KittyTextHAlign::Left,
            text,
        }
    }

    fn update(&mut self, key: u8, value: &[u8]) -> bool {
        let v = match parse_u4(value) {
            Some(v) => v,
            None => return false,
        };

        match key {
            b's' => {
                if v == 0 {
                    return false;
                }
                if v > 7 {
                    return false;
                }
                self.scale = v;
            }
            b'w' => {
                if v > 7 {
                    return false;
                }
                self.width = v;
            }
            b'n' => {
                if v > 15 {
                    return false;
                }
                self.numerator = v;
            }
            b'd' => {
                if v > 15 {
                    return false;
                }
                self.denominator = v;
            }
            b'v' => {
                match KittyTextVAlign::from_u8(v) {
                    Some(a) => self.valign = a,
                    None => return false,
                }
            }
            b'h' => {
                match KittyTextHAlign::from_u8(v) {
                    Some(a) => self.halign = a,
                    None => return false,
                }
            }
            _ => return false,
        }
        true
    }
}

fn parse_u4(s: &[u8]) -> Option<u8> {
    if s.is_empty() || s.len() > 2 {
        return None;
    }
    let mut result: u16 = 0;
    let mut i = 0;
    while i < s.len() {
        let b = unsafe { *s.get_unchecked(i) };
        if b < b'0' || b > b'9' {
            return None;
        }
        result = result * 10 + (b - b'0') as u16;
        i += 1;
    }
    if result > 255 {
        return None;
    }
    Some(result as u8)
}

pub fn parse_kitty_text_sizing<'a>(data: &'a [u8]) -> Option<KittyTextSizingOsc<'a>> {
    let semi_pos = find_byte_kc(data, b';', 0)?;
    let payload = unsafe { data.get_unchecked(semi_pos + 1..) };

    if payload.len() > KITTY_TEXT_MAX_PAYLOAD {
        return None;
    }

    if !is_safe_utf8(payload) {
        return None;
    }

    let mut osc = KittyTextSizingOsc::new(payload);

    if semi_pos > 0 {
        let params = unsafe { data.get_unchecked(..semi_pos) };
        let mut kv_pos: usize = 0;

        loop {
            if kv_pos > params.len() {
                break;
            }
            let kv_end = find_byte_kc(params, b':', kv_pos).unwrap_or(params.len());
            let kv = unsafe { params.get_unchecked(kv_pos..kv_end) };
            kv_pos = kv_end + 1;

            let eq_pos = find_byte_kc(kv, b'=', 0);
            if let Some(ep) = eq_pos {
                let k = unsafe { kv.get_unchecked(..ep) };
                let v = unsafe { kv.get_unchecked(ep + 1..) };
                if k.len() == 1 {
                    let _ = osc.update(unsafe { *k.get_unchecked(0) }, v);
                }
            }

            if kv_pos > params.len() {
                break;
            }
        }
    }

    Some(osc)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Kitty Clipboard Tests ──────────────────────────────────────

    #[test]
    fn kitty_clipboard_empty_metadata_no_payload() {
        let osc = parse_kitty_clipboard(b"", OscTerminator::St);
        assert!(osc.metadata.is_empty());
        assert!(osc.payload.is_none());
    }

    #[test]
    fn kitty_clipboard_empty_metadata_empty_payload() {
        let osc = parse_kitty_clipboard(b";", OscTerminator::St);
        assert!(osc.metadata.is_empty());
        assert!(osc.payload == Some(&b""[..]));
    }

    #[test]
    fn kitty_clipboard_metadata_and_payload() {
        let osc = parse_kitty_clipboard(b"type=read;dGV4dC9wbGFpbg==", OscTerminator::St);
        assert!(bytes_eq_kitty(osc.metadata, b"type=read"));
        assert!(osc.payload == Some(&b"dGV4dC9wbGFpbg=="[..]));
    }

    #[test]
    fn kitty_clipboard_read_type() {
        let osc = parse_kitty_clipboard(b"type=read", OscTerminator::St);
        let t = kitty_clipboard_read_type(osc.metadata).unwrap();
        assert!(t == KittyClipboardOperation::Read);
    }

    #[test]
    fn kitty_clipboard_read_status() {
        let osc = parse_kitty_clipboard(b"type=read:status=OK", OscTerminator::St);
        let s = kitty_clipboard_read_status(osc.metadata).unwrap();
        assert!(s == KittyClipboardStatus::Ok);
    }

    #[test]
    fn kitty_clipboard_read_id_valid() {
        let osc = parse_kitty_clipboard(
            b"id=5c076ad9-d36f-4705-847b-d4dbf356cc0d",
            OscTerminator::St,
        );
        let id = kitty_clipboard_read_id(osc.metadata).unwrap();
        assert!(bytes_eq_kitty(id, b"5c076ad9-d36f-4705-847b-d4dbf356cc0d"));
    }

    #[test]
    fn kitty_clipboard_read_id_invalid() {
        let osc = parse_kitty_clipboard(b"id=*42*", OscTerminator::St);
        assert!(kitty_clipboard_read_id(osc.metadata).is_none());
    }

    #[test]
    fn kitty_clipboard_read_id_empty() {
        let osc = parse_kitty_clipboard(b"id=", OscTerminator::St);
        assert!(kitty_clipboard_read_id(osc.metadata).is_none());
    }

    #[test]
    fn kitty_clipboard_read_loc() {
        let osc = parse_kitty_clipboard(b"loc=primary", OscTerminator::St);
        let loc = kitty_clipboard_read_loc(osc.metadata).unwrap();
        assert!(loc == KittyClipboardLocation::Primary);
    }

    #[test]
    fn kitty_clipboard_read_loc_invalid() {
        let osc = parse_kitty_clipboard(b"loc=bobr", OscTerminator::St);
        assert!(kitty_clipboard_read_loc(osc.metadata).is_none());
    }

    #[test]
    fn kitty_clipboard_read_mime() {
        let osc = parse_kitty_clipboard(
            b"type=read:mime=dGV4dC9wbGFpbg==;R2hvc3R0eQ==",
            OscTerminator::St,
        );
        let mime = kitty_clipboard_read_mime(osc.metadata).unwrap();
        assert!(bytes_eq_kitty(mime, b"dGV4dC9wbGFpbg=="));
    }

    #[test]
    fn kitty_clipboard_password() {
        let osc = parse_kitty_clipboard(
            b"pw=R2hvc3R0eQ==:name=Qk9CUiBLVVJXQQ==",
            OscTerminator::St,
        );
        let pw = kitty_clipboard_read_pw(osc.metadata).unwrap();
        assert!(bytes_eq_kitty(pw, b"R2hvc3R0eQ=="));
        let name = kitty_clipboard_read_name(osc.metadata).unwrap();
        assert!(bytes_eq_kitty(name, b"Qk9CUiBLVVJXQQ=="));
    }

    #[test]
    fn kitty_clipboard_invalid_status() {
        let osc = parse_kitty_clipboard(b"status=BOBR", OscTerminator::St);
        assert!(kitty_clipboard_read_status(osc.metadata).is_none());
    }

    // ─── Kitty Color Tests ──────────────────────────────────────────

    #[test]
    fn kitty_color_empty() {
        let result = parse_kitty_color(b"", OscTerminator::St);
        assert!(result.len == 0);
    }

    #[test]
    fn kitty_color_query() {
        let result = parse_kitty_color(b"foreground=?", OscTerminator::St);
        assert!(result.len == 1);
        match result.as_slice()[0] {
            kitty_color::Request::Query(k) => {
                assert!(k == kitty_color::Kind::Special(kitty_color::Special::Foreground));
            }
            _ => panic!("expected Query"),
        }
    }

    #[test]
    fn kitty_color_set() {
        let result = parse_kitty_color(b"background=#aabbcc", OscTerminator::St);
        assert!(result.len == 1);
        match result.as_slice()[0] {
            kitty_color::Request::Set { key, color } => {
                assert!(key == kitty_color::Kind::Special(kitty_color::Special::Background));
                assert!(color.r == 0xaa && color.g == 0xbb && color.b == 0xcc);
            }
            _ => panic!("expected Set"),
        }
    }

    #[test]
    fn kitty_color_reset() {
        let result = parse_kitty_color(b"cursor_text", OscTerminator::St);
        assert!(result.len == 1);
        match result.as_slice()[0] {
            kitty_color::Request::Reset(k) => {
                assert!(k == kitty_color::Kind::Special(kitty_color::Special::CursorText));
            }
            _ => panic!("expected Reset"),
        }
    }

    #[test]
    fn kitty_color_multiple() {
        let result = parse_kitty_color(
            b"foreground=?;background=rgb:f0/f8/ff;cursor_text",
            OscTerminator::St,
        );
        assert!(result.len == 3);
    }

    #[test]
    fn kitty_color_palette() {
        let result = parse_kitty_color(b"2=?", OscTerminator::St);
        assert!(result.len == 1);
        match result.as_slice()[0] {
            kitty_color::Request::Query(k) => {
                assert!(k == kitty_color::Kind::Palette(2));
            }
            _ => panic!("expected Query"),
        }
    }

    // ─── Kitty Text Sizing Tests ────────────────────────────────────

    #[test]
    fn kitty_text_empty_params() {
        let osc = parse_kitty_text_sizing(b";bobr").unwrap();
        assert!(osc.scale == 1);
        assert!(bytes_eq_kitty(osc.text, b"bobr"));
    }

    #[test]
    fn kitty_text_single_param() {
        let osc = parse_kitty_text_sizing(b"s=2;kurwa").unwrap();
        assert!(osc.scale == 2);
        assert!(bytes_eq_kitty(osc.text, b"kurwa"));
    }

    #[test]
    fn kitty_text_multiple_params() {
        let osc = parse_kitty_text_sizing(b"s=2:w=7:n=13:d=15:v=1:h=2;long").unwrap();
        assert!(osc.scale == 2);
        assert!(osc.width == 7);
        assert!(osc.numerator == 13);
        assert!(osc.denominator == 15);
        assert!(osc.valign == KittyTextVAlign::Bottom);
        assert!(osc.halign == KittyTextHAlign::Center);
        assert!(bytes_eq_kitty(osc.text, b"long"));
    }

    #[test]
    fn kitty_text_scale_zero() {
        let osc = parse_kitty_text_sizing(b"s=0;nope").unwrap();
        assert!(osc.scale == 1);
    }

    #[test]
    fn kitty_text_invalid_params() {
        let osc = parse_kitty_text_sizing(b"w=8:v=3:n=16;").unwrap();
        assert!(osc.width == 0);
        assert!(osc.valign == KittyTextVAlign::Top);
        assert!(osc.numerator == 0);
    }

    #[test]
    fn kitty_text_no_semicolon() {
        assert!(parse_kitty_text_sizing(b"no_semicolon").is_none());
    }

    #[test]
    fn kitty_text_payload_too_long() {
        let mut buf = [b'a'; 4200];
        buf[0] = b';';
        assert!(parse_kitty_text_sizing(&buf).is_none());
    }
}
