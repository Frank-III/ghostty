use crate::osc_types::*;

use crate::bytes_util::{bytes_to_str, find_byte, subslice, subslice_from};

#[inline]
fn bytes_eq(data: &[u8], start: usize, needle: &[u8]) -> bool {
    if start + needle.len() > data.len() {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if unsafe { *data.get_unchecked(start + i) } != unsafe { *needle.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

/// Parse OSC 1: change window icon.
///
/// `payload` is the data after `1;`.
#[inline]
pub fn parse_change_window_icon(payload: &[u8]) -> Command<'_> {
    Command::ChangeWindowIcon(bytes_to_str(payload))
}

/// Parse OSC 0 / OSC 2: change window title.
///
/// `payload` is the data after `0;` or `2;`.
#[inline]
pub fn parse_change_window_title(payload: &[u8]) -> Command<'_> {
    Command::ChangeWindowTitle(bytes_to_str(payload))
}

/// Parse OSC 7: report current working directory.
///
/// `payload` is the data after `7;`.
#[inline]
pub fn parse_report_pwd(payload: &[u8]) -> Command<'_> {
    Command::ReportPwd(ReportPwdOsc {
        value: bytes_to_str(payload),
    })
}

/// Parse OSC 22: mouse shape (pointer cursor).
///
/// `payload` is the data after `22;`.
#[inline]
pub fn parse_mouse_shape(payload: &[u8]) -> Command<'_> {
    Command::MouseShape(MouseShapeOsc {
        value: bytes_to_str(payload),
    })
}

/// Parse OSC 777: rxvt extension (show desktop notification).
///
/// `payload` is the data after `777;`. Expected format: `notify;Title;Body`.
pub fn parse_rxvt_extension(payload: &[u8]) -> Command<'_> {
    let semi1_pos = match find_byte(payload, b';') {
        Some(p) => p,
        None => return Command::Invalid,
    };

    let ext = subslice(payload, 0, semi1_pos);
    if ext.len() != 6 || !bytes_eq(ext, 0, b"notify") {
        return Command::Invalid;
    }

    let rest = subslice_from(payload, semi1_pos + 1);
    let semi2_pos = match find_byte(rest, b';') {
        Some(p) => p,
        None => return Command::Invalid,
    };

    let title = bytes_to_str(subslice(rest, 0, semi2_pos));
    let body = bytes_to_str(subslice_from(rest, semi2_pos + 1));

    Command::ShowDesktopNotification(ShowDesktopNotificationOsc { title, body })
}
