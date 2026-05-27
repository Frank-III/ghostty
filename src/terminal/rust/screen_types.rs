#![allow(unused)]

use core::ffi::c_void;

use crate::early::*;
use crate::constants::*;
use crate::size_types::*;
use crate::page_types::*;
use crate::style_types::*;
use crate::cursor_style::*;
use crate::charsets::*;
use crate::kitty_key::*;
use crate::ansi::*;
use crate::selection_types::*;
use crate::highlight::Pin;
use crate::hyperlink::*;
use crate::allocator::GhosttyAllocator;

#[repr(C)]
pub struct ScreenDirty {
    pub selection: bool,
    pub hyperlink_hover: bool,
}

impl Default for ScreenDirty {
    fn default() -> Self {
        Self {
            selection: false,
            hyperlink_hover: false,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticClickKind {
    Line = 0,
    Multiple = 1,
    ConservativeVertical = 2,
    SmartVertical = 3,
}

impl Default for SemanticClickKind {
    fn default() -> Self {
        Self::Line
    }
}

impl SemanticClickKind {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Line),
            1 => Some(Self::Multiple),
            2 => Some(Self::ConservativeVertical),
            3 => Some(Self::SmartVertical),
            _ => None,
        }
    }
}

pub enum ScreenSemanticClick {
    None,
    ClickEvents,
    Cl(SemanticClickKind),
}

impl Default for ScreenSemanticClick {
    fn default() -> Self {
        Self::None
    }
}

#[repr(C)]
pub struct ScreenSemanticPrompt {
    pub seen: bool,
    pub click: ScreenSemanticClick,
}

impl Default for ScreenSemanticPrompt {
    fn default() -> Self {
        Self {
            seen: false,
            click: ScreenSemanticClick::None,
        }
    }
}

impl ScreenSemanticPrompt {
    pub const DISABLED: Self = Self {
        seen: false,
        click: ScreenSemanticClick::None,
    };
}

#[repr(C)]
pub struct ScreenCursor {
    pub x: CellCountInt,
    pub y: CellCountInt,
    pub cursor_style: CursorVisualStyle,
    pub pending_wrap: bool,
    pub protected: bool,
    pub style: Style,
    pub style_id: StyleId,
    pub hyperlink_id: HyperlinkId,
    pub hyperlink_implicit_id: OffsetInt,
    pub hyperlink: *mut Hyperlink,
    pub semantic_content: SemanticContent,
    pub semantic_content_clear_eol: bool,
    pub page_pin: *mut Pin,
    pub page_row: *mut Row,
    pub page_cell: *mut Cell,
}

impl Default for ScreenCursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            cursor_style: CursorVisualStyle::Block,
            pending_wrap: false,
            protected: false,
            style: Style::default(),
            style_id: DEFAULT_ID,
            hyperlink_id: 0,
            hyperlink_implicit_id: 0,
            hyperlink: core::ptr::null_mut(),
            semantic_content: SemanticContent::Output,
            semantic_content_clear_eol: false,
            page_pin: core::ptr::null_mut(),
            page_row: core::ptr::null_mut(),
            page_cell: core::ptr::null_mut(),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScreenSavedCursor {
    pub x: CellCountInt,
    pub y: CellCountInt,
    pub style: Style,
    pub protected: bool,
    pub pending_wrap: bool,
    pub origin: bool,
    pub charset: ScreenCharsetState,
}

impl Default for ScreenSavedCursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            style: Style::default(),
            protected: false,
            pending_wrap: false,
            origin: false,
            charset: ScreenCharsetState::default(),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScreenCharsetArray {
    pub g0: CharsetId,
    pub g1: CharsetId,
    pub g2: CharsetId,
    pub g3: CharsetId,
}

impl Default for ScreenCharsetArray {
    fn default() -> Self {
        Self {
            g0: CharsetId::Utf8,
            g1: CharsetId::Utf8,
            g2: CharsetId::Utf8,
            g3: CharsetId::Utf8,
        }
    }
}

impl ScreenCharsetArray {
    #[inline]
    pub fn get(&self, slot: CharsetSlot) -> CharsetId {
        match slot {
            CharsetSlot::G0 => self.g0,
            CharsetSlot::G1 => self.g1,
            CharsetSlot::G2 => self.g2,
            CharsetSlot::G3 => self.g3,
        }
    }

    #[inline]
    pub fn set(&mut self, slot: CharsetSlot, charset: CharsetId) {
        match slot {
            CharsetSlot::G0 => self.g0 = charset,
            CharsetSlot::G1 => self.g1 = charset,
            CharsetSlot::G2 => self.g2 = charset,
            CharsetSlot::G3 => self.g3 = charset,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScreenCharsetState {
    pub charsets: ScreenCharsetArray,
    pub gl: CharsetSlot,
    pub gr: CharsetSlot,
    pub single_shift: Option<CharsetSlot>,
}

impl Default for ScreenCharsetState {
    fn default() -> Self {
        Self {
            charsets: ScreenCharsetArray::default(),
            gl: CharsetSlot::G0,
            gr: CharsetSlot::G2,
            single_shift: None,
        }
    }
}

pub enum ScreenScroll {
    Active,
    Top,
    Pin(Pin),
    Row(usize),
    DeltaRow(isize),
    DeltaPrompt(isize),
}

impl Default for ScreenScroll {
    fn default() -> Self {
        Self::Active
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PromptRedraw {
    True = 0,
    False = 1,
    Last = 2,
}

impl Default for PromptRedraw {
    fn default() -> Self {
        Self::False
    }
}

impl PromptRedraw {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::True),
            1 => Some(Self::False),
            2 => Some(Self::Last),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct ScreenResize {
    pub cols: CellCountInt,
    pub rows: CellCountInt,
    pub reflow: bool,
    pub prompt_redraw: PromptRedraw,
}

impl Default for ScreenResize {
    fn default() -> Self {
        Self {
            cols: 0,
            rows: 0,
            reflow: true,
            prompt_redraw: PromptRedraw::False,
        }
    }
}

#[repr(C)]
pub struct ScreenOptions {
    pub cols: CellCountInt,
    pub rows: CellCountInt,
    pub max_scrollback: usize,
    pub kitty_image_storage_limit: usize,
}

impl Default for ScreenOptions {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            max_scrollback: 0,
            kitty_image_storage_limit: 0,
        }
    }
}

#[repr(C)]
pub struct ScreenSelectionString {
    pub sel: Selection,
    pub trim: bool,
    pub map: *mut c_void,
}

impl Default for ScreenSelectionString {
    fn default() -> Self {
        Self {
            sel: Selection::init(
                Pin::default(),
                Pin::default(),
                false,
            ),
            trim: true,
            map: core::ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct ScreenSelectLine {
    pub pin: Pin,
    pub whitespace_ptr: *const u32,
    pub whitespace_len: usize,
    pub semantic_prompt_boundary: bool,
}

impl Default for ScreenSelectLine {
    fn default() -> Self {
        Self {
            pin: Pin::default(),
            whitespace_ptr: core::ptr::null(),
            whitespace_len: 0,
            semantic_prompt_boundary: true,
        }
    }
}

#[repr(C)]
pub struct ScreenLineIterator {
    pub screen: *const c_void,
    pub current: Option<Pin>,
}

impl Default for ScreenLineIterator {
    fn default() -> Self {
        Self {
            screen: core::ptr::null(),
            current: None,
        }
    }
}

#[repr(C)]
pub struct ScreenPromptClickMove {
    pub left: usize,
    pub right: usize,
}

impl Default for ScreenPromptClickMove {
    fn default() -> Self {
        Self { left: 0, right: 0 }
    }
}

impl ScreenPromptClickMove {
    pub const ZERO: Self = Self { left: 0, right: 0 };
}

#[repr(C)]
pub struct Screen {
    pub alloc: GhosttyAllocator,
    pub pages: *mut c_void,
    pub no_scrollback: bool,
    pub cursor: ScreenCursor,
    pub saved_cursor: Option<ScreenSavedCursor>,
    pub selection: Option<Selection>,
    pub charset: ScreenCharsetState,
    pub protected_mode: ProtectedMode,
    pub kitty_keyboard: KittyKeyFlagStack,
    pub kitty_images: *mut c_void,
    pub semantic_prompt: ScreenSemanticPrompt,
    pub dirty: ScreenDirty,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            alloc: GhosttyAllocator::null(),
            pages: core::ptr::null_mut(),
            no_scrollback: false,
            cursor: ScreenCursor::default(),
            saved_cursor: None,
            selection: None,
            charset: ScreenCharsetState::default(),
            protected_mode: ProtectedMode::OFF,
            kitty_keyboard: KittyKeyFlagStack::default(),
            kitty_images: core::ptr::null_mut(),
            semantic_prompt: ScreenSemanticPrompt::DISABLED,
            dirty: ScreenDirty::default(),
        }
    }
}
