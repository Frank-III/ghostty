use core::ptr;

pub(crate) fn paste_strip_byte(byte: u8) -> bool {
    matches!(
        byte,
        0x00 | 0x08
            | 0x05
            | 0x04
            | 0x1B
            | 0x7F
            | 0x03
            | 0x1C
            | 0x15
            | 0x1A
            | 0x11
            | 0x13
            | 0x17
            | 0x16
            | 0x12
            | 0x0F
    )
}

pub(crate) unsafe fn matches_bytes_at(
    data: *const u8,
    len: usize,
    offset: usize,
    bytes: &[u8],
) -> bool {
    if len - offset < bytes.len() {
        return false;
    }

    let mut i = 0usize;
    while i < bytes.len() {
        let actual = unsafe { ptr::read(data.add(offset + i)) };
        let expected = unsafe { ptr::read(bytes.as_ptr().add(i)) };
        if actual != expected {
            return false;
        }
        i += 1;
    }

    true
}

pub(crate) unsafe fn copy_data_bytes(out: *mut u8, offset: &mut usize, data: *const u8, len: usize) {
    let mut i = 0usize;
    while i < len {
        let byte = unsafe { ptr::read(data.add(i)) };
        unsafe {
            ptr::write(out.add(*offset + i), byte);
        }
        i += 1;
    }
    *offset += len;
}
