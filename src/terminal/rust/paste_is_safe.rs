use crate::paste_safe::*;

pub(crate) unsafe fn paste_is_safe(data: *const u8, len: usize) -> bool {
    if data.is_null() {
        return true;
    }

    unsafe { paste_data_is_safe(data, len) }
}
