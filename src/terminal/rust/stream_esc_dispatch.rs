#![allow(unused)]

use crate::ansi::*;
use crate::bytes_util::subslice_len;
use crate::charsets::*;
use crate::constants::*;
use crate::device_attributes::DeviceAttributeReq;
use crate::early::*;
use crate::mode_def::ModeTag;
use crate::stream_core::Stream;
use crate::stream_handler::StreamHandler;
use crate::stream_types::*;
use crate::vt_parser::*;

const MODE_KEYPAD_KEYS: ModeTag = ModeTag {
    value: 66,
    ansi: false,
};

pub fn configure_charset<H: StreamHandler>(
    stream: &mut Stream<H>,
    intermediates: &[u8],
    charset: CharsetId,
) {
    if intermediates.len() != 1 {
        return;
    }

    let slot = match intermediates[0] {
        b'(' => CharsetSlot::G0,
        b')' => CharsetSlot::G1,
        b'*' => CharsetSlot::G2,
        b'+' => CharsetSlot::G3,
        _ => return,
    };

    stream
        .handler
        .on_action(StreamAction::ConfigureCharset(ConfigureCharset {
            slot,
            charset,
        }));
}

pub fn esc_dispatch<H: StreamHandler>(stream: &mut Stream<H>, esc: &ParserEsc) {
    match esc.final_byte {
        b'B' => configure_charset(stream, esc_intermediates(esc), CharsetId::Ascii),
        b'A' => configure_charset(stream, esc_intermediates(esc), CharsetId::British),
        b'0' => configure_charset(stream, esc_intermediates(esc), CharsetId::DecSpecial),

        b'7' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::SaveCursor),
            _ => return,
        },

        b'8' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::RestoreCursor),
            1 => match esc.intermediates[0] {
                b'#' => stream.handler.on_action(StreamAction::Decaln),
                _ => return,
            },
            _ => return,
        },

        b'D' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::Index),
            _ => return,
        },

        b'E' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::NextLine),
            _ => return,
        },

        b'H' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::TabSet),
            _ => return,
        },

        b'M' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::ReverseIndex),
            _ => return,
        },

        b'N' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GL,
                    charset: CharsetSlot::G2,
                    locking: true,
                })),
            _ => return,
        },

        b'O' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GL,
                    charset: CharsetSlot::G3,
                    locking: true,
                })),
            _ => return,
        },

        b'V' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::ProtectedModeIso),
            _ => return,
        },

        b'W' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::ProtectedModeOff),
            _ => return,
        },

        b'Z' => {
            if esc.intermediates_len == 0 {
                stream
                    .handler
                    .on_action(StreamAction::DeviceAttributes(DeviceAttributeReq::Primary));
            }
        }

        b'c' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::FullReset),
            _ => return,
        },

        b'n' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GL,
                    charset: CharsetSlot::G2,
                    locking: false,
                })),
            _ => return,
        },

        b'o' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GL,
                    charset: CharsetSlot::G3,
                    locking: false,
                })),
            _ => return,
        },

        b'~' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GR,
                    charset: CharsetSlot::G1,
                    locking: false,
                })),
            _ => return,
        },

        b'}' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GR,
                    charset: CharsetSlot::G2,
                    locking: false,
                })),
            _ => return,
        },

        b'|' => match esc.intermediates_len {
            0 => stream
                .handler
                .on_action(StreamAction::InvokeCharset(InvokeCharset {
                    bank: ActiveSlot::GR,
                    charset: CharsetSlot::G3,
                    locking: false,
                })),
            _ => return,
        },

        b'=' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::SetMode(Mode {
                mode: MODE_KEYPAD_KEYS,
            })),
            _ => return,
        },

        b'>' => match esc.intermediates_len {
            0 => stream.handler.on_action(StreamAction::ResetMode(Mode {
                mode: MODE_KEYPAD_KEYS,
            })),
            _ => return,
        },

        b'\\' => {}

        _ => return,
    }
}

fn esc_intermediates(esc: &ParserEsc) -> &[u8] {
    subslice_len(&esc.intermediates, esc.intermediates_len as usize)
}
