use core::ptr;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_mouse_encoder_setopt_size(
    screen_width: u32,
    screen_height: u32,
    cell_width: u32,
    cell_height: u32,
    padding_top: u32,
    padding_bottom: u32,
    padding_right: u32,
    padding_left: u32,
    out_screen_width: *mut u32,
    out_screen_height: *mut u32,
    out_cell_width: *mut u32,
    out_cell_height: *mut u32,
    out_padding_top: *mut u32,
    out_padding_bottom: *mut u32,
    out_padding_right: *mut u32,
    out_padding_left: *mut u32,
    last_cell_present: *mut bool,
) {
    unsafe {
        if !out_screen_width.is_null() {
            ptr::write(out_screen_width, screen_width);
        }
        if !out_screen_height.is_null() {
            ptr::write(out_screen_height, screen_height);
        }
        if !out_cell_width.is_null() {
            ptr::write(out_cell_width, cell_width);
        }
        if !out_cell_height.is_null() {
            ptr::write(out_cell_height, cell_height);
        }
        if !out_padding_top.is_null() {
            ptr::write(out_padding_top, padding_top);
        }
        if !out_padding_bottom.is_null() {
            ptr::write(out_padding_bottom, padding_bottom);
        }
        if !out_padding_right.is_null() {
            ptr::write(out_padding_right, padding_right);
        }
        if !out_padding_left.is_null() {
            ptr::write(out_padding_left, padding_left);
        }
        if !last_cell_present.is_null() {
            ptr::write(last_cell_present, false);
        }
    }
}
