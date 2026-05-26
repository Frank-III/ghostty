use core::ptr;

pub(crate) fn sgr_sep_is_set(mask: u32, idx: usize) -> bool {
    idx < 32 && (mask & (1u32 << idx)) != 0
}

pub(crate) fn consume_unknown_colon(params_len: usize, sep_mask: u32, idx: usize) -> usize {
    let mut i = idx;
    while i < params_len - 1 && sgr_sep_is_set(sep_mask, i) {
        i += 1;
    }
    i + 1
}

pub(crate) unsafe fn parse_sgr_direct_color(
    params: *const u16,
    params_len: usize,
    sep_mask: u32,
    start: usize,
    colon: bool,
) -> Option<(usize, u8, u8, u8)> {
    if start + 4 >= params_len {
        return None;
    }
    if !colon {
        unsafe {
            return Some((
                start + 5,
                ptr::read(params.add(start + 2)) as u8,
                ptr::read(params.add(start + 3)) as u8,
                ptr::read(params.add(start + 4)) as u8,
            ));
        }
    }

    let count = count_sgr_colon(params_len, sep_mask, start + 1);
    unsafe {
        match count {
            3 => Some((
                start + 5,
                ptr::read(params.add(start + 2)) as u8,
                ptr::read(params.add(start + 3)) as u8,
                ptr::read(params.add(start + 4)) as u8,
            )),
            4 if start + 5 < params_len => Some((
                start + 6,
                ptr::read(params.add(start + 3)) as u8,
                ptr::read(params.add(start + 4)) as u8,
                ptr::read(params.add(start + 5)) as u8,
            )),
            _ => None,
        }
    }
}

pub(crate) fn count_sgr_colon(params_len: usize, sep_mask: u32, idx: usize) -> usize {
    let mut count = 0usize;
    let mut i = idx;
    while i < params_len - 1 && sgr_sep_is_set(sep_mask, i) {
        count += 1;
        i += 1;
    }
    count
}
