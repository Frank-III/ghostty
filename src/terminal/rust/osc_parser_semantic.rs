#![allow(unused)]

use crate::stream_types::{SemanticPromptAction, SemanticPrompt};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticClick {
    Line = 0,
    Multiple = 1,
    ConservativeVertical = 2,
    SmartVertical = 3,
}

impl SemanticClick {
    #[inline]
    pub fn parse(value: &[u8]) -> Option<Self> {
        if value.len() == 1 {
            match unsafe { *value.get_unchecked(0) } {
                b'm' => Some(SemanticClick::Multiple),
                b'v' => Some(SemanticClick::ConservativeVertical),
                b'w' => Some(SemanticClick::SmartVertical),
                _ => None,
            }
        } else if value.len() == 4
            && unsafe { *value.get_unchecked(0) } == b'l'
            && unsafe { *value.get_unchecked(1) } == b'i'
            && unsafe { *value.get_unchecked(2) } == b'n'
            && unsafe { *value.get_unchecked(3) } == b'e'
        {
            Some(SemanticClick::Line)
        } else {
            None
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticPromptKind {
    Initial = 0,
    Right = 1,
    Continuation = 2,
    Secondary = 3,
}

impl SemanticPromptKind {
    #[inline]
    pub fn parse(c: u8) -> Option<Self> {
        match c {
            b'i' => Some(SemanticPromptKind::Initial),
            b'r' => Some(SemanticPromptKind::Right),
            b'c' => Some(SemanticPromptKind::Continuation),
            b's' => Some(SemanticPromptKind::Secondary),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticRedraw {
    True = 0,
    False = 1,
    Last = 2,
}

impl SemanticRedraw {
    #[inline]
    pub fn parse(value: &[u8]) -> Option<Self> {
        if value.len() == 1 && unsafe { *value.get_unchecked(0) } == b'0' {
            Some(SemanticRedraw::False)
        } else if value.len() == 1 && unsafe { *value.get_unchecked(0) } == b'1' {
            Some(SemanticRedraw::True)
        } else if value.len() == 4
            && unsafe { *value.get_unchecked(0) } == b'l'
            && unsafe { *value.get_unchecked(1) } == b'a'
            && unsafe { *value.get_unchecked(2) } == b's'
            && unsafe { *value.get_unchecked(3) } == b't'
        {
            Some(SemanticRedraw::Last)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticOption {
    Aid,
    Cl,
    PromptKind,
    Err,
    Redraw,
    SpecialKey,
    ClickEvents,
    Cmdline,
    CmdlineUrl,
    ExitCode,
}

fn find_byte_in(haystack: &[u8], needle: u8, start: usize) -> Option<usize> {
    let mut i = start;
    while i < haystack.len() {
        if unsafe { *haystack.get_unchecked(i) } == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
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

fn parse_i32(s: &[u8]) -> Option<i32> {
    if s.is_empty() {
        return None;
    }
    let mut i = 0;
    let negative = unsafe { *s.get_unchecked(0) } == b'-';
    if negative {
        i = 1;
    }
    if i >= s.len() {
        return None;
    }
    let mut result: i64 = 0;
    while i < s.len() {
        let b = unsafe { *s.get_unchecked(i) };
        if b < b'0' || b > b'9' {
            return None;
        }
        result = result * 10 + (b - b'0') as i64;
        i += 1;
    }
    if negative {
        result = -result;
    }
    if result < i32::MIN as i64 || result > i32::MAX as i64 {
        return None;
    }
    Some(result as i32)
}

fn option_key(opt: SemanticOption) -> &'static [u8] {
    match opt {
        SemanticOption::Aid => b"aid",
        SemanticOption::Cl => b"cl",
        SemanticOption::PromptKind => b"k",
        SemanticOption::Err => b"err",
        SemanticOption::Redraw => b"redraw",
        SemanticOption::SpecialKey => b"special_key",
        SemanticOption::ClickEvents => b"click_events",
        SemanticOption::Cmdline => b"cmdline",
        SemanticOption::CmdlineUrl => b"cmdline_url",
        SemanticOption::ExitCode => b"",
    }
}

pub fn read_option_str<'a>(opt: SemanticOption, raw: &'a [u8]) -> Option<&'a [u8]> {
    if opt == SemanticOption::ExitCode {
        return None;
    }
    let key = option_key(opt);
    let mut remaining = raw;

    loop {
        if remaining.is_empty() {
            return None;
        }

        let len = find_byte_in(remaining, b';', 0).unwrap_or(remaining.len());
        let full = unsafe { remaining.get_unchecked(..len) };

        let eql_idx = find_byte_in(full, b'=', 0);
        if let Some(ei) = eql_idx {
            let k = unsafe { full.get_unchecked(..ei) };
            if bytes_eq(k, key) {
                let value = unsafe { full.get_unchecked(ei + 1..) };
                return Some(value);
            }
        }

        if len < remaining.len() {
            remaining = unsafe { remaining.get_unchecked(len + 1..) };
        } else {
            return None;
        }
    }
}

pub fn read_option_aid<'a>(raw: &'a [u8]) -> Option<&'a [u8]> {
    read_option_str(SemanticOption::Aid, raw)
}

pub fn read_option_cl(raw: &[u8]) -> Option<SemanticClick> {
    let value = read_option_str(SemanticOption::Cl, raw)?;
    SemanticClick::parse(value)
}

pub fn read_option_prompt_kind(raw: &[u8]) -> Option<SemanticPromptKind> {
    let value = read_option_str(SemanticOption::PromptKind, raw)?;
    if value.len() == 1 {
        SemanticPromptKind::parse(unsafe { *value.get_unchecked(0) })
    } else {
        None
    }
}

pub fn read_option_err<'a>(raw: &'a [u8]) -> Option<&'a [u8]> {
    read_option_str(SemanticOption::Err, raw)
}

pub fn read_option_redraw(raw: &[u8]) -> Option<SemanticRedraw> {
    let value = read_option_str(SemanticOption::Redraw, raw)?;
    SemanticRedraw::parse(value)
}

pub fn read_option_bool(raw: &[u8], opt: SemanticOption) -> Option<bool> {
    let value = read_option_str(opt, raw)?;
    if value.len() == 1 {
        match unsafe { *value.get_unchecked(0) } {
            b'0' => Some(false),
            b'1' => Some(true),
            _ => None,
        }
    } else {
        None
    }
}

pub fn read_option_cmdline<'a>(raw: &'a [u8]) -> Option<&'a [u8]> {
    read_option_str(SemanticOption::Cmdline, raw)
}

pub fn read_option_cmdline_url<'a>(raw: &'a [u8]) -> Option<&'a [u8]> {
    read_option_str(SemanticOption::CmdlineUrl, raw)
}

pub fn read_option_exit_code(raw: &[u8]) -> Option<i32> {
    if raw.is_empty() {
        return None;
    }
    let len = find_byte_in(raw, b';', 0).unwrap_or(raw.len());
    let first = unsafe { raw.get_unchecked(..len) };
    parse_i32(first)
}

pub fn parse_semantic_prompt(data: &[u8]) -> Option<SemanticPrompt<'_>> {
    if data.is_empty() {
        return None;
    }

    let ch = unsafe { *data.get_unchecked(0) };
    let action = match ch {
        b'L' => {
            if data.len() > 1 {
                return None;
            }
            SemanticPromptAction::FreshLine
        }
        b'A' => SemanticPromptAction::FreshLineNewPrompt,
        b'N' => SemanticPromptAction::NewCommand,
        b'P' => SemanticPromptAction::PromptStart,
        b'B' => SemanticPromptAction::EndPromptStartInput,
        b'I' => SemanticPromptAction::EndPromptStartInputTerminateEol,
        b'C' => SemanticPromptAction::EndInputStartOutput,
        b'D' => SemanticPromptAction::EndCommand,
        _ => return None,
    };

    if data.len() == 1 {
        return Some(SemanticPrompt {
            action,
            options_unvalidated: b"",
        });
    }

    if unsafe { *data.get_unchecked(1) } != b';' {
        return None;
    }

    let options = unsafe { data.get_unchecked(2..) };
    Some(SemanticPrompt {
        action,
        options_unvalidated: options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fresh_line() {
        let r = parse_semantic_prompt(b"L").unwrap();
        assert!(r.action == SemanticPromptAction::FreshLine);
    }

    #[test]
    fn parse_fresh_line_extra() {
        assert!(parse_semantic_prompt(b"Lol").is_none());
    }

    #[test]
    fn parse_fresh_line_new_prompt() {
        let r = parse_semantic_prompt(b"A").unwrap();
        assert!(r.action == SemanticPromptAction::FreshLineNewPrompt);
    }

    #[test]
    fn parse_fresh_line_new_prompt_with_aid() {
        let r = parse_semantic_prompt(b"A;aid=14").unwrap();
        assert!(r.action == SemanticPromptAction::FreshLineNewPrompt);
        let aid = read_option_aid(r.options_unvalidated).unwrap();
        assert!(bytes_eq(aid, b"14"));
    }

    #[test]
    fn parse_end_input_start_output() {
        let r = parse_semantic_prompt(b"C").unwrap();
        assert!(r.action == SemanticPromptAction::EndInputStartOutput);
    }

    #[test]
    fn parse_end_command() {
        let r = parse_semantic_prompt(b"D").unwrap();
        assert!(r.action == SemanticPromptAction::EndCommand);
    }

    #[test]
    fn parse_end_command_with_exit_code() {
        let r = parse_semantic_prompt(b"D;0").unwrap();
        assert!(r.action == SemanticPromptAction::EndCommand);
        let code = read_option_exit_code(r.options_unvalidated).unwrap();
        assert!(code == 0);
    }

    #[test]
    fn parse_end_command_with_exit_code_and_aid() {
        let r = parse_semantic_prompt(b"D;12;aid=foo").unwrap();
        let code = read_option_exit_code(r.options_unvalidated).unwrap();
        assert!(code == 12);
        let aid = read_option_aid(r.options_unvalidated).unwrap();
        assert!(bytes_eq(aid, b"foo"));
    }

    #[test]
    fn read_option_cl_line() {
        assert!(SemanticClick::parse(b"line") == Some(SemanticClick::Line));
    }

    #[test]
    fn read_option_cl_m() {
        assert!(SemanticClick::parse(b"m") == Some(SemanticClick::Multiple));
    }

    #[test]
    fn read_option_cl_v() {
        assert!(SemanticClick::parse(b"v") == Some(SemanticClick::ConservativeVertical));
    }

    #[test]
    fn read_option_cl_w() {
        assert!(SemanticClick::parse(b"w") == Some(SemanticClick::SmartVertical));
    }

    #[test]
    fn read_option_cl_invalid() {
        assert!(SemanticClick::parse(b"invalid").is_none());
    }

    #[test]
    fn read_option_prompt_kind_i() {
        assert!(SemanticPromptKind::parse(b'i') == Some(SemanticPromptKind::Initial));
    }

    #[test]
    fn read_option_redraw_values() {
        assert!(SemanticRedraw::parse(b"0") == Some(SemanticRedraw::False));
        assert!(SemanticRedraw::parse(b"1") == Some(SemanticRedraw::True));
        assert!(SemanticRedraw::parse(b"last") == Some(SemanticRedraw::Last));
        assert!(SemanticRedraw::parse(b"x").is_none());
    }

    #[test]
    fn read_option_multiple() {
        let raw = b"aid=foo;cl=line";
        let aid = read_option_aid(raw).unwrap();
        assert!(bytes_eq(aid, b"foo"));
        let cl = read_option_cl(raw).unwrap();
        assert!(cl == SemanticClick::Line);
    }

    #[test]
    fn read_option_aid_with_equals() {
        let raw = b"aid=a=b";
        let aid = read_option_aid(raw).unwrap();
        assert!(bytes_eq(aid, b"a=b"));
    }

    #[test]
    fn read_option_exit_code_negative() {
        let raw = b"-1";
        let code = read_option_exit_code(raw).unwrap();
        assert!(code == -1);
    }

    #[test]
    fn read_option_exit_code_invalid() {
        let raw = b"abc";
        assert!(read_option_exit_code(raw).is_none());
    }

    #[test]
    fn parse_prompt_start_with_kind() {
        let r = parse_semantic_prompt(b"P;k=i").unwrap();
        assert!(r.action == SemanticPromptAction::PromptStart);
        let kind = read_option_prompt_kind(r.options_unvalidated).unwrap();
        assert!(kind == SemanticPromptKind::Initial);
    }
}
