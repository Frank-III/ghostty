use core::ffi::c_void;

use crate::ansi::StatusDisplay;
#[cfg(ghostty_vt_terminal_owned)]
use crate::allocator::GhosttyAllocator;
use crate::color_palette::{default_palette, DynamicPalette};
use crate::mode_def::ModeState;
use crate::mouse_shape::MouseShape;
use crate::screen_set::ScreenSet;
use crate::size_types::CellCountInt;
use crate::style::GhosttyColorRgb;
use crate::tabstops::Tabstops;

pub(crate) const TABSTOP_INTERVAL: CellCountInt = 8;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OptionalRgb {
    pub rgb: GhosttyColorRgb,
    pub set: bool,
}

impl OptionalRgb {
    pub const UNSET: OptionalRgb = OptionalRgb {
        rgb: GhosttyColorRgb { r: 0, g: 0, b: 0 },
        set: false,
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DynamicRgb {
    pub override_val: OptionalRgb,
    pub default_val: OptionalRgb,
}

impl DynamicRgb {
    pub const UNSET: DynamicRgb = DynamicRgb {
        override_val: OptionalRgb::UNSET,
        default_val: OptionalRgb::UNSET,
    };

    pub fn get(&self) -> Option<GhosttyColorRgb> {
        if self.override_val.set {
            return Some(self.override_val.rgb);
        }
        if self.default_val.set {
            return Some(self.default_val.rgb);
        }
        None
    }

    pub fn default_color(&self) -> Option<GhosttyColorRgb> {
        if self.default_val.set {
            Some(self.default_val.rgb)
        } else {
            None
        }
    }

    pub fn set_default(&mut self, color: Option<GhosttyColorRgb>) {
        self.default_val = match color {
            Some(rgb) => OptionalRgb {
                rgb,
                set: true,
            },
            None => OptionalRgb::UNSET,
        };
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TerminalColors {
    pub background: DynamicRgb,
    pub foreground: DynamicRgb,
    pub cursor: DynamicRgb,
    pub palette: DynamicPalette,
}

impl TerminalColors {
    pub fn default_val() -> Self {
        TerminalColors {
            background: DynamicRgb::UNSET,
            foreground: DynamicRgb::UNSET,
            cursor: DynamicRgb::UNSET,
            palette: DynamicPalette::init(default_palette()),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TerminalDirty {
    pub palette: bool,
    pub reverse_colors: bool,
    pub clear: bool,
    pub preedit: bool,
}

impl Default for TerminalDirty {
    fn default() -> Self {
        TerminalDirty {
            palette: false,
            reverse_colors: false,
            clear: false,
            preedit: false,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseEvent {
    None = 0,
    X10 = 1,
    Normal = 2,
    Button = 3,
    Any = 4,
}

impl Default for MouseEvent {
    fn default() -> Self {
        MouseEvent::None
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseFormat {
    X10 = 0,
    Utf8 = 1,
    Sgr = 2,
    Urxvt = 3,
    SgrPixels = 4,
}

impl Default for MouseFormat {
    fn default() -> Self {
        MouseFormat::X10
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseShiftCapture {
    Null = 0,
    False = 1,
    True = 2,
}

impl Default for MouseShiftCapture {
    fn default() -> Self {
        MouseShiftCapture::Null
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShellRedraw {
    True = 0,
    False = 1,
    Last = 2,
}

impl Default for ShellRedraw {
    fn default() -> Self {
        ShellRedraw::True
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TerminalFlags {
    pub shell_redraws_prompt: ShellRedraw,
    pub modify_other_keys_2: bool,
    pub mouse_event: MouseEvent,
    pub mouse_format: MouseFormat,
    pub mouse_shift_capture: MouseShiftCapture,
    pub focused: bool,
    pub password_input: bool,
    pub selection_scroll: bool,
    pub search_viewport_dirty: bool,
    pub dirty: TerminalDirty,
}

impl Default for TerminalFlags {
    fn default() -> Self {
        TerminalFlags {
            shell_redraws_prompt: ShellRedraw::default(),
            modify_other_keys_2: false,
            mouse_event: MouseEvent::default(),
            mouse_format: MouseFormat::default(),
            mouse_shift_capture: MouseShiftCapture::default(),
            focused: true,
            password_input: false,
            selection_scroll: false,
            search_viewport_dirty: false,
            dirty: TerminalDirty::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TerminalScrollingRegion {
    pub top: CellCountInt,
    pub bottom: CellCountInt,
    pub left: CellCountInt,
    pub right: CellCountInt,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SemanticPrompt {
    Prompt = 0,
    PromptContinuation = 1,
    Input = 2,
    Command = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScrollViewportTag {
    Top = 0,
    Bottom = 1,
    Delta = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ScrollViewport {
    pub tag: ScrollViewportTag,
    pub delta: isize,
}

impl ScrollViewport {
    pub fn top() -> Self {
        ScrollViewport {
            tag: ScrollViewportTag::Top,
            delta: 0,
        }
    }

    pub fn bottom() -> Self {
        ScrollViewport {
            tag: ScrollViewportTag::Bottom,
            delta: 0,
        }
    }

    pub fn delta(d: isize) -> Self {
        ScrollViewport {
            tag: ScrollViewportTag::Delta,
            delta: d,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeccolmMode {
    Cols80 = 0,
    Cols132 = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SwitchScreenMode {
    Mode47 = 0,
    Mode1047 = 1,
    Mode1049 = 2,
}

#[repr(C)]
pub struct TerminalOptions {
    pub cols: CellCountInt,
    pub rows: CellCountInt,
    pub max_scrollback: usize,
    pub colors: TerminalColors,
    pub default_modes: u64,
    pub kitty_image_storage_limit: usize,
}

impl TerminalOptions {
    pub fn new(cols: CellCountInt, rows: CellCountInt) -> Self {
        TerminalOptions {
            cols,
            rows,
            max_scrollback: 10_000,
            colors: TerminalColors::default_val(),
            default_modes: 0,
            kitty_image_storage_limit: 320 * 1000 * 1000,
        }
    }
}

#[repr(C)]
pub struct Terminal {
    pub screens: ScreenSet,
    pub status_display: StatusDisplay,
    pub tabstops: Tabstops,
    pub rows: CellCountInt,
    pub cols: CellCountInt,
    pub width_px: u32,
    pub height_px: u32,
    pub scrolling_region: TerminalScrollingRegion,
    pub pwd: *mut c_void,
    pub title: *mut c_void,
    #[cfg(ghostty_vt_terminal_owned)]
    /// Allocator used for title/pwd buffers on the Rust-owned bootstrap path.
    pub bootstrap_alloc: *const GhosttyAllocator,
    #[cfg(ghostty_vt_terminal_owned)]
    /// C `TerminalWrapper` for effect callbacks (write_pty, bell, etc.).
    pub effects_wrapper: *mut c_void,
    pub colors: TerminalColors,
    pub previous_char: u32,
    pub has_previous_char: bool,
    pub modes: ModeState,
    pub mouse_shape: MouseShape,
    pub flags: TerminalFlags,
}

impl Default for TerminalScrollingRegion {
    fn default() -> Self {
        TerminalScrollingRegion {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            screens: ScreenSet::default(),
            status_display: StatusDisplay::default(),
            tabstops: Tabstops::default(),
            rows: 0,
            cols: 0,
            width_px: 0,
            height_px: 0,
            scrolling_region: TerminalScrollingRegion::default(),
            pwd: core::ptr::null_mut(),
            title: core::ptr::null_mut(),
            #[cfg(ghostty_vt_terminal_owned)]
            bootstrap_alloc: core::ptr::null_mut(),
            #[cfg(ghostty_vt_terminal_owned)]
            effects_wrapper: core::ptr::null_mut(),
            colors: TerminalColors::default_val(),
            previous_char: 0,
            has_previous_char: false,
            modes: ModeState::default(),
            mouse_shape: MouseShape::default(),
            flags: TerminalFlags::default(),
        }
    }
}
