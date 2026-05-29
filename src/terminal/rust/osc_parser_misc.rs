#![allow(unused)]

use crate::bytes_util::{bytes_to_str, find_byte_from, subslice, subslice_from};
use crate::osc_types::*;

// ─── OSC 8 Hyperlinks ──────────────────────────────────────────────────────

fn find_byte_misc(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
    find_byte_from(haystack, needle, start)
}

fn bytes_eq_misc(a: &[u8], b: &[u8]) -> bool {
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

fn bytes_to_str_misc(bytes: &[u8]) -> &str {
    bytes_to_str(bytes)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HyperlinkOscResult<'a> {
    Start { uri: &'a str, id: Option<&'a str> },
    End,
    Invalid,
}

pub fn parse_hyperlink<'a>(data: &'a [u8]) -> HyperlinkOscResult<'a> {
    let s = match find_byte_misc(data, b';', 0) {
        Some(pos) => pos,
        None => return HyperlinkOscResult::Invalid,
    };

    let uri = bytes_to_str_misc(unsafe { data.get_unchecked(s + 1..) });
    let kvs = unsafe { data.get_unchecked(..s) };

    let mut id: Option<&str> = None;

    let mut kv_start: usize = 0;
    loop {
        if kv_start >= kvs.len() {
            break;
        }

        let kv_end = find_byte_misc(kvs, b':', kv_start).unwrap_or(kvs.len());
        let kv = unsafe { kvs.get_unchecked(kv_start..kv_end) };

        if let Some(v) = find_byte_misc(kv, b'=', 0) {
            let key = unsafe { kv.get_unchecked(..v) };
            let value = unsafe { kv.get_unchecked(v + 1..) };
            if bytes_eq_misc(key, b"id") && !value.is_empty() {
                id = Some(bytes_to_str_misc(value));
            }
        }

        if kv_end >= kvs.len() {
            break;
        }
        kv_start = kv_end + 1;
    }

    if uri.is_empty() {
        if id.is_some() {
            return HyperlinkOscResult::Invalid;
        }
        return HyperlinkOscResult::End;
    }

    HyperlinkOscResult::Start { uri, id }
}

// ─── OSC 52 Clipboard ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOscResult<'a> {
    Contents { kind: u8, data: &'a str },
    Invalid,
}

pub fn parse_clipboard<'a>(data: &'a [u8]) -> ClipboardOscResult<'a> {
    if data.len() <= 1 {
        return ClipboardOscResult::Invalid;
    }

    let first = unsafe { *data.get_unchecked(0) };

    if first == b';' {
        let rest = unsafe { data.get_unchecked(1..) };
        return ClipboardOscResult::Contents {
            kind: b'c',
            data: bytes_to_str_misc(rest),
        };
    }

    if data.len() < 2 {
        return ClipboardOscResult::Invalid;
    }

    if unsafe { *data.get_unchecked(1) } != b';' {
        return ClipboardOscResult::Invalid;
    }

    let rest = unsafe { data.get_unchecked(2..) };
    ClipboardOscResult::Contents {
        kind: first,
        data: bytes_to_str_misc(rest),
    }
}

// ─── OSC 1337 iTerm2 ───────────────────────────────────────────────────────

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Iterm2Key {
    AddAnnotation = 0,
    AddHiddenAnnotation,
    Block,
    Button,
    ClearCapturedOutput,
    ClearScrollback,
    Copy,
    CopyToClipboard,
    CurrentDir,
    CursorShape,
    Custom,
    Disinter,
    EndCopy,
    File,
    FileEnd,
    FilePart,
    HighlightCursorLine,
    MultipartFile,
    OpenUrl,
    PopKeyLabels,
    PushKeyLabels,
    RemoteHost,
    ReportCellSize,
    ReportVariable,
    RequestAttention,
    RequestUpload,
    SetBackgroundImageFile,
    SetBadgeFormat,
    SetColors,
    SetKeyLabel,
    SetMark,
    SetProfile,
    SetUserVar,
    ShellIntegrationVersion,
    StealFocus,
    UnicodeVersion,
}

fn to_lower(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

fn bytes_eq_icase(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        let ca = to_lower(unsafe { *a.get_unchecked(i) });
        let cb = to_lower(unsafe { *b.get_unchecked(i) });
        if ca != cb {
            return false;
        }
        i += 1;
    }
    true
}

fn parse_iterm2_key(key_str: &[u8]) -> Option<Iterm2Key> {
    const KEYS: &[(&[u8], Iterm2Key)] = &[
        (b"AddAnnotation", Iterm2Key::AddAnnotation),
        (b"AddHiddenAnnotation", Iterm2Key::AddHiddenAnnotation),
        (b"Block", Iterm2Key::Block),
        (b"Button", Iterm2Key::Button),
        (b"ClearCapturedOutput", Iterm2Key::ClearCapturedOutput),
        (b"ClearScrollback", Iterm2Key::ClearScrollback),
        (b"Copy", Iterm2Key::Copy),
        (b"CopyToClipboard", Iterm2Key::CopyToClipboard),
        (b"CurrentDir", Iterm2Key::CurrentDir),
        (b"CursorShape", Iterm2Key::CursorShape),
        (b"Custom", Iterm2Key::Custom),
        (b"Disinter", Iterm2Key::Disinter),
        (b"EndCopy", Iterm2Key::EndCopy),
        (b"File", Iterm2Key::File),
        (b"FileEnd", Iterm2Key::FileEnd),
        (b"FilePart", Iterm2Key::FilePart),
        (b"HighlightCursorLine", Iterm2Key::HighlightCursorLine),
        (b"MultipartFile", Iterm2Key::MultipartFile),
        (b"OpenURL", Iterm2Key::OpenUrl),
        (b"PopKeyLabels", Iterm2Key::PopKeyLabels),
        (b"PushKeyLabels", Iterm2Key::PushKeyLabels),
        (b"RemoteHost", Iterm2Key::RemoteHost),
        (b"ReportCellSize", Iterm2Key::ReportCellSize),
        (b"ReportVariable", Iterm2Key::ReportVariable),
        (b"RequestAttention", Iterm2Key::RequestAttention),
        (b"RequestUpload", Iterm2Key::RequestUpload),
        (b"SetBackgroundImageFile", Iterm2Key::SetBackgroundImageFile),
        (b"SetBadgeFormat", Iterm2Key::SetBadgeFormat),
        (b"SetColors", Iterm2Key::SetColors),
        (b"SetKeyLabel", Iterm2Key::SetKeyLabel),
        (b"SetMark", Iterm2Key::SetMark),
        (b"SetProfile", Iterm2Key::SetProfile),
        (b"SetUserVar", Iterm2Key::SetUserVar),
        (
            b"ShellIntegrationVersion",
            Iterm2Key::ShellIntegrationVersion,
        ),
        (b"StealFocus", Iterm2Key::StealFocus),
        (b"UnicodeVersion", Iterm2Key::UnicodeVersion),
    ];

    let mut i = 0;
    while i < KEYS.len() {
        let (name, key) = unsafe { *KEYS.get_unchecked(i) };
        if bytes_eq_icase(key_str, name) {
            return Some(key);
        }
        i += 1;
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Iterm2OscResult<'a> {
    ClipboardContents { kind: u8, data: &'a str },
    ReportPwd { value: &'a str },
    Invalid,
}

pub fn parse_iterm2<'a>(data: &'a [u8]) -> Iterm2OscResult<'a> {
    let eq_pos = find_byte_misc(data, b'=', 0);

    let (key_str, value) = match eq_pos {
        Some(pos) => {
            let k = unsafe { data.get_unchecked(..pos) };
            let v = unsafe { data.get_unchecked(pos + 1..) };
            (k, Some(v))
        }
        None => (data, None),
    };

    let key = match parse_iterm2_key(key_str) {
        Some(k) => k,
        None => return Iterm2OscResult::Invalid,
    };

    match key {
        Iterm2Key::Copy => {
            let v = match value {
                Some(v) => v,
                None => return Iterm2OscResult::Invalid,
            };
            if v.is_empty() {
                return Iterm2OscResult::Invalid;
            }
            if unsafe { *v.get_unchecked(0) } != b':' {
                return Iterm2OscResult::Invalid;
            }
            let inner = unsafe { v.get_unchecked(1..) };
            if inner.is_empty() {
                return Iterm2OscResult::Invalid;
            }
            if inner.len() == 1 && unsafe { *inner.get_unchecked(0) } == b'?' {
                return Iterm2OscResult::Invalid;
            }
            Iterm2OscResult::ClipboardContents {
                kind: b'c',
                data: bytes_to_str_misc(inner),
            }
        }
        Iterm2Key::CurrentDir => {
            let v = match value {
                Some(v) => v,
                None => return Iterm2OscResult::Invalid,
            };
            if v.is_empty() {
                return Iterm2OscResult::Invalid;
            }
            Iterm2OscResult::ReportPwd {
                value: bytes_to_str_misc(v),
            }
        }
        _ => Iterm2OscResult::Invalid,
    }
}

// ─── OSC 3008 Context Signal ───────────────────────────────────────────────

const MAX_CONTEXT_ID_LEN: usize = 64;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContextSignalAction {
    Start = 0,
    End = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContextSignalType {
    Boot = 0,
    Container,
    Vm,
    Elevate,
    Chpriv,
    Subcontext,
    Remote,
    Shell,
    Command,
    App,
    Service,
    Session,
}

impl ContextSignalType {
    #[inline]
    pub fn parse(s: &[u8]) -> Option<Self> {
        match s {
            b"boot" => Some(ContextSignalType::Boot),
            b"container" => Some(ContextSignalType::Container),
            b"vm" => Some(ContextSignalType::Vm),
            b"elevate" => Some(ContextSignalType::Elevate),
            b"chpriv" => Some(ContextSignalType::Chpriv),
            b"subcontext" => Some(ContextSignalType::Subcontext),
            b"remote" => Some(ContextSignalType::Remote),
            b"shell" => Some(ContextSignalType::Shell),
            b"command" => Some(ContextSignalType::Command),
            b"app" => Some(ContextSignalType::App),
            b"service" => Some(ContextSignalType::Service),
            b"session" => Some(ContextSignalType::Session),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContextSignalExit {
    Success = 0,
    Failure,
    Crash,
    Interrupt,
}

impl ContextSignalExit {
    #[inline]
    pub fn parse(s: &[u8]) -> Option<Self> {
        match s {
            b"success" => Some(ContextSignalExit::Success),
            b"failure" => Some(ContextSignalExit::Failure),
            b"crash" => Some(ContextSignalExit::Crash),
            b"interrupt" => Some(ContextSignalExit::Interrupt),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContextSignalField {
    Type,
    User,
    Hostname,
    Machineid,
    Bootid,
    Pid,
    Pidfdid,
    Comm,
    Cwd,
    Cmdline,
    Vm,
    Container,
    Targetuser,
    Targethost,
    Sessionid,
    Exit,
    Status,
    Signal,
}

fn field_key(f: ContextSignalField) -> &'static [u8] {
    match f {
        ContextSignalField::Type => b"type",
        ContextSignalField::User => b"user",
        ContextSignalField::Hostname => b"hostname",
        ContextSignalField::Machineid => b"machineid",
        ContextSignalField::Bootid => b"bootid",
        ContextSignalField::Pid => b"pid",
        ContextSignalField::Pidfdid => b"pidfdid",
        ContextSignalField::Comm => b"comm",
        ContextSignalField::Cwd => b"cwd",
        ContextSignalField::Cmdline => b"cmdline",
        ContextSignalField::Vm => b"vm",
        ContextSignalField::Container => b"container",
        ContextSignalField::Targetuser => b"targetuser",
        ContextSignalField::Targethost => b"targethost",
        ContextSignalField::Sessionid => b"sessionid",
        ContextSignalField::Exit => b"exit",
        ContextSignalField::Status => b"status",
        ContextSignalField::Signal => b"signal",
    }
}

fn parse_u64(s: &[u8]) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let mut result: u64 = 0;
    let mut i = 0;
    while i < s.len() {
        let b = unsafe { *s.get_unchecked(i) };
        if b < b'0' || b > b'9' {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0') as u64)?;
        i += 1;
    }
    Some(result)
}

pub fn context_signal_read_str<'a>(
    field: ContextSignalField,
    metadata: &'a [u8],
) -> Option<&'a [u8]> {
    let key = field_key(field);
    let mut pos: usize = 0;

    loop {
        if pos >= metadata.len() {
            return None;
        }

        let end = find_byte_misc(metadata, b';', pos).unwrap_or(metadata.len());
        let full = unsafe { metadata.get_unchecked(pos..end) };

        if let Some(eql) = find_byte_misc(full, b'=', 0) {
            let k = unsafe { full.get_unchecked(..eql) };
            if bytes_eq_misc(k, key) {
                let value = unsafe { full.get_unchecked(eql + 1..) };
                return Some(value);
            }
        }

        if end >= metadata.len() {
            return None;
        }
        pos = end + 1;
    }
}

pub fn context_signal_read_type(metadata: &[u8]) -> Option<ContextSignalType> {
    let value = context_signal_read_str(ContextSignalField::Type, metadata)?;
    ContextSignalType::parse(value)
}

pub fn context_signal_read_exit(metadata: &[u8]) -> Option<ContextSignalExit> {
    let value = context_signal_read_str(ContextSignalField::Exit, metadata)?;
    ContextSignalExit::parse(value)
}

pub fn context_signal_read_u64(field: ContextSignalField, metadata: &[u8]) -> Option<u64> {
    let value = context_signal_read_str(field, metadata)?;
    parse_u64(value)
}

#[derive(Clone, Copy)]
pub struct ContextSignalOsc<'a> {
    pub action: ContextSignalAction,
    pub id: &'a [u8],
    pub metadata: &'a [u8],
}

pub fn parse_context_signal<'a>(data: &'a [u8]) -> Option<ContextSignalOsc<'a>> {
    if data.is_empty() {
        return None;
    }

    let (action, prefix_len) = if starts_with_cs(data, b"start=") {
        (ContextSignalAction::Start, 6usize)
    } else if starts_with_cs(data, b"end=") {
        (ContextSignalAction::End, 4usize)
    } else {
        return None;
    };

    let rest = unsafe { data.get_unchecked(prefix_len..) };
    if rest.is_empty() {
        return None;
    }

    let id_end = find_byte_misc(rest, b';', 0).unwrap_or(rest.len());
    let id = unsafe { rest.get_unchecked(..id_end) };

    if id.is_empty() || id.len() > MAX_CONTEXT_ID_LEN {
        return None;
    }

    let mut i = 0;
    while i < id.len() {
        let c = unsafe { *id.get_unchecked(i) };
        if c < 0x20 || c > 0x7e {
            return None;
        }
        i += 1;
    }

    let metadata = if id_end < rest.len() {
        unsafe { rest.get_unchecked(id_end + 1..) }
    } else {
        &b""[..]
    };

    Some(ContextSignalOsc {
        action,
        id,
        metadata,
    })
}

fn starts_with_cs(haystack: &[u8], needle: &[u8]) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Hyperlink Tests ────────────────────────────────────────────

    #[test]
    fn hyperlink_basic() {
        match parse_hyperlink(b";http://example.com") {
            HyperlinkOscResult::Start { uri, id } => {
                assert!(uri == "http://example.com");
                assert!(id.is_none());
            }
            _ => panic!("expected Start"),
        }
    }

    #[test]
    fn hyperlink_with_id() {
        match parse_hyperlink(b"id=foo;http://example.com") {
            HyperlinkOscResult::Start { uri, id } => {
                assert!(uri == "http://example.com");
                assert!(id == Some("foo"));
            }
            _ => panic!("expected Start"),
        }
    }

    #[test]
    fn hyperlink_empty_id() {
        match parse_hyperlink(b"id=;http://example.com") {
            HyperlinkOscResult::Start { uri, id } => {
                assert!(uri == "http://example.com");
                assert!(id.is_none());
            }
            _ => panic!("expected Start"),
        }
    }

    #[test]
    fn hyperlink_end() {
        match parse_hyperlink(b";;") {
            HyperlinkOscResult::End => {}
            _ => panic!("expected End"),
        }
    }

    #[test]
    fn hyperlink_empty_uri_with_id_is_invalid() {
        match parse_hyperlink(b"id=foo;") {
            HyperlinkOscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn hyperlink_no_semicolon() {
        match parse_hyperlink(b"no_semicolon") {
            HyperlinkOscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    // ─── Clipboard Tests ────────────────────────────────────────────

    #[test]
    fn clipboard_get_set() {
        match parse_clipboard(b"s;?") {
            ClipboardOscResult::Contents { kind, data } => {
                assert!(kind == b's');
                assert!(data == "?");
            }
            _ => panic!("expected Contents"),
        }
    }

    #[test]
    fn clipboard_optional_param() {
        match parse_clipboard(b";?") {
            ClipboardOscResult::Contents { kind, data } => {
                assert!(kind == b'c');
                assert!(data == "?");
            }
            _ => panic!("expected Contents"),
        }
    }

    #[test]
    fn clipboard_clear() {
        match parse_clipboard(b";") {
            ClipboardOscResult::Contents { kind, data } => {
                assert!(kind == b'c');
                assert!(data.is_empty());
            }
            _ => panic!("expected Contents"),
        }
    }

    #[test]
    fn clipboard_invalid_too_short() {
        match parse_clipboard(b"x") {
            ClipboardOscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    // ─── iTerm2 Tests ───────────────────────────────────────────────

    #[test]
    fn iterm2_copy_valid() {
        match parse_iterm2(b"Copy=:YWJjMTIz") {
            Iterm2OscResult::ClipboardContents { kind, data } => {
                assert!(kind == b'c');
                assert!(data == "YWJjMTIz");
            }
            _ => panic!("expected ClipboardContents"),
        }
    }

    #[test]
    fn iterm2_copy_no_value() {
        match parse_iterm2(b"Copy") {
            Iterm2OscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn iterm2_copy_no_colon() {
        match parse_iterm2(b"Copy=YWJjMTIz") {
            Iterm2OscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn iterm2_copy_question_mark() {
        match parse_iterm2(b"Copy=:?") {
            Iterm2OscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn iterm2_current_dir() {
        match parse_iterm2(b"CurrentDir=abc123") {
            Iterm2OscResult::ReportPwd { value } => {
                assert!(value == "abc123");
            }
            _ => panic!("expected ReportPwd"),
        }
    }

    #[test]
    fn iterm2_current_dir_empty() {
        match parse_iterm2(b"CurrentDir=") {
            Iterm2OscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn iterm2_unknown_key() {
        match parse_iterm2(b"BobrKurwa=abc123") {
            Iterm2OscResult::Invalid => {}
            _ => panic!("expected Invalid"),
        }
    }

    #[test]
    fn iterm2_case_insensitive() {
        match parse_iterm2(b"setbadgeformat=abc123") {
            Iterm2OscResult::Invalid => {}
            _ => panic!("expected Invalid (unimplemented key)"),
        }
    }

    // ─── Context Signal Tests ───────────────────────────────────────

    #[test]
    fn context_signal_start() {
        let osc = parse_context_signal(b"start=my-context").unwrap();
        assert!(osc.action == ContextSignalAction::Start);
        assert!(bytes_eq_misc(osc.id, b"my-context"));
        assert!(osc.metadata.is_empty());
    }

    #[test]
    fn context_signal_end() {
        let osc = parse_context_signal(b"end=my-context").unwrap();
        assert!(osc.action == ContextSignalAction::End);
        assert!(bytes_eq_misc(osc.id, b"my-context"));
    }

    #[test]
    fn context_signal_with_metadata() {
        let osc = parse_context_signal(b"start=ctx;type=container;user=lennart").unwrap();
        assert!(osc.action == ContextSignalAction::Start);
        assert!(bytes_eq_misc(osc.id, b"ctx"));
        let t = context_signal_read_type(osc.metadata).unwrap();
        assert!(t == ContextSignalType::Container);
        let user = context_signal_read_str(ContextSignalField::User, osc.metadata).unwrap();
        assert!(bytes_eq_misc(user, b"lennart"));
    }

    #[test]
    fn context_signal_end_with_exit() {
        let osc = parse_context_signal(b"end=ctx;exit=success;status=0").unwrap();
        let exit = context_signal_read_exit(osc.metadata).unwrap();
        assert!(exit == ContextSignalExit::Success);
        let status = context_signal_read_u64(ContextSignalField::Status, osc.metadata).unwrap();
        assert!(status == 0);
    }

    #[test]
    fn context_signal_invalid_no_prefix() {
        assert!(parse_context_signal(b"invalid=data").is_none());
    }

    #[test]
    fn context_signal_invalid_empty_id() {
        assert!(parse_context_signal(b"start=").is_none());
    }

    #[test]
    fn context_signal_invalid_empty_data() {
        assert!(parse_context_signal(b"").is_none());
    }

    #[test]
    fn context_signal_id_too_long() {
        let mut buf = [b'a'; 200];
        buf[..6].copy_from_slice(b"start=");
        assert!(parse_context_signal(&buf).is_none());
    }

    #[test]
    fn context_signal_pid() {
        let osc = parse_context_signal(b"start=ctx;pid=12345").unwrap();
        let pid = context_signal_read_u64(ContextSignalField::Pid, osc.metadata).unwrap();
        assert!(pid == 12345);
    }

    #[test]
    fn context_type_parse() {
        assert!(ContextSignalType::parse(b"boot") == Some(ContextSignalType::Boot));
        assert!(ContextSignalType::parse(b"container") == Some(ContextSignalType::Container));
        assert!(ContextSignalType::parse(b"vm") == Some(ContextSignalType::Vm));
        assert!(ContextSignalType::parse(b"shell") == Some(ContextSignalType::Shell));
        assert!(ContextSignalType::parse(b"unknown").is_none());
    }

    #[test]
    fn context_exit_parse() {
        assert!(ContextSignalExit::parse(b"success") == Some(ContextSignalExit::Success));
        assert!(ContextSignalExit::parse(b"failure") == Some(ContextSignalExit::Failure));
        assert!(ContextSignalExit::parse(b"crash") == Some(ContextSignalExit::Crash));
        assert!(ContextSignalExit::parse(b"interrupt") == Some(ContextSignalExit::Interrupt));
        assert!(ContextSignalExit::parse(b"unknown").is_none());
    }
}
