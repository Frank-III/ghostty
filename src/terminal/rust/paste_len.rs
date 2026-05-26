use crate::constants::*;

pub(crate) fn paste_encoded_len(data_len: usize, bracketed: bool) -> usize {
    let prefix_len = if bracketed { PASTE_START.len() } else { 0 };
    let suffix_len = if bracketed { PASTE_END.len() } else { 0 };
    prefix_len + data_len + suffix_len
}
