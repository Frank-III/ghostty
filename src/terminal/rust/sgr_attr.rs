use core::ffi::c_int;
use core::ptr;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttySgrUnknown {
    pub(crate) full_ptr: *const u16,
    pub(crate) full_len: usize,
    pub(crate) partial_ptr: *const u16,
    pub(crate) partial_len: usize,
}

#[repr(C)]
pub union GhosttySgrAttributeValue {
    pub(crate) unknown: GhosttySgrUnknown,
    pub(crate) padding: [u64; 8],
}

#[repr(C)]
pub struct GhosttySgrAttribute {
    pub(crate) tag: c_int,
    pub(crate) value: GhosttySgrAttributeValue,
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_unknown_full(
    unknown: GhosttySgrUnknown,
    ptr_out: *mut *const u16,
) -> usize {
    if !ptr_out.is_null() {
        unsafe {
            ptr::write(ptr_out, unknown.full_ptr);
        }
    }

    unknown.full_len
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_unknown_partial(
    unknown: GhosttySgrUnknown,
    ptr_out: *mut *const u16,
) -> usize {
    if !ptr_out.is_null() {
        unsafe {
            ptr::write(ptr_out, unknown.partial_ptr);
        }
    }

    unknown.partial_len
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_attribute_tag(attr: GhosttySgrAttribute) -> c_int {
    attr.tag
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_sgr_attribute_value(
    attr: *mut GhosttySgrAttribute,
) -> *mut GhosttySgrAttributeValue {
    unsafe { core::ptr::addr_of_mut!((*attr).value) }
}
