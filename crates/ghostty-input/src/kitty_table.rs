use crate::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KittyTableEntry {
    pub key: Key,
    pub code: u32,
    pub final_byte: u8,
    pub modifier: bool,
}

pub const ENTRIES: &[KittyTableEntry] = &[
    KittyTableEntry {
        key: Key::Escape,
        code: 27,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Enter,
        code: 13,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Tab,
        code: 9,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Backspace,
        code: 127,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Insert,
        code: 2,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Delete,
        code: 3,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::ArrowLeft,
        code: 1,
        final_byte: b'D',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::ArrowRight,
        code: 1,
        final_byte: b'C',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::ArrowUp,
        code: 1,
        final_byte: b'A',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::ArrowDown,
        code: 1,
        final_byte: b'B',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::PageUp,
        code: 5,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::PageDown,
        code: 6,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Home,
        code: 1,
        final_byte: b'H',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::End,
        code: 1,
        final_byte: b'F',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::CapsLock,
        code: 57358,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::ScrollLock,
        code: 57359,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumLock,
        code: 57360,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::PrintScreen,
        code: 57361,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Pause,
        code: 57362,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F1,
        code: 1,
        final_byte: b'P',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F2,
        code: 1,
        final_byte: b'Q',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F3,
        code: 13,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F4,
        code: 1,
        final_byte: b'S',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F5,
        code: 15,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F6,
        code: 17,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F7,
        code: 18,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F8,
        code: 19,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F9,
        code: 20,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F10,
        code: 21,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F11,
        code: 23,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F12,
        code: 24,
        final_byte: b'~',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F13,
        code: 57376,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F14,
        code: 57377,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F15,
        code: 57378,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F16,
        code: 57379,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F17,
        code: 57380,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F18,
        code: 57381,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F19,
        code: 57382,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F20,
        code: 57383,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F21,
        code: 57384,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F22,
        code: 57385,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F23,
        code: 57386,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F24,
        code: 57387,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::F25,
        code: 57388,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad0,
        code: 57399,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad1,
        code: 57400,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad2,
        code: 57401,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad3,
        code: 57402,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad4,
        code: 57403,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad5,
        code: 57404,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad6,
        code: 57405,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad7,
        code: 57406,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad8,
        code: 57407,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::Numpad9,
        code: 57408,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadDecimal,
        code: 57409,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadDivide,
        code: 57410,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadMultiply,
        code: 57411,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadSubtract,
        code: 57412,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadAdd,
        code: 57413,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadEnter,
        code: 57414,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadEqual,
        code: 57415,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadSeparator,
        code: 57416,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadLeft,
        code: 57417,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadRight,
        code: 57418,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadUp,
        code: 57419,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadDown,
        code: 57420,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadPageUp,
        code: 57421,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadPageDown,
        code: 57422,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadHome,
        code: 57423,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadEnd,
        code: 57424,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadInsert,
        code: 57425,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadDelete,
        code: 57426,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::NumpadBegin,
        code: 57427,
        final_byte: b'u',
        modifier: false,
    },
    KittyTableEntry {
        key: Key::ShiftLeft,
        code: 57441,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::ShiftRight,
        code: 57447,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::ControlLeft,
        code: 57442,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::ControlRight,
        code: 57448,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::MetaLeft,
        code: 57444,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::MetaRight,
        code: 57450,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::AltLeft,
        code: 57443,
        final_byte: b'u',
        modifier: true,
    },
    KittyTableEntry {
        key: Key::AltRight,
        code: 57449,
        final_byte: b'u',
        modifier: true,
    },
];

pub fn find(key: Key) -> Option<&'static KittyTableEntry> {
    ENTRIES.iter().find(|e| e.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_81_entries() {
        assert_eq!(ENTRIES.len(), 81);
    }

    #[test]
    fn find_escape() {
        let e = find(Key::Escape).unwrap();
        assert_eq!(e.code, 27);
        assert_eq!(e.final_byte, b'u');
        assert!(!e.modifier);
    }

    #[test]
    fn find_arrow_up() {
        let e = find(Key::ArrowUp).unwrap();
        assert_eq!(e.code, 1);
        assert_eq!(e.final_byte, b'A');
    }

    #[test]
    fn find_shift_left_is_modifier() {
        let e = find(Key::ShiftLeft).unwrap();
        assert!(e.modifier);
        assert_eq!(e.code, 57441);
    }

    #[test]
    fn find_unknown_returns_none() {
        assert!(find(Key::Unidentified).is_none());
    }
}
