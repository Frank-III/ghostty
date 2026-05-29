use crate::constants::*;
use crate::early::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeTag {
    pub value: u16,
    pub ansi: bool,
}

impl ModeTag {
    pub fn to_u16(self) -> u16 {
        (if self.ansi { MODE_ANSI_MASK } else { 0 }) | (self.value & MODE_VALUE_MASK)
    }

    pub fn from_u16(v: u16) -> Self {
        ModeTag {
            value: v & MODE_VALUE_MASK,
            ansi: (v & MODE_ANSI_MASK) != 0,
        }
    }
}

impl Default for ModeTag {
    fn default() -> Self {
        ModeTag {
            value: 0,
            ansi: false,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeReportState {
    NotRecognized = 0,
    Set = 1,
    Reset = 2,
    PermanentlySet = 3,
    PermanentlyReset = 4,
}

impl Default for ModeReportState {
    fn default() -> Self {
        ModeReportState::NotRecognized
    }
}

impl ModeReportState {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(ModeReportState::NotRecognized),
            1 => Some(ModeReportState::Set),
            2 => Some(ModeReportState::Reset),
            3 => Some(ModeReportState::PermanentlySet),
            4 => Some(ModeReportState::PermanentlyReset),
            _ => None,
        }
    }
}

pub struct ModeEntry {
    pub name: &'static str,
    pub value: u16,
    pub default: bool,
    pub ansi: bool,
    pub disabled: bool,
}

pub static MODE_ENTRIES: &[ModeEntry] = &[
    ModeEntry {
        name: "disable_keyboard",
        value: 2,
        default: false,
        ansi: true,
        disabled: false,
    },
    ModeEntry {
        name: "insert",
        value: 4,
        default: false,
        ansi: true,
        disabled: false,
    },
    ModeEntry {
        name: "send_receive_mode",
        value: 12,
        default: true,
        ansi: true,
        disabled: false,
    },
    ModeEntry {
        name: "linefeed",
        value: 20,
        default: false,
        ansi: true,
        disabled: false,
    },
    ModeEntry {
        name: "cursor_keys",
        value: 1,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "132_column",
        value: 3,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "slow_scroll",
        value: 4,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "reverse_colors",
        value: 5,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "origin",
        value: 6,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "wraparound",
        value: 7,
        default: true,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "autorepeat",
        value: 8,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_event_x10",
        value: 9,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "cursor_blinking",
        value: 12,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "cursor_visible",
        value: 25,
        default: true,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "enable_mode_3",
        value: 40,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "reverse_wrap",
        value: 45,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "alt_screen_legacy",
        value: 47,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "keypad_keys",
        value: 66,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "backarrow_key_mode",
        value: 67,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "enable_left_and_right_margin",
        value: 69,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_event_normal",
        value: 1000,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_event_button",
        value: 1002,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_event_any",
        value: 1003,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "focus_event",
        value: 1004,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_format_utf8",
        value: 1005,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_format_sgr",
        value: 1006,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_alternate_scroll",
        value: 1007,
        default: true,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_format_urxvt",
        value: 1015,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "mouse_format_sgr_pixels",
        value: 1016,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "ignore_keypad_with_numlock",
        value: 1035,
        default: true,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "alt_esc_prefix",
        value: 1036,
        default: true,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "alt_sends_escape",
        value: 1039,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "reverse_wrap_extended",
        value: 1045,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "alt_screen",
        value: 1047,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "save_cursor",
        value: 1048,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "alt_screen_save_cursor_clear_enter",
        value: 1049,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "bracketed_paste",
        value: 2004,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "synchronized_output",
        value: 2026,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "grapheme_cluster",
        value: 2027,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "report_color_scheme",
        value: 2031,
        default: false,
        ansi: false,
        disabled: false,
    },
    ModeEntry {
        name: "in_band_size_reports",
        value: 2048,
        default: false,
        ansi: false,
        disabled: false,
    },
];

pub const MODE_COUNT: usize = MODE_ENTRIES.len();

pub fn mode_find_index(value: u16, ansi: bool) -> Option<u8> {
    for (i, entry) in MODE_ENTRIES.iter().enumerate() {
        if !entry.disabled && entry.value == value && entry.ansi == ansi {
            return Some(i as u8);
        }
    }
    None
}

pub fn mode_tag_from_index(idx: u8) -> ModeTag {
    let idx = idx as usize;
    if idx >= MODE_COUNT {
        return ModeTag {
            value: 0,
            ansi: true,
        };
    }
    let entry = unsafe { MODE_ENTRIES.get_unchecked(idx) };
    ModeTag {
        value: entry.value,
        ansi: entry.ansi,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModeState {
    values: u64,
    saved: u64,
    default: u64,
}

impl ModeState {
    pub fn new() -> Self {
        let mut default: u64 = 0;
        for (i, entry) in MODE_ENTRIES.iter().enumerate() {
            if entry.default {
                default |= 1u64 << (i as u64);
            }
        }
        ModeState {
            values: default,
            saved: 0,
            default,
        }
    }

    pub fn reset(&mut self) {
        self.values = self.default;
        self.saved = 0;
    }

    fn get_bit(&self, idx: u8) -> bool {
        (self.values >> (idx as u64)) & 1 != 0
    }

    fn set_bit(&mut self, idx: u8, value: bool) {
        if value {
            self.values |= 1u64 << (idx as u64);
        } else {
            self.values &= !(1u64 << (idx as u64));
        }
    }

    fn save_bit(&mut self, idx: u8) {
        let val = self.get_bit(idx);
        if val {
            self.saved |= 1u64 << (idx as u64);
        } else {
            self.saved &= !(1u64 << (idx as u64));
        }
    }

    fn restore_bit(&mut self, idx: u8) -> bool {
        let val = (self.saved >> (idx as u64)) & 1 != 0;
        self.set_bit(idx, val);
        val
    }

    pub fn set_by_tag(&mut self, tag: ModeTag, value: bool) {
        if let Some(idx) = mode_find_index(tag.value, tag.ansi) {
            self.set_bit(idx, value);
        }
    }

    pub fn get_by_tag(&self, tag: ModeTag) -> bool {
        if let Some(idx) = mode_find_index(tag.value, tag.ansi) {
            self.get_bit(idx)
        } else {
            false
        }
    }

    pub fn save_by_tag(&mut self, tag: ModeTag) {
        if let Some(idx) = mode_find_index(tag.value, tag.ansi) {
            self.save_bit(idx);
        }
    }

    pub fn restore_by_tag(&mut self, tag: ModeTag) -> bool {
        if let Some(idx) = mode_find_index(tag.value, tag.ansi) {
            self.restore_bit(idx)
        } else {
            false
        }
    }

    pub fn get_report(&self, lookup: ModeTag) -> (ModeTag, ModeReportState) {
        match mode_find_index(lookup.value, lookup.ansi) {
            Some(idx) => {
                let state = if self.get_bit(idx) {
                    ModeReportState::Set
                } else {
                    ModeReportState::Reset
                };
                (lookup, state)
            }
            None => (lookup, ModeReportState::NotRecognized),
        }
    }
}

impl Default for ModeState {
    fn default() -> Self {
        ModeState::new()
    }
}
