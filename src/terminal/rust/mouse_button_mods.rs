use crate::constants::*;

pub(crate) fn mouse_button_apply_mods(mut code: u8, mods: u16) -> u8 {
    if (mods & MOD_SHIFT) != 0 {
        code = code.wrapping_add(4);
    }
    if (mods & MOD_ALT) != 0 {
        code = code.wrapping_add(8);
    }
    if (mods & MOD_CTRL) != 0 {
        code = code.wrapping_add(16);
    }

    code
}
