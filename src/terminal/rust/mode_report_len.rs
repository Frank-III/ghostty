use crate::simple::*;

pub(crate) fn mode_report_len(value: u64, ansi: bool, state: u64) -> usize {
    b"\x1B[".len()
        + if ansi { 0 } else { 1 }
        + decimal_len(value)
        + 1
        + decimal_len(state)
        + b"$y".len()
}
