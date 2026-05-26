use core::ptr;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_parser_reset(idx: *mut usize) {
    if idx.is_null() {
        return;
    }

    unsafe {
        ptr::write(idx, 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_params_sep_mask(seps: *const u8, len: usize) -> u32 {
    if seps.is_null() {
        return 0;
    }

    let mut mask = 0u32;
    let mut i = 0usize;
    while i < len && i < 24 {
        let sep = unsafe { ptr::read(seps.add(i)) };
        if sep == b':' {
            mask |= 1u32 << i;
        }
        i += 1;
    }
    mask
}
