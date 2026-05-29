//! Kitty keyboard protocol CSI sequences (`src/input/key_encode.zig` `KittySequence`).

use std::fmt::{self, Write};

/// Kitty protocol modifier bitmask (`key_encode.zig` `KittyMods`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KittyMods {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub super_key: bool,
    pub hyper: bool,
    pub meta: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

impl KittyMods {
    pub fn int(self) -> u8 {
        let mut v: u8 = 0;
        if self.shift {
            v |= 1 << 0;
        }
        if self.alt {
            v |= 1 << 1;
        }
        if self.ctrl {
            v |= 1 << 2;
        }
        if self.super_key {
            v |= 1 << 3;
        }
        if self.hyper {
            v |= 1 << 4;
        }
        if self.meta {
            v |= 1 << 5;
        }
        if self.caps_lock {
            v |= 1 << 6;
        }
        if self.num_lock {
            v |= 1 << 7;
        }
        v
    }

    pub fn from_input(
        _action: crate::key::Action,
        _key: crate::key::Key,
        mods: crate::Mods,
    ) -> Self {
        Self {
            shift: mods.shift,
            alt: mods.alt,
            ctrl: mods.ctrl,
            super_key: mods.super_key,
            caps_lock: mods.caps_lock,
            num_lock: mods.num_lock,
            hyper: false,
            meta: false,
        }
    }

    pub fn prevents_text(self, alt_prevents_text: bool) -> bool {
        (self.alt && alt_prevents_text)
            || self.ctrl
            || self.super_key
            || self.hyper
            || self.meta
    }

    /// Spec modifier parameter (`bitmask + 1`).
    pub fn seq_int(self) -> u16 {
        u16::from(self.int()) + 1
    }
}

/// Event-type field in a Kitty sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KittyEvent {
    #[default]
    None,
    Press,
    Repeat,
    Release,
}

impl KittyEvent {
    fn as_u2(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Press => 1,
            Self::Repeat => 2,
            Self::Release => 3,
        }
    }
}

/// Kitty CSI key sequence: `CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ; text u`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittySequence {
    pub key: u32,
    pub final_byte: u8,
    pub mods: KittyMods,
    pub event: KittyEvent,
    pub alternates: [Option<u32>; 2],
    pub text: String,
}

impl KittySequence {
    pub fn encode<W: Write>(&self, writer: &mut W) -> fmt::Result {
        if self.final_byte == b'u' || self.final_byte == b'~' {
            self.encode_full(writer)
        } else {
            self.encode_special(writer)
        }
    }

    pub fn encode_to_string(&self) -> String {
        let mut s = String::new();
        self.encode(&mut s).expect("string writer");
        s
    }

    fn encode_full<W: Write>(&self, writer: &mut W) -> fmt::Result {
        write!(writer, "\x1b[{}", self.key)?;

        if let Some(shifted) = self.alternates[0] {
            write!(writer, ":{shifted}")?;
        }
        if let Some(base) = self.alternates[1] {
            if self.alternates[0].is_none() {
                write!(writer, "::{base}")?;
            } else {
                write!(writer, ":{base}")?;
            }
        }

        let mods = self.mods.seq_int();
        let mut emit_prior = false;
        if !matches!(self.event, KittyEvent::None | KittyEvent::Press) {
            write!(writer, ";{mods}:{}", self.event.as_u2())?;
            emit_prior = true;
        } else if mods > 1 {
            write!(writer, ";{mods}")?;
            emit_prior = true;
        }

        if !self.text.is_empty() {
            let mut count = 0usize;
            for ch in self.text.chars() {
                if is_control(ch) {
                    continue;
                }
                if count == 0 {
                    if !emit_prior {
                        writer.write_str(";")?;
                    }
                    writer.write_str(";")?;
                } else {
                    writer.write_str(":")?;
                }
                write!(writer, "{}", ch as u32)?;
                count += 1;
            }
        }

        write!(writer, "{}", self.final_byte as char)
    }

    fn encode_special<W: Write>(&self, writer: &mut W) -> fmt::Result {
        let mods = self.mods.seq_int();
        if !matches!(self.event, KittyEvent::None) {
            write!(
                writer,
                "\x1b[1;{}:{}",
                mods,
                self.event.as_u2(),
            )?;
            write!(writer, "{}", self.final_byte as char)?;
            return Ok(());
        }

        if mods > 1 {
            write!(writer, "\x1b[1;{}", mods)?;
            write!(writer, "{}", self.final_byte as char)?;
            return Ok(());
        }

        write!(writer, "\x1b[{}", self.final_byte as char)
    }
}

fn is_control(cp: char) -> bool {
    (cp as u32) < 0x20 || cp == '\x7f'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kitty_mods_sequence_values() {
        assert_eq!(KittyMods::default().seq_int(), 1);
        assert_eq!(
            KittyMods {
                shift: true,
                ..Default::default()
            }
            .seq_int(),
            2
        );
        assert_eq!(
            KittyMods {
                alt: true,
                ..Default::default()
            }
            .seq_int(),
            3
        );
        assert_eq!(
            KittyMods {
                ctrl: true,
                ..Default::default()
            }
            .seq_int(),
            5
        );
        assert_eq!(
            KittyMods {
                alt: true,
                shift: true,
                ..Default::default()
            }
            .seq_int(),
            4
        );
        assert_eq!(
            KittyMods {
                ctrl: true,
                shift: true,
                ..Default::default()
            }
            .seq_int(),
            6
        );
        assert_eq!(
            KittyMods {
                alt: true,
                ctrl: true,
                ..Default::default()
            }
            .seq_int(),
            7
        );
        assert_eq!(
            KittyMods {
                alt: true,
                ctrl: true,
                shift: true,
                ..Default::default()
            }
            .seq_int(),
            8
        );
    }

    #[test]
    fn kitty_sequence_backspace() {
        let plain = KittySequence {
            key: 127,
            final_byte: b'u',
            ..Default::default()
        };
        assert_eq!(plain.encode_to_string(), "\x1b[127u");

        let release = KittySequence {
            key: 127,
            final_byte: b'u',
            event: KittyEvent::Release,
            ..Default::default()
        };
        assert_eq!(release.encode_to_string(), "\x1b[127;1:3u");

        let shift = KittySequence {
            key: 127,
            final_byte: b'u',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(shift.encode_to_string(), "\x1b[127;2u");
    }

    #[test]
    fn kitty_sequence_text() {
        let plain = KittySequence {
            key: 127,
            final_byte: b'u',
            text: "A".into(),
            ..Default::default()
        };
        assert_eq!(plain.encode_to_string(), "\x1b[127;;65u");

        let release = KittySequence {
            key: 127,
            final_byte: b'u',
            event: KittyEvent::Release,
            text: "A".into(),
            ..Default::default()
        };
        assert_eq!(release.encode_to_string(), "\x1b[127;1:3;65u");

        let shift = KittySequence {
            key: 127,
            final_byte: b'u',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            text: "A".into(),
            ..Default::default()
        };
        assert_eq!(shift.encode_to_string(), "\x1b[127;2;65u");
    }

    #[test]
    fn kitty_sequence_text_with_control_characters() {
        let newline = KittySequence {
            key: 127,
            final_byte: b'u',
            text: "\n".into(),
            ..Default::default()
        };
        assert_eq!(newline.encode_to_string(), "\x1b[127u");

        let newline_shift = KittySequence {
            key: 127,
            final_byte: b'u',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            text: "\n".into(),
            ..Default::default()
        };
        assert_eq!(newline_shift.encode_to_string(), "\x1b[127;2u");
    }

    #[test]
    fn kitty_sequence_special() {
        let no_mods = KittySequence {
            key: 1,
            final_byte: b'A',
            ..Default::default()
        };
        assert_eq!(no_mods.encode_to_string(), "\x1b[A");

        let mods_only = KittySequence {
            key: 1,
            final_byte: b'A',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(mods_only.encode_to_string(), "\x1b[1;2A");

        let mods_and_event = KittySequence {
            key: 1,
            final_byte: b'A',
            event: KittyEvent::Release,
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(mods_and_event.encode_to_string(), "\x1b[1;2:3A");
    }

    #[test]
    fn kitty_sequence_shift_editing_keys() {
        let shift_enter = KittySequence {
            key: 13,
            final_byte: b'u',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(shift_enter.encode_to_string(), "\x1b[13;2u");

        let shift_tab = KittySequence {
            key: 9,
            final_byte: b'u',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(shift_tab.encode_to_string(), "\x1b[9;2u");

        let delete = KittySequence {
            key: 127,
            final_byte: b'~',
            ..Default::default()
        };
        assert_eq!(delete.encode_to_string(), "\x1b[127~");
    }

    #[test]
    fn kitty_sequence_ctrl_release() {
        let seq = KittySequence {
            key: 97,
            final_byte: b'u',
            event: KittyEvent::Release,
            mods: KittyMods {
                ctrl: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(seq.encode_to_string(), "\x1b[97;5:3u");
    }

    #[test]
    fn kitty_sequence_function_keys() {
        let f1 = KittySequence {
            key: 1,
            final_byte: b'P',
            ..Default::default()
        };
        assert_eq!(f1.encode_to_string(), "\x1b[P");

        let f1_shift = KittySequence {
            key: 1,
            final_byte: b'P',
            mods: KittyMods {
                shift: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(f1_shift.encode_to_string(), "\x1b[1;2P");
    }
}

impl Default for KittySequence {
    fn default() -> Self {
        Self {
            key: 0,
            final_byte: b'u',
            mods: KittyMods::default(),
            event: KittyEvent::default(),
            alternates: [None, None],
            text: String::new(),
        }
    }
}
