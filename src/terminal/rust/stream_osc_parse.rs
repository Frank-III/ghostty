#![allow(unused)]

use crate::osc_types::*;
use crate::stream_types::*;
use crate::vt_parser::{ParserOsc, MAX_OSC_BUF};

#[inline]
fn get_byte(data: &[u8], idx: usize) -> Option<u8> {
    if idx < data.len() {
        Some(unsafe { *data.get_unchecked(idx) })
    } else {
        None
    }
}

#[inline]
fn subslice(data: &[u8], start: usize, end: usize) -> &[u8] {
    if start > end || end > data.len() {
        return &[];
    }
    unsafe { data.get_unchecked(start..end) }
}

#[inline]
fn subslice_from(data: &[u8], start: usize) -> &[u8] {
    if start >= data.len() {
        return &[];
    }
    unsafe { data.get_unchecked(start..data.len()) }
}

#[inline]
fn find_byte(data: &[u8], b: u8) -> Option<usize> {
    let mut i = 0;
    while i < data.len() {
        if unsafe { *data.get_unchecked(i) } == b {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[inline]
fn bytes_eq(data: &[u8], start: usize, needle: &[u8]) -> bool {
    if start + needle.len() > data.len() {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if unsafe { *data.get_unchecked(start + i) } != unsafe { *needle.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

#[inline]
fn parse_u16(data: &[u8]) -> Option<u16> {
    if data.is_empty() {
        return None;
    }
    let mut acc: u16 = 0;
    let mut i = 0;
    while i < data.len() {
        let c = unsafe { *data.get_unchecked(i) };
        if c < b'0' || c > b'9' {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add((c - b'0') as u16)?;
        i += 1;
    }
    Some(acc)
}

#[inline]
fn bytes_to_str(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => "",
    }
}

fn color_op_from_num(num: u16) -> Option<ColorOscOp> {
    match num {
        4 => Some(ColorOscOp::Osc4),
        5 => Some(ColorOscOp::Osc5),
        10 => Some(ColorOscOp::Osc10),
        11 => Some(ColorOscOp::Osc11),
        12 => Some(ColorOscOp::Osc12),
        13 => Some(ColorOscOp::Osc13),
        14 => Some(ColorOscOp::Osc14),
        15 => Some(ColorOscOp::Osc15),
        16 => Some(ColorOscOp::Osc16),
        17 => Some(ColorOscOp::Osc17),
        18 => Some(ColorOscOp::Osc18),
        19 => Some(ColorOscOp::Osc19),
        52 => Some(ColorOscOp::Osc52),
        104 => Some(ColorOscOp::Osc104),
        105 => Some(ColorOscOp::Osc105),
        110 => Some(ColorOscOp::Osc110),
        111 => Some(ColorOscOp::Osc111),
        112 => Some(ColorOscOp::Osc112),
        113 => Some(ColorOscOp::Osc113),
        114 => Some(ColorOscOp::Osc114),
        115 => Some(ColorOscOp::Osc115),
        116 => Some(ColorOscOp::Osc116),
        117 => Some(ColorOscOp::Osc117),
        118 => Some(ColorOscOp::Osc118),
        119 => Some(ColorOscOp::Osc119),
        _ => None,
    }
}

fn parse_hyperlink(payload: &[u8]) -> Command<'_> {
    let semi = find_byte(payload, b';');
    let semi_pos = match semi {
        Some(p) => p,
        None => return Command::Invalid,
    };
    let uri_bytes = subslice_from(payload, semi_pos + 1);
    let uri = bytes_to_str(uri_bytes);
    let options = subslice(payload, 0, semi_pos);

    if uri.is_empty() {
        return Command::HyperlinkEnd;
    }

    let mut id: Option<&str> = None;
    let mut pos = 0;
    while pos < options.len() {
        let next_sep = find_byte(subslice_from(options, pos), b':');
        let kv_end = match next_sep {
            Some(p) => pos + p,
            None => options.len(),
        };
        let kv = subslice(options, pos, kv_end);
        let eq = find_byte(kv, b'=');
        if let Some(eq_pos) = eq {
            let key = subslice(kv, 0, eq_pos);
            let val = subslice_from(kv, eq_pos + 1);
            if key.len() == 2
                && unsafe { *key.get_unchecked(0) } == b'i'
                && unsafe { *key.get_unchecked(1) } == b'd'
                && !val.is_empty()
            {
                id = Some(bytes_to_str(val));
            }
        }
        pos = kv_end + 1;
        if next_sep.is_none() {
            break;
        }
    }

    Command::HyperlinkStart(HyperlinkStart { uri, id })
}

fn parse_clipboard(payload: &[u8]) -> Command<'_> {
    if payload.is_empty() {
        return Command::Invalid;
    }
    let first = unsafe { *payload.get_unchecked(0) };
    if first == b';' {
        let data_str = bytes_to_str(subslice_from(payload, 1));
        return Command::ClipboardContents(ClipboardContentsOsc {
            kind: b'c',
            data: data_str,
        });
    }
    if payload.len() < 2 {
        return Command::Invalid;
    }
    let second = unsafe { *payload.get_unchecked(1) };
    if second != b';' {
        return Command::Invalid;
    }
    let data_str = bytes_to_str(subslice_from(payload, 2));
    Command::ClipboardContents(ClipboardContentsOsc {
        kind: first,
        data: data_str,
    })
}

fn parse_semantic_prompt(payload: &[u8]) -> Command<'_> {
    if payload.is_empty() {
        return Command::Invalid;
    }
    let action_byte = unsafe { *payload.get_unchecked(0) };
    let action = match action_byte {
        b'L' => {
            if payload.len() != 1 {
                return Command::Invalid;
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
        _ => return Command::Invalid,
    };

    let mut options_unvalidated: &[u8] = &[];
    if payload.len() > 1 {
        let second = unsafe { *payload.get_unchecked(1) };
        if second != b';' {
            return Command::Invalid;
        }
        options_unvalidated = subslice_from(payload, 2);
    }

    Command::SemanticPrompt(SemanticPrompt {
        action,
        options_unvalidated,
    })
}

fn parse_osc777(payload: &[u8]) -> Command<'_> {
    let semi1 = find_byte(payload, b';');
    let semi1_pos = match semi1 {
        Some(p) => p,
        None => return Command::Invalid,
    };
    let ext = subslice(payload, 0, semi1_pos);
    if ext.len() != 6 {
        return Command::Invalid;
    }
    if !bytes_eq(ext, 0, b"notify") {
        return Command::Invalid;
    }
    let rest = subslice_from(payload, semi1_pos + 1);
    let semi2 = find_byte(rest, b';');
    let semi2_pos = match semi2 {
        Some(p) => p,
        None => return Command::Invalid,
    };
    let title = bytes_to_str(subslice(rest, 0, semi2_pos));
    let body = bytes_to_str(subslice_from(rest, semi2_pos + 1));
    Command::ShowDesktopNotification(ShowDesktopNotificationOsc { title, body })
}

fn parse_osc9_notification(payload: &[u8]) -> Command<'_> {
    let title = "";
    let body = bytes_to_str(payload);
    Command::ShowDesktopNotification(ShowDesktopNotificationOsc { title, body })
}

fn parse_conemu_sleep(payload: &[u8]) -> Command<'_> {
    let mut acc: u16 = 0;
    let mut i = 0;
    let mut has_digit = false;
    while i < payload.len() {
        let c = unsafe { *payload.get_unchecked(i) };
        if c < b'0' || c > b'9' {
            break;
        }
        has_digit = true;
        acc = match acc.checked_mul(10) {
            Some(v) => v,
            None => return Command::ConemuSleep { duration_ms: 100 },
        };
        acc = match acc.checked_add((c - b'0') as u16) {
            Some(v) => v,
            None => return Command::ConemuSleep { duration_ms: 100 },
        };
        i += 1;
    }
    let ms = if has_digit && acc <= 10_000 { acc } else if has_digit { 10_000 } else { 100 };
    Command::ConemuSleep { duration_ms: ms }
}

fn parse_conemu_progress(payload: &[u8]) -> Command<'_> {
    if payload.is_empty() {
        return Command::Invalid;
    }
    let state_byte = unsafe { *payload.get_unchecked(0) };
    let state = match state_byte {
        b'0' => ProgressState::Remove,
        b'1' => ProgressState::Set,
        b'2' => ProgressState::Error,
        b'3' => ProgressState::Indeterminate,
        b'4' => ProgressState::Pause,
        _ => return Command::Invalid,
    };
    let mut progress: Option<u8> = None;
    match state {
        ProgressState::Set | ProgressState::Error | ProgressState::Pause => {
            if payload.len() >= 3 {
                let sep = unsafe { *payload.get_unchecked(1) };
                if sep == b';' {
                    let val_bytes = subslice_from(payload, 2);
                    if let Some(v) = parse_u16(val_bytes) {
                        let clamped = if v > 100 { 100u8 } else { v as u8 };
                        progress = Some(clamped);
                    }
                }
            }
        }
        _ => {}
    }
    Command::ConemuProgressReport(ProgressReportFull { state, progress })
}

fn parse_conemu_xterm_emulation(payload: &[u8]) -> Command<'_> {
    if payload.is_empty() {
        return Command::ConemuXtermEmulation(ConemuXtermEmulation {
            keyboard: Some(true),
            output: Some(true),
        });
    }
    let first = unsafe { *payload.get_unchecked(0) };
    match first {
        b'0' => Command::ConemuXtermEmulation(ConemuXtermEmulation {
            keyboard: Some(false),
            output: Some(false),
        }),
        b'1' => Command::ConemuXtermEmulation(ConemuXtermEmulation {
            keyboard: Some(true),
            output: Some(true),
        }),
        b'2' => Command::ConemuXtermEmulation(ConemuXtermEmulation {
            keyboard: None,
            output: Some(false),
        }),
        b'3' => Command::ConemuXtermEmulation(ConemuXtermEmulation {
            keyboard: None,
            output: Some(true),
        }),
        _ => Command::Invalid,
    }
}

fn parse_osc9(payload: &[u8]) -> Command<'_> {
    if payload.is_empty() {
        return parse_osc9_notification(payload);
    }
    let first = unsafe { *payload.get_unchecked(0) };
    match first {
        b'1' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                match second {
                    b';' => return parse_conemu_sleep(subslice_from(payload, 2)),
                    b'0' => {
                        if payload.len() == 2 {
                            return parse_conemu_xterm_emulation(&[]);
                        }
                        if payload.len() >= 4 {
                            let third = unsafe { *payload.get_unchecked(2) };
                            if third == b';' {
                                let rest = subslice_from(payload, 3);
                                return parse_conemu_xterm_emulation(rest);
                            }
                        }
                    }
                    b'1' => {
                        if payload.len() >= 3 {
                            let third = unsafe { *payload.get_unchecked(2) };
                            if third == b';' {
                                let val = bytes_to_str(subslice_from(payload, 3));
                                return Command::ConemuComment(val);
                            }
                        }
                    }
                    b'2' => {
                        return Command::SemanticPrompt(SemanticPrompt {
                            action: SemanticPromptAction::FreshLineNewPrompt,
                            options_unvalidated: &[],
                        });
                    }
                    _ => {}
                }
            }
            parse_osc9_notification(payload)
        }
        b'2' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    let val = bytes_to_str(subslice_from(payload, 2));
                    return Command::ConemuShowMessageBox(val);
                }
            }
            parse_osc9_notification(payload)
        }
        b'3' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    let val = bytes_to_str(subslice_from(payload, 2));
                    if val.is_empty() {
                        return Command::ConemuChangeTabTitle(ConemuChangeTabTitle::Reset);
                    }
                    return Command::ConemuChangeTabTitle(ConemuChangeTabTitle::Value(val));
                }
            }
            parse_osc9_notification(payload)
        }
        b'4' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    return parse_conemu_progress(subslice_from(payload, 2));
                }
            }
            parse_osc9_notification(payload)
        }
        b'5' => Command::ConemuWaitInput,
        b'6' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    let val = bytes_to_str(subslice_from(payload, 2));
                    return Command::ConemuGuimacro(val);
                }
            }
            parse_osc9_notification(payload)
        }
        b'7' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    let val = bytes_to_str(subslice_from(payload, 2));
                    return Command::ConemuRunProcess(val);
                }
            }
            parse_osc9_notification(payload)
        }
        b'8' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    let val = bytes_to_str(subslice_from(payload, 2));
                    return Command::ConemuOutputEnvironmentVariable(val);
                }
            }
            parse_osc9_notification(payload)
        }
        b'9' => {
            if payload.len() >= 2 {
                let second = unsafe { *payload.get_unchecked(1) };
                if second == b';' {
                    let val = bytes_to_str(subslice_from(payload, 2));
                    return Command::ReportPwd(ReportPwdOsc { value: val });
                }
            }
            parse_osc9_notification(payload)
        }
        _ => parse_osc9_notification(payload),
    }
}

#[inline]
pub(crate) fn parse_osc_number(data: &[u8]) -> Option<u16> {
    parse_u16(data)
}

pub fn parse(osc: &ParserOsc) -> Command<'_> {
    let len = (osc.data_len as usize).min(MAX_OSC_BUF);
    if len == 0 {
        return Command::Invalid;
    }

    let data: &[u8] = &osc.data[..len];

    let semi = find_byte(data, b';');
    let num_end = match semi {
        Some(p) => p,
        None => len,
    };

    let num_bytes = subslice(data, 0, num_end);
    let osc_num = match parse_u16(num_bytes) {
        Some(n) => n,
        None => return Command::Invalid,
    };

    let payload = match semi {
        Some(p) => subslice_from(data, p + 1),
        None => &[],
    };

    let terminator = if osc.terminator == 0x07 {
        OscTerminator::Bel
    } else {
        OscTerminator::St
    };

    match osc_num {
        0 | 2 => Command::Invalid,
        1 => Command::ChangeWindowIcon(bytes_to_str(payload)),
        4 | 5 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 104 | 105 | 110 | 111
        | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 => {
            let op = match color_op_from_num(osc_num) {
                Some(op) => op,
                None => return Command::Invalid,
            };
            Command::ColorOperation(ColorOperationOsc {
                op,
                requests: payload,
                terminator,
            })
        }
        7 => Command::ReportPwd(ReportPwdOsc {
            value: bytes_to_str(payload),
        }),
        8 => parse_hyperlink(payload),
        9 => parse_osc9(payload),
        21 => Command::KittyColorProtocol(KittyColorOsc { kind: 0 }),
        22 => Command::MouseShape(MouseShapeOsc {
            value: bytes_to_str(payload),
        }),
        52 => parse_clipboard(payload),
        66 => Command::KittyTextSizing,
        133 => parse_semantic_prompt(payload),
        777 => parse_osc777(payload),
        1337 => Command::Invalid,
        3008 => Command::ContextSignal,
        5522 => Command::KittyClipboardProtocol,
        _ => Command::Invalid,
    }
}
