use crate::constants::*;
use crate::early::*;

const KITTY_KEY_STACK_LEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KittyKeyFlags(u8);

impl KittyKeyFlags {
    pub const MASK: u8 = 0b1_1111;

    pub const DISAMBIGUATE: u8 = 1 << 0;
    pub const REPORT_EVENTS: u8 = 1 << 1;
    pub const REPORT_ALTERNATES: u8 = 1 << 2;
    pub const REPORT_ALL: u8 = 1 << 3;
    pub const REPORT_ASSOCIATED: u8 = 1 << 4;

    pub const DISABLED: Self = KittyKeyFlags(0);
    pub const TRUE: Self = KittyKeyFlags(Self::MASK);

    pub fn new(raw: u8) -> Self {
        KittyKeyFlags(raw & Self::MASK)
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn has(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KittySetMode {
    Set = 0,
    Or = 1,
    Not = 2,
}

impl Default for KittySetMode {
    fn default() -> Self {
        KittySetMode::Set
    }
}

pub struct KittyKeyFlagStack {
    flags: [KittyKeyFlags; KITTY_KEY_STACK_LEN],
    idx: u8,
}

impl Default for KittyKeyFlagStack {
    fn default() -> Self {
        KittyKeyFlagStack {
            flags: [KittyKeyFlags::DISABLED; KITTY_KEY_STACK_LEN],
            idx: 0,
        }
    }
}

impl KittyKeyFlagStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> KittyKeyFlags {
        unsafe { *self.flags.get_unchecked(self.idx as usize) }
    }

    pub fn set(&mut self, mode: KittySetMode, v: KittyKeyFlags) {
        let cur = unsafe { *self.flags.get_unchecked(self.idx as usize) };
        let slot = unsafe { self.flags.get_unchecked_mut(self.idx as usize) };
        *slot = match mode {
            KittySetMode::Set => v,
            KittySetMode::Or => KittyKeyFlags::new(cur.0 | v.0),
            KittySetMode::Not => KittyKeyFlags::new(cur.0 & !v.0),
        };
    }

    pub fn push(&mut self, flags: KittyKeyFlags) {
        self.idx = (self.idx + 1) % (KITTY_KEY_STACK_LEN as u8);
        let slot = unsafe { self.flags.get_unchecked_mut(self.idx as usize) };
        *slot = flags;
    }

    pub fn pop(&mut self, n: usize) {
        if n >= KITTY_KEY_STACK_LEN {
            self.idx = 0;
            self.flags = [KittyKeyFlags::DISABLED; KITTY_KEY_STACK_LEN];
            return;
        }
        for _ in 0..n {
            let slot = unsafe { self.flags.get_unchecked_mut(self.idx as usize) };
            *slot = KittyKeyFlags::DISABLED;
            if self.idx == 0 {
                self.idx = (KITTY_KEY_STACK_LEN as u8) - 1;
            } else {
                self.idx -= 1;
            }
        }
    }
}
