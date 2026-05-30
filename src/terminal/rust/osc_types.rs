#![allow(unused)]

use crate::stream_types::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressState {
    Remove = 0,
    Set,
    Error,
    Indeterminate,
    Pause,
}

impl ProgressState {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        if v <= 4 {
            Some(unsafe { core::mem::transmute(v) })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressReportFull {
    pub state: ProgressState,
    pub progress: Option<u8>,
}

impl Default for ProgressReportFull {
    fn default() -> Self {
        Self {
            state: ProgressState::Set,
            progress: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConemuXtermEmulation {
    pub keyboard: Option<bool>,
    pub output: Option<bool>,
}

impl Default for ConemuXtermEmulation {
    fn default() -> Self {
        Self {
            keyboard: None,
            output: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConemuChangeTabTitle<'a> {
    Reset,
    Value(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KittyColorOsc<'a> {
    pub requests: &'a [u8],
    pub terminator: OscTerminator,
}

impl<'a> Default for KittyColorOsc<'a> {
    fn default() -> Self {
        Self {
            requests: &[],
            terminator: OscTerminator::St,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HyperlinkStart<'a> {
    pub uri: &'a str,
    pub id: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardContentsOsc<'a> {
    pub kind: u8,
    pub data: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowDesktopNotificationOsc<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportPwdOsc<'a> {
    pub value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseShapeOsc<'a> {
    pub value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorOperationOsc<'a> {
    pub op: ColorOscOp,
    pub requests: &'a [u8],
    pub terminator: OscTerminator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command<'a> {
    Invalid,
    ChangeWindowTitle(&'a str),
    ChangeWindowIcon(&'a str),
    SemanticPrompt(SemanticPrompt<'a>),
    ClipboardContents(ClipboardContentsOsc<'a>),
    ReportPwd(ReportPwdOsc<'a>),
    MouseShape(MouseShapeOsc<'a>),
    ColorOperation(ColorOperationOsc<'a>),
    KittyColorProtocol(KittyColorOsc<'a>),
    ShowDesktopNotification(ShowDesktopNotificationOsc<'a>),
    HyperlinkStart(HyperlinkStart<'a>),
    HyperlinkEnd,
    ConemuSleep { duration_ms: u16 },
    ConemuShowMessageBox(&'a str),
    ConemuChangeTabTitle(ConemuChangeTabTitle<'a>),
    ConemuProgressReport(ProgressReportFull),
    ConemuWaitInput,
    ConemuGuimacro(&'a str),
    ConemuRunProcess(&'a str),
    ConemuOutputEnvironmentVariable(&'a str),
    ConemuXtermEmulation(ConemuXtermEmulation),
    ConemuComment(&'a str),
    KittyTextSizing,
    KittyClipboardProtocol,
    ContextSignal,
}

impl<'a> Default for Command<'a> {
    fn default() -> Self {
        Command::Invalid
    }
}

pub const TERMINATOR_STRING_ST: &[u8] = b"\x1b\\";
pub const TERMINATOR_STRING_BEL: &[u8] = b"\x07";

#[inline]
pub fn terminator_string(t: OscTerminator) -> &'static [u8] {
    match t {
        OscTerminator::St => TERMINATOR_STRING_ST,
        OscTerminator::Bel => TERMINATOR_STRING_BEL,
    }
}

#[inline]
pub fn terminator_from_ch(ch: Option<u8>) -> OscTerminator {
    match ch {
        Some(0x07) => OscTerminator::Bel,
        _ => OscTerminator::St,
    }
}
