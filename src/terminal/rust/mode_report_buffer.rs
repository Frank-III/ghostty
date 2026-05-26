pub(crate) fn mode_report_buffer_ready(out: *mut u8, out_len: usize, total: usize) -> bool {
    !out.is_null() && out_len >= total
}
