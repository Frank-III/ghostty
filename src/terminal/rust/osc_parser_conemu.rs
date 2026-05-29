#![allow(unused)]

use crate::bytes_util::bytes_to_str;
use crate::osc_types::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConemuOsc9Result<'a> {
    Sleep { duration_ms: u16 },
    ShowMessageBox(&'a str),
    ChangeTabTitle(ConemuChangeTabTitle<'a>),
    ProgressReport(ProgressReportFull),
    WaitInput,
    Guimacro(&'a str),
    RunProcess(&'a str),
    OutputEnvironmentVariable(&'a str),
    XtermEmulation(ConemuXtermEmulation),
    Comment(&'a str),
    ReportPwd(&'a str),
    SemanticPromptFreshLine,
    DesktopNotification { title: &'a str, body: &'a str },
    Invalid,
}

fn parse_u16_clamped(s: &[u8], max: u16) -> u16 {
    if s.is_empty() {
        return 100;
    }
    let mut result: u32 = 0;
    let mut i = 0;
    while i < s.len() {
        let b = unsafe { *s.get_unchecked(i) };
        if b < b'0' || b > b'9' {
            return 100;
        }
        result = result * 10 + (b - b'0') as u32;
        if result > max as u32 {
            return max;
        }
        i += 1;
    }
    if result == 0 && i > 0 {
        return 0;
    }
    if i == 0 {
        return 100;
    }
    result as u16
}

fn parse_u8_clamped(s: &[u8]) -> Option<u8> {
    if s.is_empty() {
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
    if result > 100 {
        Some(100)
    } else {
        Some(result as u8)
    }
}

pub fn parse_conemu_osc9<'a>(data: &'a [u8]) -> ConemuOsc9Result<'a> {
    if data.is_empty() {
        return ConemuOsc9Result::DesktopNotification {
            title: "",
            body: "",
        };
    }

    let first = unsafe { *data.get_unchecked(0) };

    match first {
        b'1' => {
            if data.len() < 2 {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let second = unsafe { *data.get_unchecked(1) };
            match second {
                b';' => {
                    let rest = unsafe { data.get_unchecked(2..) };
                    let duration = parse_u16_clamped(rest, 10_000);
                    ConemuOsc9Result::Sleep {
                        duration_ms: if duration == 0 { 100 } else { duration },
                    }
                }
                b'0' => {
                    if data.len() == 2 {
                        return ConemuOsc9Result::XtermEmulation(ConemuXtermEmulation {
                            keyboard: Some(true),
                            output: Some(true),
                        });
                    }
                    if data.len() < 4 {
                        return ConemuOsc9Result::DesktopNotification {
                            title: "",
                            body: bytes_to_str(data),
                        };
                    }
                    if unsafe { *data.get_unchecked(2) } != b';' {
                        return ConemuOsc9Result::DesktopNotification {
                            title: "",
                            body: bytes_to_str(data),
                        };
                    }
                    let val = unsafe { *data.get_unchecked(3) };
                    match val {
                        b'0' => ConemuOsc9Result::XtermEmulation(ConemuXtermEmulation {
                            keyboard: Some(false),
                            output: Some(false),
                        }),
                        b'1' => ConemuOsc9Result::XtermEmulation(ConemuXtermEmulation {
                            keyboard: Some(true),
                            output: Some(true),
                        }),
                        b'2' => ConemuOsc9Result::XtermEmulation(ConemuXtermEmulation {
                            keyboard: None,
                            output: Some(false),
                        }),
                        b'3' => ConemuOsc9Result::XtermEmulation(ConemuXtermEmulation {
                            keyboard: None,
                            output: Some(true),
                        }),
                        _ => ConemuOsc9Result::DesktopNotification {
                            title: "",
                            body: bytes_to_str(data),
                        },
                    }
                }
                b'1' => {
                    if data.len() < 3 || unsafe { *data.get_unchecked(2) } != b';' {
                        return ConemuOsc9Result::DesktopNotification {
                            title: "",
                            body: bytes_to_str(data),
                        };
                    }
                    let rest = unsafe { data.get_unchecked(3..) };
                    ConemuOsc9Result::Comment(bytes_to_str(rest))
                }
                b'2' => ConemuOsc9Result::SemanticPromptFreshLine,
                _ => ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                },
            }
        }
        b'2' => {
            if data.len() < 2 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let rest = unsafe { data.get_unchecked(2..) };
            ConemuOsc9Result::ShowMessageBox(bytes_to_str(rest))
        }
        b'3' => {
            if data.len() < 2 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            if data.len() == 2 {
                return ConemuOsc9Result::ChangeTabTitle(ConemuChangeTabTitle::Reset);
            }
            let rest = unsafe { data.get_unchecked(2..) };
            ConemuOsc9Result::ChangeTabTitle(ConemuChangeTabTitle::Value(bytes_to_str(rest)))
        }
        b'4' => {
            if data.len() < 3 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let state_ch = unsafe { *data.get_unchecked(2) };
            let state = match state_ch {
                b'0' => ProgressState::Remove,
                b'1' => ProgressState::Set,
                b'2' => ProgressState::Error,
                b'3' => ProgressState::Indeterminate,
                b'4' => ProgressState::Pause,
                _ => {
                    return ConemuOsc9Result::DesktopNotification {
                        title: "",
                        body: bytes_to_str(data),
                    }
                }
            };

            let mut progress: Option<u8> = None;

            match state {
                ProgressState::Remove | ProgressState::Indeterminate => {}
                ProgressState::Set | ProgressState::Error | ProgressState::Pause => {
                    if data.len() >= 4 && unsafe { *data.get_unchecked(3) } == b';' {
                        let val_str = unsafe { data.get_unchecked(4..) };
                        progress = parse_u8_clamped(val_str);
                    }
                }
            }

            if state == ProgressState::Set && progress.is_none() {
                progress = Some(0);
            }

            ConemuOsc9Result::ProgressReport(ProgressReportFull { state, progress })
        }
        b'5' => ConemuOsc9Result::WaitInput,
        b'6' => {
            if data.len() < 2 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let rest = unsafe { data.get_unchecked(2..) };
            ConemuOsc9Result::Guimacro(bytes_to_str(rest))
        }
        b'7' => {
            if data.len() < 2 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let rest = unsafe { data.get_unchecked(2..) };
            ConemuOsc9Result::RunProcess(bytes_to_str(rest))
        }
        b'8' => {
            if data.len() < 2 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let rest = unsafe { data.get_unchecked(2..) };
            ConemuOsc9Result::OutputEnvironmentVariable(bytes_to_str(rest))
        }
        b'9' => {
            if data.len() < 2 || unsafe { *data.get_unchecked(1) } != b';' {
                return ConemuOsc9Result::DesktopNotification {
                    title: "",
                    body: bytes_to_str(data),
                };
            }
            let rest = unsafe { data.get_unchecked(2..) };
            ConemuOsc9Result::ReportPwd(bytes_to_str(rest))
        }
        _ => ConemuOsc9Result::DesktopNotification {
            title: "",
            body: bytes_to_str(data),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_notification() {
        match parse_conemu_osc9(b"Hello world") {
            ConemuOsc9Result::DesktopNotification { body, .. } => {
                assert!(body == "Hello world");
            }
            _ => panic!("expected DesktopNotification"),
        }
    }

    #[test]
    fn conemu_sleep() {
        match parse_conemu_osc9(b"1;420") {
            ConemuOsc9Result::Sleep { duration_ms } => assert!(duration_ms == 420),
            _ => panic!("expected Sleep"),
        }
    }

    #[test]
    fn conemu_sleep_default() {
        match parse_conemu_osc9(b"1;") {
            ConemuOsc9Result::Sleep { duration_ms } => assert!(duration_ms == 100),
            _ => panic!("expected Sleep"),
        }
    }

    #[test]
    fn conemu_sleep_clamped() {
        match parse_conemu_osc9(b"1;12345") {
            ConemuOsc9Result::Sleep { duration_ms } => assert!(duration_ms == 10000),
            _ => panic!("expected Sleep"),
        }
    }

    #[test]
    fn conemu_message_box() {
        match parse_conemu_osc9(b"2;hello world") {
            ConemuOsc9Result::ShowMessageBox(msg) => assert!(msg == "hello world"),
            _ => panic!("expected ShowMessageBox"),
        }
    }

    #[test]
    fn conemu_tab_title() {
        match parse_conemu_osc9(b"3;foo bar") {
            ConemuOsc9Result::ChangeTabTitle(ConemuChangeTabTitle::Value(v)) => {
                assert!(v == "foo bar");
            }
            _ => panic!("expected ChangeTabTitle"),
        }
    }

    #[test]
    fn conemu_tab_title_reset() {
        match parse_conemu_osc9(b"3;") {
            ConemuOsc9Result::ChangeTabTitle(ConemuChangeTabTitle::Reset) => {}
            _ => panic!("expected Reset"),
        }
    }

    #[test]
    fn conemu_progress_set() {
        match parse_conemu_osc9(b"4;1;100") {
            ConemuOsc9Result::ProgressReport(r) => {
                assert!(r.state == ProgressState::Set);
                assert!(r.progress == Some(100));
            }
            _ => panic!("expected ProgressReport"),
        }
    }

    #[test]
    fn conemu_progress_set_clamped() {
        match parse_conemu_osc9(b"4;1;900") {
            ConemuOsc9Result::ProgressReport(r) => {
                assert!(r.state == ProgressState::Set);
                assert!(r.progress == Some(100));
            }
            _ => panic!("expected ProgressReport"),
        }
    }

    #[test]
    fn conemu_progress_remove() {
        match parse_conemu_osc9(b"4;0") {
            ConemuOsc9Result::ProgressReport(r) => {
                assert!(r.state == ProgressState::Remove);
                assert!(r.progress.is_none());
            }
            _ => panic!("expected ProgressReport"),
        }
    }

    #[test]
    fn conemu_progress_error() {
        match parse_conemu_osc9(b"4;2") {
            ConemuOsc9Result::ProgressReport(r) => {
                assert!(r.state == ProgressState::Error);
            }
            _ => panic!("expected ProgressReport"),
        }
    }

    #[test]
    fn conemu_progress_pause() {
        match parse_conemu_osc9(b"4;4") {
            ConemuOsc9Result::ProgressReport(r) => {
                assert!(r.state == ProgressState::Pause);
            }
            _ => panic!("expected ProgressReport"),
        }
    }

    #[test]
    fn conemu_wait_input() {
        match parse_conemu_osc9(b"5") {
            ConemuOsc9Result::WaitInput => {}
            _ => panic!("expected WaitInput"),
        }
    }

    #[test]
    fn conemu_guimacro() {
        match parse_conemu_osc9(b"6;ab") {
            ConemuOsc9Result::Guimacro(v) => assert!(v == "ab"),
            _ => panic!("expected Guimacro"),
        }
    }

    #[test]
    fn conemu_run_process() {
        match parse_conemu_osc9(b"7;ab") {
            ConemuOsc9Result::RunProcess(v) => assert!(v == "ab"),
            _ => panic!("expected RunProcess"),
        }
    }

    #[test]
    fn conemu_output_env() {
        match parse_conemu_osc9(b"8;ab") {
            ConemuOsc9Result::OutputEnvironmentVariable(v) => assert!(v == "ab"),
            _ => panic!("expected OutputEnvironmentVariable"),
        }
    }

    #[test]
    fn conemu_report_pwd() {
        match parse_conemu_osc9(b"9;ab") {
            ConemuOsc9Result::ReportPwd(v) => assert!(v == "ab"),
            _ => panic!("expected ReportPwd"),
        }
    }

    #[test]
    fn conemu_xterm_emulation_on() {
        match parse_conemu_osc9(b"10") {
            ConemuOsc9Result::XtermEmulation(e) => {
                assert!(e.keyboard == Some(true));
                assert!(e.output == Some(true));
            }
            _ => panic!("expected XtermEmulation"),
        }
    }

    #[test]
    fn conemu_xterm_emulation_off() {
        match parse_conemu_osc9(b"10;0") {
            ConemuOsc9Result::XtermEmulation(e) => {
                assert!(e.keyboard == Some(false));
                assert!(e.output == Some(false));
            }
            _ => panic!("expected XtermEmulation"),
        }
    }

    #[test]
    fn conemu_xterm_emulation_output_only() {
        match parse_conemu_osc9(b"10;3") {
            ConemuOsc9Result::XtermEmulation(e) => {
                assert!(e.keyboard.is_none());
                assert!(e.output == Some(true));
            }
            _ => panic!("expected XtermEmulation"),
        }
    }

    #[test]
    fn conemu_comment() {
        match parse_conemu_osc9(b"11;ab") {
            ConemuOsc9Result::Comment(v) => assert!(v == "ab"),
            _ => panic!("expected Comment"),
        }
    }

    #[test]
    fn conemu_semantic_prompt() {
        match parse_conemu_osc9(b"12") {
            ConemuOsc9Result::SemanticPromptFreshLine => {}
            _ => panic!("expected SemanticPromptFreshLine"),
        }
    }

    #[test]
    fn conemu_sleep_invalid_falls_to_notification() {
        match parse_conemu_osc9(b"1") {
            ConemuOsc9Result::DesktopNotification { body, .. } => {
                assert!(body == "1");
            }
            _ => panic!("expected DesktopNotification"),
        }
    }

    #[test]
    fn conemu_progress_invalid_falls_to_notification() {
        match parse_conemu_osc9(b"4;5") {
            ConemuOsc9Result::DesktopNotification { body, .. } => {
                assert!(body == "4;5");
            }
            _ => panic!("expected DesktopNotification"),
        }
    }
}
