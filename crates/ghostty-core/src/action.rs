//! C-compatible apprt actions (`include/ghostty.h` `ghostty_action_s`).

use core::ffi::c_char;

/// `ghostty_action_tag_e`
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeActionTag {
    Quit = 0,
    NewWindow = 1,
    NewTab = 2,
    CloseTab = 3,
    NewSplit = 4,
    CloseAllWindows = 5,
    ToggleMaximize = 6,
    ToggleFullscreen = 7,
    ToggleTabOverview = 8,
    ToggleWindowDecorations = 9,
    ToggleQuickTerminal = 10,
    ToggleCommandPalette = 11,
    ToggleVisibility = 12,
    ToggleBackgroundOpacity = 13,
    MoveTab = 14,
    GotoTab = 15,
    GotoSplit = 16,
    GotoWindow = 17,
    ResizeSplit = 18,
    EqualizeSplits = 19,
    ToggleSplitZoom = 20,
    PresentTerminal = 21,
    SizeLimit = 22,
    ResetWindowSize = 23,
    InitialSize = 24,
    CellSize = 25,
    Scrollbar = 26,
    Render = 27,
    Inspector = 28,
    ShowGtkInspector = 29,
    RenderInspector = 30,
    DesktopNotification = 31,
    SetTitle = 32,
    SetTabTitle = 33,
    PromptTitle = 34,
    Pwd = 35,
    MouseShape = 36,
    MouseVisibility = 37,
    MouseOverLink = 38,
    RendererHealth = 39,
    OpenConfig = 40,
    QuitTimer = 41,
    FloatWindow = 42,
    SecureInput = 43,
    KeySequence = 44,
    KeyTable = 45,
    ColorChange = 46,
    ReloadConfig = 47,
    ConfigChange = 48,
    CloseWindow = 49,
    RingBell = 50,
    Undo = 51,
    Redo = 52,
    CheckForUpdates = 53,
    OpenUrl = 54,
    ShowChildExited = 55,
    ProgressReport = 56,
    ShowOnScreenKeyboard = 57,
    CommandFinished = 58,
    StartSearch = 59,
    EndSearch = 60,
    SearchTotal = 61,
    SearchSelected = 62,
    Readonly = 63,
    CopyTitleToClipboard = 64,
}

/// `ghostty_action_split_direction_e`
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSplitDirection {
    Right = 0,
    Down = 1,
    Left = 2,
    Up = 3,
}

/// `ghostty_action_set_title_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeSetTitle {
    pub title: *const c_char,
}

/// `ghostty_action_desktop_notification_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeDesktopNotification {
    pub title: *const c_char,
    pub body: *const c_char,
}

/// `ghostty_action_pwd_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimePwd {
    pub pwd: *const c_char,
}

/// `ghostty_action_mouse_over_link_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeMouseOverLink {
    pub url: *const c_char,
    pub len: usize,
}

/// `ghostty_action_size_limit_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeSizeLimit {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
}

/// `ghostty_action_cell_size_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeCellSize {
    pub width: u32,
    pub height: u32,
}

/// `ghostty_action_scrollbar_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeScrollbar {
    pub offset: u64,
    pub len: u64,
}

/// `ghostty_action_key_sequence_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeKeySequence {
    pub sequence: *const c_char,
}

/// `ghostty_action_open_url_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeOpenUrl {
    pub kind: u32,
    pub url: *const c_char,
    pub len: usize,
}

/// `ghostty_action_color_change_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeColorChange {
    pub kind: i32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// `ghostty_action_u`
#[repr(C)]
#[derive(Clone, Copy)]
pub union RuntimeActionUnion {
    pub new_split: RuntimeSplitDirection,
    pub set_title: RuntimeSetTitle,
    pub set_tab_title: RuntimeSetTitle,
    pub desktop_notification: RuntimeDesktopNotification,
    pub pwd: RuntimePwd,
    pub mouse_over_link: RuntimeMouseOverLink,
    pub size_limit: RuntimeSizeLimit,
    pub cell_size: RuntimeCellSize,
    pub scrollbar: RuntimeScrollbar,
    pub key_sequence: RuntimeKeySequence,
    pub open_url: RuntimeOpenUrl,
    pub color_change: RuntimeColorChange,
    pub tag_only: u32,
}

/// `ghostty_action_s`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RuntimeAction {
    pub tag: RuntimeActionTag,
    pub action: RuntimeActionUnion,
}

impl RuntimeAction {
    pub fn with_tag(tag: RuntimeActionTag) -> Self {
        Self {
            tag,
            action: RuntimeActionUnion { tag_only: 0 },
        }
    }

    pub fn ring_bell() -> Self {
        Self::with_tag(RuntimeActionTag::RingBell)
    }

    pub fn present_terminal() -> Self {
        Self::with_tag(RuntimeActionTag::PresentTerminal)
    }

    pub fn set_title_ptr(title: *const c_char) -> Self {
        Self {
            tag: RuntimeActionTag::SetTitle,
            action: RuntimeActionUnion {
                set_title: RuntimeSetTitle { title },
            },
        }
    }

    pub fn color_change(kind: i32, color: ghostty_config::RgbColor) -> Self {
        Self {
            tag: RuntimeActionTag::ColorChange,
            action: RuntimeActionUnion {
                color_change: RuntimeColorChange {
                    kind,
                    r: color.r,
                    g: color.g,
                    b: color.b,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn set_title_layout() {
        assert_eq!(size_of::<RuntimeSetTitle>(), size_of::<*const c_char>());
        assert!(
            size_of::<RuntimeAction>()
                >= size_of::<RuntimeActionTag>() + size_of::<RuntimeActionUnion>()
        );
    }

    #[test]
    fn tag_values_match_header_subset() {
        assert_eq!(RuntimeActionTag::SetTitle as u32, 32);
        assert_eq!(RuntimeActionTag::PresentTerminal as u32, 21);
        assert_eq!(RuntimeActionTag::RingBell as u32, 50);
    }
}
