#![allow(unused)]

use crate::early::*;
use crate::constants::*;
use crate::vt_parser::ParserCsi;
use crate::stream_types::*;
use crate::stream_handler::StreamHandler;
use crate::stream_core::Stream;
use crate::ansi::{CursorStyle, StatusDisplay, ModifyKeyFormat, ProtectedMode};
use crate::csi::{EraseDisplay, EraseLine, TabClear, SizeReportStyle};
use crate::mode_def::{ModeTag, mode_find_index, mode_tag_from_index};
use crate::kitty_key::KittyKeyFlags;
use crate::device_attributes::DeviceAttributeReq;
use crate::device_status::DeviceStatusRequest;
use crate::mouse_shape::MouseShape;

// ============================================================================
// Param helpers
// ============================================================================

#[inline]
fn param(csi: &ParserCsi, idx: usize, default: u16) -> u16 {
    if idx < csi.params_len as usize {
        csi.params[idx]
    } else {
        default
    }
}

#[inline]
fn single_param(csi: &ParserCsi, default: u16) -> Option<u16> {
    match csi.params_len {
        0 => Some(default),
        1 => Some(csi.params[0]),
        _ => None,
    }
}

#[inline]
fn params_slice(csi: &ParserCsi) -> &[u16] {
    &csi.params[..csi.params_len as usize]
}

#[inline]
fn no_intermediates(csi: &ParserCsi) -> bool {
    csi.intermediates_len == 0
}

#[inline]
fn intermediate_byte(csi: &ParserCsi, idx: usize) -> Option<u8> {
    if idx < csi.intermediates_len as usize {
        Some(csi.intermediates[idx])
    } else {
        None
    }
}

#[inline]
fn emit<H: StreamHandler>(stream: &mut Stream<H>, action: StreamAction<'static>) {
    stream.handler.on_action(action);
}

// ============================================================================
// SGR param-to-tag mapping (simplified; deferred complex color parsing)
// ============================================================================

fn sgr_param_to_tag(p: u16) -> SgrAttributeTag {
    match p {
        0 => SgrAttributeTag::None,
        1 => SgrAttributeTag::Bold,
        2 => SgrAttributeTag::Dim,
        3 => SgrAttributeTag::Italic,
        4 => SgrAttributeTag::Underline,
        5 => SgrAttributeTag::Blink,
        6 => SgrAttributeTag::RapidBlink,
        7 => SgrAttributeTag::Reverse,
        8 => SgrAttributeTag::Hidden,
        9 => SgrAttributeTag::Strikethrough,
        10..=19 => SgrAttributeTag::Font,
        21 => SgrAttributeTag::DoubleUnderline,
        22..=29 => SgrAttributeTag::None,
        30..=37 => SgrAttributeTag::FgColor,
        38 => SgrAttributeTag::FgColor,
        39 => SgrAttributeTag::FgColorReset,
        40..=47 => SgrAttributeTag::BgColor,
        48 => SgrAttributeTag::BgColor,
        49 => SgrAttributeTag::BgColorReset,
        53 => SgrAttributeTag::Overline,
        58 => SgrAttributeTag::UnderlineColor,
        59 => SgrAttributeTag::UnderlineColorReset,
        90..=97 => SgrAttributeTag::FgColorBright,
        100..=107 => SgrAttributeTag::BgColorBright,
        _ => SgrAttributeTag::None,
    }
}

fn dispatch_sgr<H: StreamHandler>(stream: &mut Stream<H>, csi: &ParserCsi) {
    let plen = csi.params_len as usize;
    if plen == 0 {
        emit(stream, StreamAction::SetAttribute(SgrAttribute::default()));
        return;
    }

    let mut i: usize = 0;
    while i < plen {
        let p = csi.params[i];
        let tag = sgr_param_to_tag(p);
        let mut value: u32 = p as u32;
        let mut skip: usize = 1;

        match p {
            38 | 48 | 58 => {
                if i + 1 < plen {
                    match csi.params[i + 1] {
                        5 if i + 2 < plen => {
                            value = csi.params[i + 2] as u32;
                            skip = 3;
                        }
                        2 if i + 4 < plen => {
                            let r = csi.params[i + 2] as u32;
                            let g = csi.params[i + 3] as u32;
                            let b = csi.params[i + 4] as u32;
                            value = (r << 16) | (g << 8) | b;
                            skip = 5;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        emit(stream, StreamAction::SetAttribute(SgrAttribute { tag, value }));
        i += skip;
    }
}

fn dispatch_modify_key_format<H: StreamHandler>(stream: &mut Stream<H>, csi: &ParserCsi) {
    let plen = csi.params_len as usize;
    if plen == 0 {
        emit(stream, StreamAction::ModifyKeyFormat(ModifyKeyFormat::LEGACY));
        return;
    }

    let mut format = match csi.params[0] {
        0 => ModifyKeyFormat::LEGACY,
        1 => ModifyKeyFormat::CURSOR_KEYS,
        2 => ModifyKeyFormat::FUNCTION_KEYS,
        4 => ModifyKeyFormat::OTHER_KEYS_NONE,
        _ => return,
    };

    if plen > 2 {
        return;
    }

    if plen == 2 {
        if matches!(format, ModifyKeyFormat::OTHER_KEYS_NONE) && csi.params[1] == 2 {
            format = ModifyKeyFormat::OTHER_KEYS_NUMERIC;
        }
    }

    emit(stream, StreamAction::ModifyKeyFormat(format));
}

fn dispatch_mode<H: StreamHandler>(stream: &mut Stream<H>, csi: &ParserCsi, set: bool) {
    let ansi = match csi.intermediates_len {
        0 => true,
        1 if csi.intermediates[0] == b'?' => false,
        _ => return,
    };

    for &mode_int in params_slice(csi) {
        if let Some(idx) = mode_find_index(mode_int, ansi) {
            let mode = mode_tag_from_index(idx);
            if set {
                emit(stream, StreamAction::SetMode(Mode { mode }));
            } else {
                emit(stream, StreamAction::ResetMode(Mode { mode }));
            }
        }
    }
}

fn erase_protected(csi: &ParserCsi) -> Option<bool> {
    match csi.intermediates_len {
        0 => Some(false),
        1 if csi.intermediates[0] == b'?' => Some(true),
        _ => None,
    }
}

// ============================================================================
// Main dispatch
// ============================================================================

pub fn csi_dispatch<H: StreamHandler>(stream: &mut Stream<H>, csi: &ParserCsi) {
    match csi.final_byte {
        // CUU - Cursor Up (also 'k')
        b'A' | b'k' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorUp(CursorMovement { value: v })),
                None => {}
            }
        }

        // CUD - Cursor Down
        b'B' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorDown(CursorMovement { value: v })),
                None => {}
            }
        }

        // CUF - Cursor Right
        b'C' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorRight(CursorMovement { value: v })),
                None => {}
            }
        }

        // CUB - Cursor Left (also 'j')
        b'D' | b'j' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorLeft(CursorMovement { value: v })),
                None => {}
            }
        }

        // CNL - Cursor Next Line
        b'E' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => {
                    emit(stream, StreamAction::CursorDown(CursorMovement { value: v }));
                    emit(stream, StreamAction::CarriageReturn);
                }
                None => {}
            }
        }

        // CPL - Cursor Previous Line
        b'F' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => {
                    emit(stream, StreamAction::CursorUp(CursorMovement { value: v }));
                    emit(stream, StreamAction::CarriageReturn);
                }
                None => {}
            }
        }

        // HPA - Cursor Horizontal Position Absolute (also '`')
        b'G' | b'`' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorCol(CursorMovement { value: v })),
                None => {}
            }
        }

        // CUP - Cursor Position (also HVP 'f')
        b'H' | b'f' => {
            if !no_intermediates(csi) {
                return;
            }
            let pos = match csi.params_len {
                0 => CursorPos { row: 1, col: 1 },
                1 => CursorPos { row: csi.params[0], col: 1 },
                2 => CursorPos { row: csi.params[0], col: csi.params[1] },
                _ => return,
            };
            emit(stream, StreamAction::CursorPos(pos));
        }

        // CHT - Cursor Horizontal Tabulation
        b'I' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::HorizontalTab(v)),
                None => {}
            }
        }

        // ED - Erase Display
        b'J' => {
            let protected = match erase_protected(csi) {
                Some(v) => v,
                None => return,
            };
            let mode = match csi.params_len {
                0 => EraseDisplay::Below,
                1 => match EraseDisplay::from_u8(csi.params[0] as u8) {
                    Some(m) => m,
                    None => return,
                },
                _ => return,
            };
            match mode {
                EraseDisplay::Below => emit(stream, StreamAction::EraseDisplayBelow(protected)),
                EraseDisplay::Above => emit(stream, StreamAction::EraseDisplayAbove(protected)),
                EraseDisplay::Complete => emit(stream, StreamAction::EraseDisplayComplete(protected)),
                EraseDisplay::Scrollback => emit(stream, StreamAction::EraseDisplayScrollback(protected)),
                EraseDisplay::ScrollComplete => emit(stream, StreamAction::EraseDisplayScrollComplete(protected)),
            }
        }

        // EL - Erase Line
        b'K' => {
            let protected = match erase_protected(csi) {
                Some(v) => v,
                None => return,
            };
            let mode = match csi.params_len {
                0 => EraseLine::Right,
                1 if csi.params[0] < 5 => match EraseLine::from_u8(csi.params[0] as u8) {
                    Some(m) => m,
                    None => return,
                },
                _ => return,
            };
            match mode {
                EraseLine::Right => emit(stream, StreamAction::EraseLineRight(protected)),
                EraseLine::Left => emit(stream, StreamAction::EraseLineLeft(protected)),
                EraseLine::Complete => emit(stream, StreamAction::EraseLineComplete(protected)),
                EraseLine::RightUnlessPendingWrap => {
                    emit(stream, StreamAction::EraseLineRightUnlessPendingWrap(protected))
                }
            }
        }

        // IL - Insert Lines
        b'L' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::InsertLines(v as usize)),
                None => {}
            }
        }

        // DL - Delete Lines
        b'M' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::DeleteLines(v as usize)),
                None => {}
            }
        }

        // DCH - Delete Characters
        b'P' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::DeleteChars(v as usize)),
                None => {}
            }
        }

        // SU - Scroll Up
        b'S' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::ScrollUp(v as usize)),
                None => {}
            }
        }

        // SD - Scroll Down
        b'T' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::ScrollDown(v as usize)),
                None => {}
            }
        }

        // Cursor Tabulation Control
        b'W' => {
            if !no_intermediates(csi) && !(csi.intermediates_len == 1 && csi.intermediates[0] == b'?') {
                return;
            }
            if no_intermediates(csi) {
                if csi.params_len == 0 || (csi.params_len == 1 && csi.params[0] == 0) {
                    emit(stream, StreamAction::TabSet);
                    return;
                }
                if csi.params_len == 1 {
                    match csi.params[0] {
                        2 => {
                            emit(stream, StreamAction::TabClearCurrent);
                            return;
                        }
                        5 => {
                            emit(stream, StreamAction::TabClearAll);
                            return;
                        }
                        _ => {}
                    }
                }
                return;
            }
            // intermediates[0] == '?'
            if csi.params_len == 1 && csi.params[0] == 5 {
                emit(stream, StreamAction::TabReset);
            }
        }

        // ECH - Erase Characters
        b'X' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::EraseChars(v as usize)),
                None => {}
            }
        }

        // CBT - Cursor Backward Tabulation
        b'Z' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::HorizontalTabBack(v)),
                None => {}
            }
        }

        // HPR - Cursor Horizontal Position Relative
        b'a' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorColRelative(CursorMovement { value: v })),
                None => {}
            }
        }

        // REP - Repeat Previous Char
        b'b' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::PrintRepeat(v as usize)),
                None => {}
            }
        }

        // DA - Device Attributes
        b'c' => {
            let req = match csi.intermediates_len {
                0 => DeviceAttributeReq::Primary,
                1 => match csi.intermediates[0] {
                    b'>' => DeviceAttributeReq::Secondary,
                    b'=' => DeviceAttributeReq::Tertiary,
                    _ => return,
                },
                _ => return,
            };
            emit(stream, StreamAction::DeviceAttributes(req));
        }

        // VPA - Cursor Vertical Position Absolute
        b'd' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorRow(CursorMovement { value: v })),
                None => {}
            }
        }

        // VPR - Cursor Vertical Position Relative
        b'e' => {
            if !no_intermediates(csi) {
                return;
            }
            match single_param(csi, 1) {
                Some(v) => emit(stream, StreamAction::CursorRowRelative(CursorMovement { value: v })),
                None => {}
            }
        }

        // TBC - Tab Clear
        b'g' => {
            if !no_intermediates(csi) {
                return;
            }
            if csi.params_len != 1 {
                return;
            }
            match TabClear::from_u8(csi.params[0] as u8) {
                Some(TabClear::Current) => emit(stream, StreamAction::TabClearCurrent),
                Some(TabClear::All) => emit(stream, StreamAction::TabClearAll),
                _ => {}
            }
        }

        // SM - Set Mode
        b'h' => dispatch_mode(stream, csi, true),

        // RM - Reset Mode
        b'l' => dispatch_mode(stream, csi, false),

        // SGR - Select Graphic Rendition
        b'm' => {
            if no_intermediates(csi) {
                dispatch_sgr(stream, csi);
            } else if csi.intermediates_len == 1 && csi.intermediates[0] == b'>' {
                dispatch_modify_key_format(stream, csi);
            }
        }

        // DSR - Device Status Report
        b'n' => {
            if no_intermediates(csi) || (csi.intermediates_len == 1 && csi.intermediates[0] == b'?') {
                if csi.params_len != 1 {
                    return;
                }
                let question = csi.intermediates_len == 1;
                match DeviceStatusRequest::from_int(csi.params[0], question) {
                    Some(req) => {
                        emit(stream, StreamAction::DeviceStatus(DeviceStatus {
                            request: csi.params[0],
                            question,
                        }));
                    }
                    None => {}
                }
                return;
            }
            if csi.intermediates_len == 1 && csi.intermediates[0] == b'>' {
                emit(stream, StreamAction::ModifyKeyFormat(ModifyKeyFormat::OTHER_KEYS_NUMERIC_EXCEPT));
            }
        }

        // DECRQM - Request Mode
        b'p' => {
            if csi.intermediates_len < 1 {
                return;
            }
            let ansi = if csi.intermediates_len == 1 && csi.intermediates[0] == b'$' {
                true
            } else if csi.intermediates_len == 2
                && csi.intermediates[0] == b'?'
                && csi.intermediates[1] == b'$'
            {
                false
            } else {
                return;
            };

            if csi.params_len != 1 {
                return;
            }
            let mode_raw = csi.params[0];
            match mode_find_index(mode_raw, ansi) {
                Some(idx) => {
                    let mode = mode_tag_from_index(idx);
                    emit(stream, StreamAction::RequestMode(Mode { mode }));
                }
                None => {
                    emit(stream, StreamAction::RequestModeUnknown(RawMode {
                        mode: mode_raw,
                        ansi,
                    }));
                }
            }
        }

        // DECSCUSR / DECSCA / XTVERSION
        b'q' => {
            if csi.intermediates_len != 1 {
                return;
            }
            match csi.intermediates[0] {
                // DECSCUSR - Cursor Style
                b' ' => {
                    let style = match csi.params_len {
                        0 => CursorStyle::DEFAULT,
                        1 => match csi.params[0] {
                            0 => CursorStyle::DEFAULT,
                            1 => CursorStyle::BLINKING_BLOCK,
                            2 => CursorStyle::STEADY_BLOCK,
                            3 => CursorStyle::BLINKING_UNDERLINE,
                            4 => CursorStyle::STEADY_UNDERLINE,
                            5 => CursorStyle::BLINKING_BAR,
                            6 => CursorStyle::STEADY_BAR,
                            _ => return,
                        },
                        _ => return,
                    };
                    emit(stream, StreamAction::CursorStyle(style));
                }

                // DECSCA - Protected Mode
                b'"' => {
                    let mode = match csi.params_len {
                        0 => ProtectedMode::OFF,
                        1 => match csi.params[0] {
                            0 | 2 => ProtectedMode::OFF,
                            1 => ProtectedMode::DEC,
                            _ => return,
                        },
                        _ => return,
                    };
                    match mode {
                        ProtectedMode::OFF => emit(stream, StreamAction::ProtectedModeOff),
                        ProtectedMode::ISO => emit(stream, StreamAction::ProtectedModeIso),
                        ProtectedMode::DEC => emit(stream, StreamAction::ProtectedModeDec),
                    }
                }

                // XTVERSION
                b'>' => emit(stream, StreamAction::Xtversion),

                _ => {}
            }
        }

        // DECSTBM - Set Top and Bottom Margins / Restore Mode
        b'r' => {
            if no_intermediates(csi) {
                let margin = match csi.params_len {
                    0 => Margin { top_left: 0, bottom_right: 0 },
                    1 => Margin { top_left: csi.params[0], bottom_right: 0 },
                    2 => Margin { top_left: csi.params[0], bottom_right: csi.params[1] },
                    _ => return,
                };
                emit(stream, StreamAction::TopAndBottomMargin(margin));
                return;
            }
            if csi.intermediates_len == 1 && csi.intermediates[0] == b'?' {
                for &mode_int in params_slice(csi) {
                    if let Some(idx) = mode_find_index(mode_int, false) {
                        let mode = mode_tag_from_index(idx);
                        emit(stream, StreamAction::RestoreMode(Mode { mode }));
                    }
                }
            }
        }

        // DECSLRM / Save Mode / XTSHIFTESCAPE
        b's' => {
            if no_intermediates(csi) {
                match csi.params_len {
                    0 => emit(stream, StreamAction::LeftAndRightMarginAmbiguous),
                    1 => emit(stream, StreamAction::LeftAndRightMargin(Margin {
                        top_left: csi.params[0],
                        bottom_right: 0,
                    })),
                    2 => emit(stream, StreamAction::LeftAndRightMargin(Margin {
                        top_left: csi.params[0],
                        bottom_right: csi.params[1],
                    })),
                    _ => {}
                }
                return;
            }
            if csi.intermediates_len == 1 {
                match csi.intermediates[0] {
                    b'?' => {
                        for &mode_int in params_slice(csi) {
                            if let Some(idx) = mode_find_index(mode_int, false) {
                                let mode = mode_tag_from_index(idx);
                                emit(stream, StreamAction::SaveMode(Mode { mode }));
                            }
                        }
                    }
                    // XTSHIFTESCAPE
                    b'>' => {
                        let capture = match csi.params_len {
                            0 => false,
                            1 => match csi.params[0] {
                                0 => false,
                                1 => true,
                                _ => return,
                            },
                            _ => return,
                        };
                        emit(stream, StreamAction::MouseShiftCapture(capture));
                    }
                    _ => {}
                }
            }
        }

        // XTWINOPS - Window Operations / Size Reports / Title Push/Pop
        b't' => {
            if !no_intermediates(csi) {
                return;
            }
            if csi.params_len == 0 {
                return;
            }
            match csi.params[0] {
                14 if csi.params_len == 1 => {
                    emit(stream, StreamAction::SizeReport(SizeReportStyle::Csi14t));
                }
                16 if csi.params_len == 1 => {
                    emit(stream, StreamAction::SizeReport(SizeReportStyle::Csi16t));
                }
                18 if csi.params_len == 1 => {
                    emit(stream, StreamAction::SizeReport(SizeReportStyle::Csi18t));
                }
                21 if csi.params_len == 1 => {
                    emit(stream, StreamAction::SizeReport(SizeReportStyle::Csi21t));
                }
                22 | 23 if (csi.params_len == 2 || csi.params_len == 3)
                    && (csi.params[1] == 0 || csi.params[1] == 2) =>
                {
                    let index = if csi.params_len == 3 { csi.params[2] } else { 0 };
                    if csi.params[0] == 22 {
                        emit(stream, StreamAction::TitlePush(index));
                    } else {
                        emit(stream, StreamAction::TitlePop(index));
                    }
                }
                _ => {}
            }
        }

        // Restore Cursor / Kitty Keyboard Protocol
        b'u' => {
            if no_intermediates(csi) {
                emit(stream, StreamAction::RestoreCursor);
                return;
            }
            if csi.intermediates_len != 1 {
                return;
            }
            match csi.intermediates[0] {
                // Query
                b'?' => {
                    emit(stream, StreamAction::KittyKeyboardQuery);
                }

                // Push
                b'>' => {
                    let flags = if csi.params_len == 1 {
                        let raw = csi.params[0];
                        if raw > 31 { return; }
                        raw as u8
                    } else {
                        0
                    };
                    emit(stream, StreamAction::KittyKeyboardPush(KittyKeyboardFlags { flags }));
                }

                // Pop
                b'<' => {
                    let number = if csi.params_len == 1 { csi.params[0] } else { 1 };
                    emit(stream, StreamAction::KittyKeyboardPop(number));
                }

                // Set / SetOr / SetNot
                b'=' => {
                    if csi.params_len < 1 {
                        return;
                    }
                    let raw = csi.params[0];
                    if raw > 31 { return; }
                    let flags = raw as u8;
                    let number: u16 = if csi.params_len >= 2 { csi.params[1] } else { 1 };
                    let kbf = KittyKeyboardFlags { flags };
                    match number {
                        1 => emit(stream, StreamAction::KittyKeyboardSet(kbf)),
                        2 => emit(stream, StreamAction::KittyKeyboardSetOr(kbf)),
                        3 => emit(stream, StreamAction::KittyKeyboardSetNot(kbf)),
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        // ICH - Insert Blanks
        b'@' => {
            if !no_intermediates(csi) {
                return;
            }
            let val = match csi.params_len {
                0 => 1usize,
                1 => {
                    let v = csi.params[0] as usize;
                    if v < 1 { 1 } else { v }
                }
                _ => return,
            };
            emit(stream, StreamAction::InsertBlanks(val));
        }

        // DECSASD - Select Active Status Display
        b'}' => {
            if csi.intermediates_len != 1 || csi.intermediates[0] != b'$' {
                return;
            }
            if csi.params_len != 1 {
                return;
            }
            let display = match csi.params[0] {
                0 => StatusDisplay::MAIN,
                1 => StatusDisplay::STATUS_LINE,
                _ => return,
            };
            emit(stream, StreamAction::ActiveStatusDisplay(display));
        }

        _ => {}
    }
}
