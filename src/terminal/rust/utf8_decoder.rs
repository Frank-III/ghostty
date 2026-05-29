use crate::constants::*;
use crate::early::*;

const CHAR_CLASSES: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    8, 8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    10, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, 11, 6, 6, 6, 5, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8,
];

const TRANSITIONS: [u8; 108] = [
    0, 12, 24, 36, 60, 96, 84, 12, 12, 12, 48, 72, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    12, 0, 12, 12, 12, 12, 12, 0, 12, 0, 12, 12, 12, 24, 12, 12, 12, 12, 12, 24, 12, 24, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 24, 12, 12, 12, 12, 12, 24, 12, 12, 12, 12, 12, 12, 12, 24, 12, 12,
    12, 12, 12, 12, 12, 12, 12, 36, 12, 36, 12, 12, 12, 36, 12, 12, 12, 12, 12, 36, 12, 36, 12, 12,
    12, 36, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
];

const ACCEPT_STATE: u8 = 0;
const REJECT_STATE: u8 = 12;

pub struct Utf8Decoder {
    accumulator: u32,
    state: u8,
}

impl Utf8Decoder {
    pub fn new() -> Self {
        Self {
            accumulator: 0,
            state: ACCEPT_STATE,
        }
    }

    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.state = ACCEPT_STATE;
    }

    pub fn in_ground(&self) -> bool {
        self.state == ACCEPT_STATE
    }

    pub fn next(&mut self, byte: u8) -> (Option<u32>, bool) {
        let char_class = unsafe { *CHAR_CLASSES.get_unchecked(byte as usize) };
        let initial_state = self.state;

        if self.state != ACCEPT_STATE {
            self.accumulator = (self.accumulator << 6) | ((byte & 0x3F) as u32);
        } else {
            self.accumulator = ((0xFF >> char_class) as u32) & (byte as u32);
        }

        let state_idx = self.state as usize;
        let class_idx = state_idx + char_class as usize;
        self.state = unsafe { *TRANSITIONS.get_unchecked(class_idx) };

        if self.state == ACCEPT_STATE {
            let cp = self.accumulator;
            self.accumulator = 0;
            (Some(cp), true)
        } else if self.state == REJECT_STATE {
            self.accumulator = 0;
            self.state = ACCEPT_STATE;
            (Some(0xFFFD), initial_state == ACCEPT_STATE)
        } else {
            (None, true)
        }
    }
}

impl Default for Utf8Decoder {
    fn default() -> Self {
        Self::new()
    }
}
