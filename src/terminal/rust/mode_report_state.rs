use core::ffi::c_int;

pub(crate) fn mode_report_state_valid(state: c_int) -> bool {
    (0..=4).contains(&state)
}
