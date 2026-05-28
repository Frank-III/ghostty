//! Key modifier types (`src/input/key_mods.zig`).

/// Single modifier key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mod {
    Shift,
    Ctrl,
    Alt,
    Super,
}

/// Left or right instance of a modifier (when the platform reports it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModSide {
    #[default]
    Left,
    Right,
}

/// Per-modifier side bits (packed u4 in Zig).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModSides {
    pub shift: ModSide,
    pub ctrl: ModSide,
    pub alt: ModSide,
    pub super_key: ModSide,
}

/// Bindable modifier keys only (Zig `Mods.Keys`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// macOS option-key behavior (`src/input/config.zig` `OptionAsAlt`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptionAsAlt {
    #[default]
    False,
    True,
    Left,
    Right,
}

/// Bitmask for all key modifiers (`include/ghostty.h` layout).
///
/// Backing layout matches Zig `packed struct(u16)` field order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub sides: ModSides,
}

impl Mods {
    /// Mask with all side bits set to "right" (Zig `side_mask`).
    pub const SIDE_MASK: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
        caps_lock: false,
        num_lock: false,
        sides: ModSides {
            shift: ModSide::Right,
            ctrl: ModSide::Right,
            alt: ModSide::Right,
            super_key: ModSide::Right,
        },
    };

    pub fn int(self) -> u16 {
        let mut v: u16 = 0;
        if self.shift {
            v |= 1 << 0;
        }
        if self.ctrl {
            v |= 1 << 1;
        }
        if self.alt {
            v |= 1 << 2;
        }
        if self.super_key {
            v |= 1 << 3;
        }
        if self.caps_lock {
            v |= 1 << 4;
        }
        if self.num_lock {
            v |= 1 << 5;
        }
        if self.sides.shift == ModSide::Right {
            v |= 1 << 6;
        }
        if self.sides.ctrl == ModSide::Right {
            v |= 1 << 7;
        }
        if self.sides.alt == ModSide::Right {
            v |= 1 << 8;
        }
        if self.sides.super_key == ModSide::Right {
            v |= 1 << 9;
        }
        v
    }

    pub fn from_int(v: u16) -> Self {
        Self {
            shift: v & (1 << 0) != 0,
            ctrl: v & (1 << 1) != 0,
            alt: v & (1 << 2) != 0,
            super_key: v & (1 << 3) != 0,
            caps_lock: v & (1 << 4) != 0,
            num_lock: v & (1 << 5) != 0,
            sides: ModSides {
                shift: if v & (1 << 6) != 0 {
                    ModSide::Right
                } else {
                    ModSide::Left
                },
                ctrl: if v & (1 << 7) != 0 {
                    ModSide::Right
                } else {
                    ModSide::Left
                },
                alt: if v & (1 << 8) != 0 {
                    ModSide::Right
                } else {
                    ModSide::Left
                },
                super_key: if v & (1 << 9) != 0 {
                    ModSide::Right
                } else {
                    ModSide::Left
                },
            },
        }
    }

    pub fn empty(self) -> bool {
        self.int() == 0
    }

    pub fn equal(self, other: Self) -> bool {
        self.int() == other.int()
    }

    pub fn keys(self) -> ModKeys {
        ModKeys {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            super_key: self.super_key,
        }
    }

    pub fn binding(self) -> Self {
        Self {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            super_key: self.super_key,
            caps_lock: false,
            num_lock: false,
            sides: ModSides::default(),
        }
    }

    pub fn unset(self, other: Self) -> Self {
        Self::from_int(self.int() & !other.int())
    }

    pub fn without_locks(self) -> Self {
        Self {
            caps_lock: false,
            num_lock: false,
            ..self
        }
    }

    /// Translation mods for key mapping (e.g. macOS option-as-alt).
    pub fn translation(self, option_as_alt: OptionAsAlt) -> Self {
        let mut result = self;

        #[cfg(target_os = "macos")]
        {
            let strip_alt = match option_as_alt {
                OptionAsAlt::False => false,
                OptionAsAlt::True => true,
                OptionAsAlt::Left => self.sides.alt == ModSide::Left,
                OptionAsAlt::Right => self.sides.alt == ModSide::Right,
            };
            if strip_alt {
                result.alt = false;
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = option_as_alt;
        }

        result
    }

    #[cfg(target_os = "macos")]
    pub fn ctrl_or_super(self) -> bool {
        self.super_key
    }

    #[cfg(not(target_os = "macos"))]
    pub fn ctrl_or_super(self) -> bool {
        self.ctrl
    }
}

impl ModKeys {
    pub fn int(self) -> u8 {
        let mut v: u8 = 0;
        if self.shift {
            v |= 1;
        }
        if self.ctrl {
            v |= 1 << 1;
        }
        if self.alt {
            v |= 1 << 2;
        }
        if self.super_key {
            v |= 1 << 3;
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backing_bit_layout() {
        assert_eq!(Mods::default().int(), 0);
        assert_eq!(Mods { shift: true, ..Default::default() }.int(), 0b1);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn translation_macos_option_as_alt() {
        assert_eq!(Mods::default().translation(OptionAsAlt::True), Mods::default());

        let with_alt = Mods {
            alt: true,
            ..Default::default()
        };
        assert_eq!(with_alt.translation(OptionAsAlt::True), Mods::default());

        assert_eq!(
            with_alt.translation(OptionAsAlt::False),
            with_alt
        );

        let right_alt = Mods {
            alt: true,
            sides: ModSides {
                alt: ModSide::Right,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            right_alt.translation(OptionAsAlt::Left),
            right_alt
        );

        let left_alt = Mods {
            alt: true,
            sides: ModSides {
                alt: ModSide::Left,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            left_alt.translation(OptionAsAlt::Right),
            left_alt
        );

        let mods = Mods {
            alt: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(
            mods.translation(OptionAsAlt::True),
            Mods {
                shift: true,
                ..Default::default()
            }
        );
    }
}
