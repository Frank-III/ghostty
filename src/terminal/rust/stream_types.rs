use crate::constants::*;
use crate::early::*;

use crate::ansi::{CursorStyle, ModifyKeyFormat, StatusDisplay};
use crate::charsets::{ActiveSlot, CharsetId, CharsetSlot};
use crate::csi::SizeReportStyle;
use crate::device_attributes::DeviceAttributeReq;
use crate::mode_def::ModeTag;
use crate::mouse_shape::MouseShape;
use crate::vt_parser::ParserDcs;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamActionKey {
    Print = 0,
    PrintRepeat,
    Bell,
    Backspace,
    HorizontalTab,
    HorizontalTabBack,
    Linefeed,
    CarriageReturn,
    Enquiry,
    InvokeCharset,
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    CursorCol,
    CursorRow,
    CursorColRelative,
    CursorRowRelative,
    CursorPos,
    CursorStyle,
    EraseDisplayBelow,
    EraseDisplayAbove,
    EraseDisplayComplete,
    EraseDisplayScrollback,
    EraseDisplayScrollComplete,
    EraseLineRight,
    EraseLineLeft,
    EraseLineComplete,
    EraseLineRightUnlessPendingWrap,
    DeleteChars,
    EraseChars,
    InsertLines,
    InsertBlanks,
    DeleteLines,
    ScrollUp,
    ScrollDown,
    TabClearCurrent,
    TabClearAll,
    TabSet,
    TabReset,
    Index,
    NextLine,
    ReverseIndex,
    FullReset,
    SetMode,
    ResetMode,
    SaveMode,
    RestoreMode,
    RequestMode,
    RequestModeUnknown,
    TopAndBottomMargin,
    LeftAndRightMargin,
    LeftAndRightMarginAmbiguous,
    SaveCursor,
    RestoreCursor,
    ModifyKeyFormat,
    MouseShiftCapture,
    ProtectedModeOff,
    ProtectedModeIso,
    ProtectedModeDec,
    SizeReport,
    TitlePush,
    TitlePop,
    Xtversion,
    DeviceAttributes,
    DeviceStatus,
    KittyKeyboardQuery,
    KittyKeyboardPush,
    KittyKeyboardPop,
    KittyKeyboardSet,
    KittyKeyboardSetOr,
    KittyKeyboardSetNot,
    DcsHook,
    DcsPut,
    DcsUnhook,
    ApcStart,
    ApcEnd,
    ApcPut,
    EndHyperlink,
    ActiveStatusDisplay,
    Decaln,
    WindowTitle,
    ReportPwd,
    ShowDesktopNotification,
    ProgressReport,
    StartHyperlink,
    ClipboardContents,
    MouseShape,
    ConfigureCharset,
    SetAttribute,
    KittyColorReport,
    ColorOperation,
    SemanticPrompt,
}

pub const STREAM_ACTION_KEY_COUNT: usize = 93;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Print {
    pub cp: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvokeCharset {
    pub bank: ActiveSlot,
    pub charset: CharsetSlot,
    pub locking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorMovement {
    pub value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceStatus {
    pub request: u16,
    pub question: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    pub mode: ModeTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawMode {
    pub mode: u16,
    pub ansi: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Margin {
    pub top_left: u16,
    pub bottom_right: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KittyKeyboardFlags {
    pub flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowTitle<'a> {
    pub title: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportPwd<'a> {
    pub url: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowDesktopNotification<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartHyperlink<'a> {
    pub uri: &'a str,
    pub id: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardContents<'a> {
    pub kind: u8,
    pub data: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigureCharset {
    pub slot: CharsetSlot,
    pub charset: CharsetId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorOperation<'a> {
    pub op: ColorOscOp,
    pub requests: &'a [u8],
    pub terminator: OscTerminator,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOscOp {
    Osc4 = 0,
    Osc5,
    Osc10,
    Osc11,
    Osc12,
    Osc13,
    Osc14,
    Osc15,
    Osc16,
    Osc17,
    Osc18,
    Osc19,
    Osc21,
    Osc52,
    Osc104,
    Osc105,
    Osc110,
    Osc111,
    Osc112,
    Osc113,
    Osc114,
    Osc115,
    Osc116,
    Osc117,
    Osc118,
    Osc119,
    Osc121,
}

impl Default for ColorOscOp {
    fn default() -> Self {
        ColorOscOp::Osc4
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscTerminator {
    St = 0,
    Bel = 1,
}

impl Default for OscTerminator {
    fn default() -> Self {
        OscTerminator::St
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticPromptAction {
    FreshLine = 0,
    FreshLineNewPrompt,
    NewCommand,
    PromptStart,
    EndPromptStartInput,
    EndPromptStartInputTerminateEol,
    EndInputStartOutput,
    EndCommand,
}

impl Default for SemanticPromptAction {
    fn default() -> Self {
        SemanticPromptAction::FreshLine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticPrompt<'a> {
    pub action: SemanticPromptAction,
    pub options_unvalidated: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressReport {
    pub progress: Option<u8>,
}

impl Default for ProgressReport {
    fn default() -> Self {
        ProgressReport { progress: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KittyColorReport<'a> {
    pub requests: &'a [u8],
    pub terminator: OscTerminator,
}

impl<'a> Default for KittyColorReport<'a> {
    fn default() -> Self {
        KittyColorReport {
            requests: &[],
            terminator: OscTerminator::St,
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SgrAttributeTag {
    None = 0,
    Bold = 1,
    Dim = 2,
    Italic = 3,
    Underline = 4,
    Blink = 5,
    RapidBlink = 6,
    Reverse = 7,
    Hidden = 8,
    Strikethrough = 9,
    DoubleUnderline = 21,
    FgColor = 30,
    BgColor = 40,
    FgColorBright = 90,
    BgColorBright = 100,
    FgColorReset = 39,
    BgColorReset = 49,
    UnderlineColor = 58,
    UnderlineColorReset = 59,
    Overline = 53,
    Font = 10,
}

impl Default for SgrAttributeTag {
    fn default() -> Self {
        SgrAttributeTag::None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SgrAttribute {
    pub tag: SgrAttributeTag,
    pub value: u32,
}

impl Default for SgrAttribute {
    fn default() -> Self {
        SgrAttribute {
            tag: SgrAttributeTag::None,
            value: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StreamAction<'a> {
    Print(Print),
    PrintRepeat(usize),
    Bell,
    Backspace,
    HorizontalTab(u16),
    HorizontalTabBack(u16),
    Linefeed,
    CarriageReturn,
    Enquiry,
    InvokeCharset(InvokeCharset),
    CursorUp(CursorMovement),
    CursorDown(CursorMovement),
    CursorLeft(CursorMovement),
    CursorRight(CursorMovement),
    CursorCol(CursorMovement),
    CursorRow(CursorMovement),
    CursorColRelative(CursorMovement),
    CursorRowRelative(CursorMovement),
    CursorPos(CursorPos),
    CursorStyle(CursorStyle),
    EraseDisplayBelow(bool),
    EraseDisplayAbove(bool),
    EraseDisplayComplete(bool),
    EraseDisplayScrollback(bool),
    EraseDisplayScrollComplete(bool),
    EraseLineRight(bool),
    EraseLineLeft(bool),
    EraseLineComplete(bool),
    EraseLineRightUnlessPendingWrap(bool),
    DeleteChars(usize),
    EraseChars(usize),
    InsertLines(usize),
    InsertBlanks(usize),
    DeleteLines(usize),
    ScrollUp(usize),
    ScrollDown(usize),
    TabClearCurrent,
    TabClearAll,
    TabSet,
    TabReset,
    Index,
    NextLine,
    ReverseIndex,
    FullReset,
    SetMode(Mode),
    ResetMode(Mode),
    SaveMode(Mode),
    RestoreMode(Mode),
    RequestMode(Mode),
    RequestModeUnknown(RawMode),
    TopAndBottomMargin(Margin),
    LeftAndRightMargin(Margin),
    LeftAndRightMarginAmbiguous,
    SaveCursor,
    RestoreCursor,
    ModifyKeyFormat(ModifyKeyFormat),
    MouseShiftCapture(bool),
    ProtectedModeOff,
    ProtectedModeIso,
    ProtectedModeDec,
    SizeReport(SizeReportStyle),
    TitlePush(u16),
    TitlePop(u16),
    Xtversion,
    DeviceAttributes(DeviceAttributeReq),
    DeviceStatus(DeviceStatus),
    KittyKeyboardQuery,
    KittyKeyboardPush(KittyKeyboardFlags),
    KittyKeyboardPop(u16),
    KittyKeyboardSet(KittyKeyboardFlags),
    KittyKeyboardSetOr(KittyKeyboardFlags),
    KittyKeyboardSetNot(KittyKeyboardFlags),
    DcsHook(ParserDcs),
    DcsPut(u8),
    DcsUnhook,
    ApcStart,
    ApcEnd,
    ApcPut(u8),
    EndHyperlink,
    ActiveStatusDisplay(StatusDisplay),
    Decaln,
    WindowTitle(WindowTitle<'a>),
    ReportPwd(ReportPwd<'a>),
    ShowDesktopNotification(ShowDesktopNotification<'a>),
    ProgressReport(ProgressReport),
    StartHyperlink(StartHyperlink<'a>),
    ClipboardContents(ClipboardContents<'a>),
    MouseShape(MouseShape),
    ConfigureCharset(ConfigureCharset),
    SetAttribute(SgrAttribute),
    KittyColorReport(KittyColorReport<'a>),
    ColorOperation(ColorOperation<'a>),
    SemanticPrompt(SemanticPrompt<'a>),
}

impl StreamActionKey {
    pub fn from_u8(v: u8) -> Option<Self> {
        if (v as usize) < STREAM_ACTION_KEY_COUNT {
            Some(unsafe { core::mem::transmute(v) })
        } else {
            None
        }
    }
}
