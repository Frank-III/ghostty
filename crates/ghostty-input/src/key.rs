use crate::key_mods::Mods;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Action {
    Release,
    #[default]
    Press,
    Repeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Key {
    #[default]
    Unidentified,
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    MetaLeft,
    MetaRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    Convert,
    KanaMode,
    NonConvert,
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadSubtract,
    NumpadSeparator,
    NumpadUp,
    NumpadDown,
    NumpadRight,
    NumpadLeft,
    NumpadBegin,
    NumpadHome,
    NumpadEnd,
    NumpadInsert,
    NumpadDelete,
    NumpadPageUp,
    NumpadPageDown,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Fn,
    FnLock,
    PrintScreen,
    ScrollLock,
    Pause,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    Copy,
    Cut,
    Paste,
}

impl Key {
    pub fn from_ascii(ch: u8) -> Option<Self> {
        for &(cp, key) in Self::CODEPOINT_MAP.iter() {
            if key.keypad() {
                continue;
            }
            if cp == ch as u32 {
                return Some(key);
            }
        }
        None
    }

    pub fn from_w3c(code: &str) -> Option<Self> {
        let mut result = String::with_capacity(code.len());
        let mut prev_was_boundary = true;
        for ch in code.chars() {
            if ch.is_ascii_uppercase() || ch.is_ascii_digit() {
                if !prev_was_boundary && !result.is_empty() {
                    result.push('_');
                }
                result.push(ch.to_ascii_lowercase());
                prev_was_boundary = true;
            } else if ch.is_ascii_lowercase() {
                result.push(ch);
                prev_was_boundary = false;
            } else {
                return None;
            }
        }
        Self::from_w3c_string(&result)
    }

    pub fn w3c(self) -> String {
        let name = match self {
            Self::Unidentified => "unidentified",
            Self::Backquote => "backquote",
            Self::Backslash => "backslash",
            Self::BracketLeft => "bracket_left",
            Self::BracketRight => "bracket_right",
            Self::Comma => "comma",
            Self::Digit0 => "digit_0",
            Self::Digit1 => "digit_1",
            Self::Digit2 => "digit_2",
            Self::Digit3 => "digit_3",
            Self::Digit4 => "digit_4",
            Self::Digit5 => "digit_5",
            Self::Digit6 => "digit_6",
            Self::Digit7 => "digit_7",
            Self::Digit8 => "digit_8",
            Self::Digit9 => "digit_9",
            Self::Equal => "equal",
            Self::IntlBackslash => "intl_backslash",
            Self::IntlRo => "intl_ro",
            Self::IntlYen => "intl_yen",
            Self::KeyA => "key_a",
            Self::KeyB => "key_b",
            Self::KeyC => "key_c",
            Self::KeyD => "key_d",
            Self::KeyE => "key_e",
            Self::KeyF => "key_f",
            Self::KeyG => "key_g",
            Self::KeyH => "key_h",
            Self::KeyI => "key_i",
            Self::KeyJ => "key_j",
            Self::KeyK => "key_k",
            Self::KeyL => "key_l",
            Self::KeyM => "key_m",
            Self::KeyN => "key_n",
            Self::KeyO => "key_o",
            Self::KeyP => "key_p",
            Self::KeyQ => "key_q",
            Self::KeyR => "key_r",
            Self::KeyS => "key_s",
            Self::KeyT => "key_t",
            Self::KeyU => "key_u",
            Self::KeyV => "key_v",
            Self::KeyW => "key_w",
            Self::KeyX => "key_x",
            Self::KeyY => "key_y",
            Self::KeyZ => "key_z",
            Self::Minus => "minus",
            Self::Period => "period",
            Self::Quote => "quote",
            Self::Semicolon => "semicolon",
            Self::Slash => "slash",
            Self::AltLeft => "alt_left",
            Self::AltRight => "alt_right",
            Self::Backspace => "backspace",
            Self::CapsLock => "caps_lock",
            Self::ContextMenu => "context_menu",
            Self::ControlLeft => "control_left",
            Self::ControlRight => "control_right",
            Self::Enter => "enter",
            Self::MetaLeft => "meta_left",
            Self::MetaRight => "meta_right",
            Self::ShiftLeft => "shift_left",
            Self::ShiftRight => "shift_right",
            Self::Space => "space",
            Self::Tab => "tab",
            Self::Convert => "convert",
            Self::KanaMode => "kana_mode",
            Self::NonConvert => "non_convert",
            Self::Delete => "delete",
            Self::End => "end",
            Self::Help => "help",
            Self::Home => "home",
            Self::Insert => "insert",
            Self::PageDown => "page_down",
            Self::PageUp => "page_up",
            Self::ArrowDown => "arrow_down",
            Self::ArrowLeft => "arrow_left",
            Self::ArrowRight => "arrow_right",
            Self::ArrowUp => "arrow_up",
            Self::NumLock => "num_lock",
            Self::Numpad0 => "numpad_0",
            Self::Numpad1 => "numpad_1",
            Self::Numpad2 => "numpad_2",
            Self::Numpad3 => "numpad_3",
            Self::Numpad4 => "numpad_4",
            Self::Numpad5 => "numpad_5",
            Self::Numpad6 => "numpad_6",
            Self::Numpad7 => "numpad_7",
            Self::Numpad8 => "numpad_8",
            Self::Numpad9 => "numpad_9",
            Self::NumpadAdd => "numpad_add",
            Self::NumpadBackspace => "numpad_backspace",
            Self::NumpadClear => "numpad_clear",
            Self::NumpadClearEntry => "numpad_clear_entry",
            Self::NumpadComma => "numpad_comma",
            Self::NumpadDecimal => "numpad_decimal",
            Self::NumpadDivide => "numpad_divide",
            Self::NumpadEnter => "numpad_enter",
            Self::NumpadEqual => "numpad_equal",
            Self::NumpadMemoryAdd => "numpad_memory_add",
            Self::NumpadMemoryClear => "numpad_memory_clear",
            Self::NumpadMemoryRecall => "numpad_memory_recall",
            Self::NumpadMemoryStore => "numpad_memory_store",
            Self::NumpadMemorySubtract => "numpad_memory_subtract",
            Self::NumpadMultiply => "numpad_multiply",
            Self::NumpadParenLeft => "numpad_paren_left",
            Self::NumpadParenRight => "numpad_paren_right",
            Self::NumpadSubtract => "numpad_subtract",
            Self::NumpadSeparator => "numpad_separator",
            Self::NumpadUp => "numpad_up",
            Self::NumpadDown => "numpad_down",
            Self::NumpadRight => "numpad_right",
            Self::NumpadLeft => "numpad_left",
            Self::NumpadBegin => "numpad_begin",
            Self::NumpadHome => "numpad_home",
            Self::NumpadEnd => "numpad_end",
            Self::NumpadInsert => "numpad_insert",
            Self::NumpadDelete => "numpad_delete",
            Self::NumpadPageUp => "numpad_page_up",
            Self::NumpadPageDown => "numpad_page_down",
            Self::Escape => "escape",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
            Self::F8 => "f8",
            Self::F9 => "f9",
            Self::F10 => "f10",
            Self::F11 => "f11",
            Self::F12 => "f12",
            Self::F13 => "f13",
            Self::F14 => "f14",
            Self::F15 => "f15",
            Self::F16 => "f16",
            Self::F17 => "f17",
            Self::F18 => "f18",
            Self::F19 => "f19",
            Self::F20 => "f20",
            Self::F21 => "f21",
            Self::F22 => "f22",
            Self::F23 => "f23",
            Self::F24 => "f24",
            Self::F25 => "f25",
            Self::Fn => "fn",
            Self::FnLock => "fn_lock",
            Self::PrintScreen => "print_screen",
            Self::ScrollLock => "scroll_lock",
            Self::Pause => "pause",
            Self::BrowserBack => "browser_back",
            Self::BrowserFavorites => "browser_favorites",
            Self::BrowserForward => "browser_forward",
            Self::BrowserHome => "browser_home",
            Self::BrowserRefresh => "browser_refresh",
            Self::BrowserSearch => "browser_search",
            Self::BrowserStop => "browser_stop",
            Self::Eject => "eject",
            Self::LaunchApp1 => "launch_app_1",
            Self::LaunchApp2 => "launch_app_2",
            Self::LaunchMail => "launch_mail",
            Self::MediaPlayPause => "media_play_pause",
            Self::MediaSelect => "media_select",
            Self::MediaStop => "media_stop",
            Self::MediaTrackNext => "media_track_next",
            Self::MediaTrackPrevious => "media_track_previous",
            Self::Power => "power",
            Self::Sleep => "sleep",
            Self::AudioVolumeDown => "audio_volume_down",
            Self::AudioVolumeMute => "audio_volume_mute",
            Self::AudioVolumeUp => "audio_volume_up",
            Self::WakeUp => "wake_up",
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste => "paste",
        };
        let mut result = String::with_capacity(name.len());
        let mut capitalize_next = true;
        for ch in name.chars() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }
        result
    }

    pub fn printable(self) -> bool {
        for &(_, key) in Self::CODEPOINT_MAP.iter() {
            if key == self {
                return true;
            }
        }
        false
    }

    pub fn modifier(self) -> bool {
        matches!(
            self,
            Self::ShiftLeft
                | Self::ControlLeft
                | Self::AltLeft
                | Self::MetaLeft
                | Self::ShiftRight
                | Self::ControlRight
                | Self::AltRight
                | Self::MetaRight
        )
    }

    pub fn keypad(self) -> bool {
        matches!(
            self,
            Self::NumLock
                | Self::Numpad0
                | Self::Numpad1
                | Self::Numpad2
                | Self::Numpad3
                | Self::Numpad4
                | Self::Numpad5
                | Self::Numpad6
                | Self::Numpad7
                | Self::Numpad8
                | Self::Numpad9
                | Self::NumpadAdd
                | Self::NumpadBackspace
                | Self::NumpadClear
                | Self::NumpadClearEntry
                | Self::NumpadComma
                | Self::NumpadDecimal
                | Self::NumpadDivide
                | Self::NumpadEnter
                | Self::NumpadEqual
                | Self::NumpadMemoryAdd
                | Self::NumpadMemoryClear
                | Self::NumpadMemoryRecall
                | Self::NumpadMemoryStore
                | Self::NumpadMemorySubtract
                | Self::NumpadMultiply
                | Self::NumpadParenLeft
                | Self::NumpadParenRight
                | Self::NumpadSubtract
                | Self::NumpadSeparator
                | Self::NumpadUp
                | Self::NumpadDown
                | Self::NumpadRight
                | Self::NumpadLeft
                | Self::NumpadBegin
                | Self::NumpadHome
                | Self::NumpadEnd
                | Self::NumpadInsert
                | Self::NumpadDelete
                | Self::NumpadPageUp
                | Self::NumpadPageDown
        )
    }

    pub fn codepoint(self) -> Option<u32> {
        for &(cp, key) in Self::CODEPOINT_MAP.iter() {
            if key == self {
                return Some(cp);
            }
        }
        None
    }

    pub fn left_or_right_shift(self) -> bool {
        matches!(self, Self::ShiftLeft | Self::ShiftRight)
    }

    pub fn left_or_right_alt(self) -> bool {
        matches!(self, Self::AltLeft | Self::AltRight)
    }

    const CODEPOINT_MAP: &'static [(u32, Self)] = &[
        ('a' as u32, Self::KeyA),
        ('b' as u32, Self::KeyB),
        ('c' as u32, Self::KeyC),
        ('d' as u32, Self::KeyD),
        ('e' as u32, Self::KeyE),
        ('f' as u32, Self::KeyF),
        ('g' as u32, Self::KeyG),
        ('h' as u32, Self::KeyH),
        ('i' as u32, Self::KeyI),
        ('j' as u32, Self::KeyJ),
        ('k' as u32, Self::KeyK),
        ('l' as u32, Self::KeyL),
        ('m' as u32, Self::KeyM),
        ('n' as u32, Self::KeyN),
        ('o' as u32, Self::KeyO),
        ('p' as u32, Self::KeyP),
        ('q' as u32, Self::KeyQ),
        ('r' as u32, Self::KeyR),
        ('s' as u32, Self::KeyS),
        ('t' as u32, Self::KeyT),
        ('u' as u32, Self::KeyU),
        ('v' as u32, Self::KeyV),
        ('w' as u32, Self::KeyW),
        ('x' as u32, Self::KeyX),
        ('y' as u32, Self::KeyY),
        ('z' as u32, Self::KeyZ),
        ('0' as u32, Self::Digit0),
        ('1' as u32, Self::Digit1),
        ('2' as u32, Self::Digit2),
        ('3' as u32, Self::Digit3),
        ('4' as u32, Self::Digit4),
        ('5' as u32, Self::Digit5),
        ('6' as u32, Self::Digit6),
        ('7' as u32, Self::Digit7),
        ('8' as u32, Self::Digit8),
        ('9' as u32, Self::Digit9),
        (';' as u32, Self::Semicolon),
        (' ' as u32, Self::Space),
        ('\'' as u32, Self::Quote),
        (',' as u32, Self::Comma),
        ('`' as u32, Self::Backquote),
        ('.' as u32, Self::Period),
        ('/' as u32, Self::Slash),
        ('-' as u32, Self::Minus),
        ('=' as u32, Self::Equal),
        ('[' as u32, Self::BracketLeft),
        (']' as u32, Self::BracketRight),
        ('\\' as u32, Self::Backslash),
        ('\t' as u32, Self::Tab),
        ('0' as u32, Self::Numpad0),
        ('1' as u32, Self::Numpad1),
        ('2' as u32, Self::Numpad2),
        ('3' as u32, Self::Numpad3),
        ('4' as u32, Self::Numpad4),
        ('5' as u32, Self::Numpad5),
        ('6' as u32, Self::Numpad6),
        ('7' as u32, Self::Numpad7),
        ('8' as u32, Self::Numpad8),
        ('9' as u32, Self::Numpad9),
        ('.' as u32, Self::NumpadDecimal),
        ('/' as u32, Self::NumpadDivide),
        ('*' as u32, Self::NumpadMultiply),
        ('-' as u32, Self::NumpadSubtract),
        ('+' as u32, Self::NumpadAdd),
        ('=' as u32, Self::NumpadEqual),
    ];

    fn from_w3c_string(s: &str) -> Option<Self> {
        match s {
            "unidentified" => Some(Self::Unidentified),
            "backquote" => Some(Self::Backquote),
            "backslash" => Some(Self::Backslash),
            "bracket_left" => Some(Self::BracketLeft),
            "bracket_right" => Some(Self::BracketRight),
            "comma" => Some(Self::Comma),
            "digit_0" => Some(Self::Digit0),
            "digit_1" => Some(Self::Digit1),
            "digit_2" => Some(Self::Digit2),
            "digit_3" => Some(Self::Digit3),
            "digit_4" => Some(Self::Digit4),
            "digit_5" => Some(Self::Digit5),
            "digit_6" => Some(Self::Digit6),
            "digit_7" => Some(Self::Digit7),
            "digit_8" => Some(Self::Digit8),
            "digit_9" => Some(Self::Digit9),
            "equal" => Some(Self::Equal),
            "intl_backslash" => Some(Self::IntlBackslash),
            "intl_ro" => Some(Self::IntlRo),
            "intl_yen" => Some(Self::IntlYen),
            "key_a" => Some(Self::KeyA),
            "key_b" => Some(Self::KeyB),
            "key_c" => Some(Self::KeyC),
            "key_d" => Some(Self::KeyD),
            "key_e" => Some(Self::KeyE),
            "key_f" => Some(Self::KeyF),
            "key_g" => Some(Self::KeyG),
            "key_h" => Some(Self::KeyH),
            "key_i" => Some(Self::KeyI),
            "key_j" => Some(Self::KeyJ),
            "key_k" => Some(Self::KeyK),
            "key_l" => Some(Self::KeyL),
            "key_m" => Some(Self::KeyM),
            "key_n" => Some(Self::KeyN),
            "key_o" => Some(Self::KeyO),
            "key_p" => Some(Self::KeyP),
            "key_q" => Some(Self::KeyQ),
            "key_r" => Some(Self::KeyR),
            "key_s" => Some(Self::KeyS),
            "key_t" => Some(Self::KeyT),
            "key_u" => Some(Self::KeyU),
            "key_v" => Some(Self::KeyV),
            "key_w" => Some(Self::KeyW),
            "key_x" => Some(Self::KeyX),
            "key_y" => Some(Self::KeyY),
            "key_z" => Some(Self::KeyZ),
            "minus" => Some(Self::Minus),
            "period" => Some(Self::Period),
            "quote" => Some(Self::Quote),
            "semicolon" => Some(Self::Semicolon),
            "slash" => Some(Self::Slash),
            "alt_left" => Some(Self::AltLeft),
            "alt_right" => Some(Self::AltRight),
            "backspace" => Some(Self::Backspace),
            "caps_lock" => Some(Self::CapsLock),
            "context_menu" => Some(Self::ContextMenu),
            "control_left" => Some(Self::ControlLeft),
            "control_right" => Some(Self::ControlRight),
            "enter" => Some(Self::Enter),
            "meta_left" => Some(Self::MetaLeft),
            "meta_right" => Some(Self::MetaRight),
            "shift_left" => Some(Self::ShiftLeft),
            "shift_right" => Some(Self::ShiftRight),
            "space" => Some(Self::Space),
            "tab" => Some(Self::Tab),
            "convert" => Some(Self::Convert),
            "kana_mode" => Some(Self::KanaMode),
            "non_convert" => Some(Self::NonConvert),
            "delete" => Some(Self::Delete),
            "end" => Some(Self::End),
            "help" => Some(Self::Help),
            "home" => Some(Self::Home),
            "insert" => Some(Self::Insert),
            "page_down" => Some(Self::PageDown),
            "page_up" => Some(Self::PageUp),
            "arrow_down" => Some(Self::ArrowDown),
            "arrow_left" => Some(Self::ArrowLeft),
            "arrow_right" => Some(Self::ArrowRight),
            "arrow_up" => Some(Self::ArrowUp),
            "num_lock" => Some(Self::NumLock),
            "numpad_0" => Some(Self::Numpad0),
            "numpad_1" => Some(Self::Numpad1),
            "numpad_2" => Some(Self::Numpad2),
            "numpad_3" => Some(Self::Numpad3),
            "numpad_4" => Some(Self::Numpad4),
            "numpad_5" => Some(Self::Numpad5),
            "numpad_6" => Some(Self::Numpad6),
            "numpad_7" => Some(Self::Numpad7),
            "numpad_8" => Some(Self::Numpad8),
            "numpad_9" => Some(Self::Numpad9),
            "numpad_add" => Some(Self::NumpadAdd),
            "numpad_backspace" => Some(Self::NumpadBackspace),
            "numpad_clear" => Some(Self::NumpadClear),
            "numpad_clear_entry" => Some(Self::NumpadClearEntry),
            "numpad_comma" => Some(Self::NumpadComma),
            "numpad_decimal" => Some(Self::NumpadDecimal),
            "numpad_divide" => Some(Self::NumpadDivide),
            "numpad_enter" => Some(Self::NumpadEnter),
            "numpad_equal" => Some(Self::NumpadEqual),
            "numpad_memory_add" => Some(Self::NumpadMemoryAdd),
            "numpad_memory_clear" => Some(Self::NumpadMemoryClear),
            "numpad_memory_recall" => Some(Self::NumpadMemoryRecall),
            "numpad_memory_store" => Some(Self::NumpadMemoryStore),
            "numpad_memory_subtract" => Some(Self::NumpadMemorySubtract),
            "numpad_multiply" => Some(Self::NumpadMultiply),
            "numpad_paren_left" => Some(Self::NumpadParenLeft),
            "numpad_paren_right" => Some(Self::NumpadParenRight),
            "numpad_subtract" => Some(Self::NumpadSubtract),
            "numpad_separator" => Some(Self::NumpadSeparator),
            "numpad_up" => Some(Self::NumpadUp),
            "numpad_down" => Some(Self::NumpadDown),
            "numpad_right" => Some(Self::NumpadRight),
            "numpad_left" => Some(Self::NumpadLeft),
            "numpad_begin" => Some(Self::NumpadBegin),
            "numpad_home" => Some(Self::NumpadHome),
            "numpad_end" => Some(Self::NumpadEnd),
            "numpad_insert" => Some(Self::NumpadInsert),
            "numpad_delete" => Some(Self::NumpadDelete),
            "numpad_page_up" => Some(Self::NumpadPageUp),
            "numpad_page_down" => Some(Self::NumpadPageDown),
            "escape" => Some(Self::Escape),
            "f1" => Some(Self::F1),
            "f2" => Some(Self::F2),
            "f3" => Some(Self::F3),
            "f4" => Some(Self::F4),
            "f5" => Some(Self::F5),
            "f6" => Some(Self::F6),
            "f7" => Some(Self::F7),
            "f8" => Some(Self::F8),
            "f9" => Some(Self::F9),
            "f10" => Some(Self::F10),
            "f11" => Some(Self::F11),
            "f12" => Some(Self::F12),
            "f13" => Some(Self::F13),
            "f14" => Some(Self::F14),
            "f15" => Some(Self::F15),
            "f16" => Some(Self::F16),
            "f17" => Some(Self::F17),
            "f18" => Some(Self::F18),
            "f19" => Some(Self::F19),
            "f20" => Some(Self::F20),
            "f21" => Some(Self::F21),
            "f22" => Some(Self::F22),
            "f23" => Some(Self::F23),
            "f24" => Some(Self::F24),
            "f25" => Some(Self::F25),
            "fn" => Some(Self::Fn),
            "fn_lock" => Some(Self::FnLock),
            "print_screen" => Some(Self::PrintScreen),
            "scroll_lock" => Some(Self::ScrollLock),
            "pause" => Some(Self::Pause),
            "browser_back" => Some(Self::BrowserBack),
            "browser_favorites" => Some(Self::BrowserFavorites),
            "browser_forward" => Some(Self::BrowserForward),
            "browser_home" => Some(Self::BrowserHome),
            "browser_refresh" => Some(Self::BrowserRefresh),
            "browser_search" => Some(Self::BrowserSearch),
            "browser_stop" => Some(Self::BrowserStop),
            "eject" => Some(Self::Eject),
            "launch_app_1" => Some(Self::LaunchApp1),
            "launch_app_2" => Some(Self::LaunchApp2),
            "launch_mail" => Some(Self::LaunchMail),
            "media_play_pause" => Some(Self::MediaPlayPause),
            "media_select" => Some(Self::MediaSelect),
            "media_stop" => Some(Self::MediaStop),
            "media_track_next" => Some(Self::MediaTrackNext),
            "media_track_previous" => Some(Self::MediaTrackPrevious),
            "power" => Some(Self::Power),
            "sleep" => Some(Self::Sleep),
            "audio_volume_down" => Some(Self::AudioVolumeDown),
            "audio_volume_mute" => Some(Self::AudioVolumeMute),
            "audio_volume_up" => Some(Self::AudioVolumeUp),
            "wake_up" => Some(Self::WakeUp),
            "copy" => Some(Self::Copy),
            "cut" => Some(Self::Cut),
            "paste" => Some(Self::Paste),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeyEvent {
    pub action: Action,
    pub key: Key,
    pub mods: Mods,
    pub consumed_mods: Mods,
    pub composing: bool,
    pub utf8: String,
    pub unshifted_codepoint: u32,
}

impl KeyEvent {
    pub fn effective_mods(&self) -> Mods {
        if self.utf8.is_empty() {
            return self.mods;
        }
        self.mods.unset(self.consumed_mods)
    }

    pub fn binding_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.key.hash(&mut hasher);
        self.unshifted_codepoint.hash(&mut hasher);
        self.mods.binding().int().hash(&mut hasher);
        hasher.finish()
    }
}

pub fn ctrl_seq(logical_key: Key, utf8: &str, unshifted_codepoint: u32, mods: Mods) -> Option<u8> {
    if !mods.ctrl {
        return None;
    }

    let mut binding_mods = mods.binding();
    binding_mods.alt = false;

    let char_byte = if utf8.len() == 1 {
        utf8.as_bytes()[0]
    } else if let Some(cp) = logical_key.codepoint() {
        if let Ok(byte) = u8::try_from(cp) {
            let ctrl_only = Mods {
                ctrl: true,
                ..Mods::default()
            }
            .int();
            if binding_mods.int() != ctrl_only {
                return None;
            }
            byte
        } else {
            return None;
        }
    } else {
        return None;
    };

    if binding_mods.shift && (char_byte < b'A' || char_byte > b'Z') {
        if char_byte != b'@' {
            binding_mods.shift = false;
        }
    }

    let mut char_mut = char_byte;
    if char_byte >= b'A' && char_byte <= b'Z' && unshifted_codepoint > 0 {
        if let Ok(byte) = u8::try_from(unshifted_codepoint) {
            char_mut = byte;
        }
    }

    let ctrl_only = Mods {
        ctrl: true,
        ..Mods::default()
    }
    .int();
    if binding_mods.int() != ctrl_only {
        return None;
    }

    match char_mut {
        b' ' => Some(0),
        b'/' => Some(31),
        b'0' => Some(48),
        b'1' => Some(49),
        b'2' => Some(0),
        b'3' => Some(27),
        b'4' => Some(28),
        b'5' => Some(29),
        b'6' => Some(30),
        b'7' => Some(31),
        b'8' => Some(127),
        b'9' => Some(57),
        b'?' => Some(127),
        b'@' => Some(0),
        b'\\' => Some(28),
        b']' => Some(29),
        b'^' => Some(30),
        b'_' => Some(31),
        b'a' => Some(1),
        b'b' => Some(2),
        b'c' => Some(3),
        b'd' => Some(4),
        b'e' => Some(5),
        b'f' => Some(6),
        b'g' => Some(7),
        b'h' => Some(8),
        b'j' => Some(10),
        b'k' => Some(11),
        b'l' => Some(12),
        b'n' => Some(14),
        b'o' => Some(15),
        b'p' => Some(16),
        b'q' => Some(17),
        b'r' => Some(18),
        b's' => Some(19),
        b't' => Some(20),
        b'u' => Some(21),
        b'v' => Some(22),
        b'w' => Some(23),
        b'x' => Some(24),
        b'y' => Some(25),
        b'z' => Some(26),
        b'~' => Some(30),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CsiUMods {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl CsiUMods {
    pub fn from_input(mods: Mods) -> Self {
        Self {
            shift: mods.shift,
            alt: mods.alt,
            ctrl: mods.ctrl,
        }
    }

    pub fn int(self) -> u8 {
        let mut v = 0u8;
        if self.shift {
            v |= 1 << 0;
        }
        if self.alt {
            v |= 1 << 1;
        }
        if self.ctrl {
            v |= 1 << 2;
        }
        v
    }

    pub fn seq_int(self) -> u8 {
        self.int() + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_ascii_should_not_return_keypad_keys() {
        assert_eq!(Key::from_ascii(b'0'), Some(Key::Digit0));
        assert_eq!(Key::from_ascii(b'*'), None);
    }

    #[test]
    fn keypad_keys() {
        assert!(Key::Numpad0.keypad());
        assert!(!Key::Digit1.keypad());
    }

    #[test]
    fn w3c_roundtrip() {
        let keys = [
            Key::KeyA,
            Key::Digit0,
            Key::Escape,
            Key::ArrowUp,
            Key::Numpad5,
            Key::F12,
        ];
        for key in keys {
            let w3c_name = key.w3c();
            assert_eq!(Key::from_w3c(&w3c_name), Some(key));
        }
    }

    #[test]
    fn ctrl_or_super() {
        let m = crate::key_mods::ctrl_or_super(Mods::default());
        assert!(m.ctrl_or_super());
    }

    #[test]
    fn csi_u_mods_sequence_values() {
        assert_eq!(CsiUMods::default().seq_int(), 1);
        assert_eq!(CsiUMods { shift: true, ..Default::default() }.seq_int(), 2);
        assert_eq!(CsiUMods { alt: true, ..Default::default() }.seq_int(), 3);
        assert_eq!(CsiUMods { shift: true, alt: true, ..Default::default() }.seq_int(), 4);
        assert_eq!(CsiUMods { ctrl: true, ..Default::default() }.seq_int(), 5);
        assert_eq!(CsiUMods { shift: true, ctrl: true, ..Default::default() }.seq_int(), 6);
        assert_eq!(CsiUMods { alt: true, ctrl: true, ..Default::default() }.seq_int(), 7);
        assert_eq!(
            CsiUMods { shift: true, alt: true, ctrl: true }.seq_int(),
            8
        );
    }

    #[test]
    fn ctrl_seq_ctrl_c() {
        let mods = Mods {
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(ctrl_seq(Key::KeyC, "c", 'c' as u32, mods), Some(3));
    }

    #[test]
    fn ctrl_seq_no_ctrl() {
        let mods = Mods::default();
        assert_eq!(ctrl_seq(Key::KeyC, "c", 'c' as u32, mods), None);
    }

    #[test]
    fn ctrl_seq_ctrl_space() {
        let mods = Mods {
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(ctrl_seq(Key::Space, " ", ' ' as u32, mods), Some(0));
    }

    #[test]
    fn ctrl_seq_fixterm_awkward_i_excluded() {
        let mods = Mods {
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(ctrl_seq(Key::KeyI, "i", 'i' as u32, mods), None);
    }

    #[test]
    fn ctrl_seq_ctrl_shift_minus() {
        let mods = Mods {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(ctrl_seq(Key::Minus, "_", '_' as u32, mods), Some(31));
    }

    #[test]
    fn ctrl_seq_with_alt_allowed() {
        let mods = Mods {
            ctrl: true,
            alt: true,
            ..Default::default()
        };
        assert_eq!(ctrl_seq(Key::KeyC, "c", 'c' as u32, mods), Some(3));
    }

    #[test]
    fn kitty_mods_from_input() {
        use crate::kitty_sequence::KittyMods;
        let mods = Mods {
            shift: true,
            ctrl: true,
            ..Default::default()
        };
        let km = KittyMods::from_input(Action::Press, Key::KeyA, mods);
        assert!(km.shift);
        assert!(km.ctrl);
        assert!(!km.alt);
    }

    #[test]
    fn kitty_mods_prevents_text() {
        use crate::kitty_sequence::KittyMods;
        let plain = KittyMods::default();
        assert!(!plain.prevents_text(false));
        assert!(!plain.prevents_text(true));

        let with_ctrl = KittyMods {
            ctrl: true,
            ..Default::default()
        };
        assert!(with_ctrl.prevents_text(false));

        let with_alt = KittyMods {
            alt: true,
            ..Default::default()
        };
        assert!(!with_alt.prevents_text(false));
        assert!(with_alt.prevents_text(true));
    }
}
