use crate::mouse_setopt_size_write::mouse_encoder_setopt_size_write;

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
        mouse_encoder_setopt_size_state(
            screen_width,
            screen_height,
            cell_width,
            cell_height,
            padding_top,
            padding_bottom,
            padding_right,
            padding_left,
            out_screen_width,
            out_screen_height,
            out_cell_width,
            out_cell_height,
            out_padding_top,
            out_padding_bottom,
            out_padding_right,
            out_padding_left,
            last_cell_present,
        );
    }
}

pub(crate) unsafe fn mouse_encoder_setopt_size_state(
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
        mouse_encoder_setopt_size_write(
            screen_width,
            screen_height,
            cell_width,
            cell_height,
            padding_top,
            padding_bottom,
            padding_right,
            padding_left,
            out_screen_width,
            out_screen_height,
            out_cell_width,
            out_cell_height,
            out_padding_top,
            out_padding_bottom,
            out_padding_right,
            out_padding_left,
            last_cell_present,
        );
    }
}
