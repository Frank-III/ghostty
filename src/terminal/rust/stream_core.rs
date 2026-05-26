#![allow(unused)]
use crate::early::*;
use crate::constants::*;
use crate::ansi::*;
use crate::stream_handler::*;
use crate::stream_types::*;
use crate::vt_parser::*;
use crate::utf8_decoder::*;
use crate::charsets::*;
use crate::mode_def::*;

pub struct Stream<H: StreamHandler> {
    pub(crate) handler: H,
    parser: VtParser,
    utf8decoder: Utf8Decoder,
}

impl<H: StreamHandler> Stream<H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            parser: VtParser::new(),
            utf8decoder: Utf8Decoder::new(),
        }
    }

    pub fn into_handler(self) -> H {
        self.handler
    }

    #[inline(always)]
    pub fn next(&mut self, c: u8) {
        if self.parser.state == State::Ground {
            self.next_utf8(c);
            return;
        }
        self.next_non_utf8(c);
    }

    #[inline(always)]
    fn next_utf8(&mut self, c: u8) {
        let (codepoint, consumed) = self.utf8decoder.next(c);
        if let Some(cp) = codepoint {
            self.handle_codepoint(cp);
        }
        if !consumed {
            let (retry_cp, retry_consumed) = self.utf8decoder.next(c);
            debug_assert!(retry_consumed);
            if let Some(cp) = retry_cp {
                self.handle_codepoint(cp);
            }
        }
    }

    #[inline(always)]
    fn handle_codepoint(&mut self, c: u32) {
        if c <= 0xF {
            self.execute(c as u8);
            return;
        }
        if c == 0x1B {
            self.parser.state = State::Escape;
            self.parser.clear();
            return;
        }
        self.print(c);
    }

    fn next_non_utf8(&mut self, c: u8) {
        debug_assert!(self.parser.state != State::Ground);

        if self.parser.state == State::Escape && c == b'[' {
            self.parser.state = State::CsiEntry;
            return;
        }

        if self.parser.state == State::CsiParam {
            let handled = match c {
                0x00..=0x0F => { self.execute(c); true }
                0x10..=0x17 | 0x19 | 0x1C..=0x1F => true,
                0x18 | 0x1A => { self.parser.state = State::Ground; true }
                b'0'..=b'9' => {
                    if (self.parser.params_idx as usize) < MAX_PARAMS {
                        self.parser.param_acc = self.parser.param_acc.saturating_mul(10);
                        self.parser.param_acc = self.parser.param_acc.saturating_add((c - b'0') as u16);
                        self.parser.param_acc_idx |= 1;
                    }
                    true
                }
                b':' | b';' => {
                    if (self.parser.params_idx as usize) < MAX_PARAMS {
                        self.parser.params[self.parser.params_idx as usize] = self.parser.param_acc;
                        if c == b':' {
                            self.parser.params_sep |= 1u32 << (self.parser.params_idx as u32);
                        }
                        self.parser.params_idx += 1;
                        self.parser.param_acc = 0;
                        self.parser.param_acc_idx = 0;
                    }
                    true
                }
                0x7F => true,
                _ => false,
            };
            if handled {
                return;
            }
        }

        let actions = self.parser.next(c);

        for action_opt in actions {
            let Some(action) = action_opt else { continue };

            if self.handler.on_raw_action(action.tag) {
                continue;
            }

            match action.tag {
                ParserActionTag::Print => self.print(action.byte as u32),
                ParserActionTag::Execute => self.execute(action.byte),
                ParserActionTag::CsiDispatch => self.csi_dispatch(action.csi),
                ParserActionTag::EscDispatch => self.esc_dispatch(action.esc),
                ParserActionTag::OscDispatch => self.osc_dispatch(action.osc),
                ParserActionTag::DcsHook => self.handler.on_dcs_hook(action.dcs),
                ParserActionTag::DcsPut => self.handler.on_dcs_put(action.byte),
                ParserActionTag::DcsUnhook => self.handler.on_dcs_unhook(),
                ParserActionTag::ApcStart => self.handler.on_apc_start(),
                ParserActionTag::ApcPut => self.handler.on_apc_put(action.byte),
                ParserActionTag::ApcEnd => self.handler.on_apc_end(),
                ParserActionTag::None => {}
            }
        }
    }

    #[inline(always)]
    fn print(&mut self, cp: u32) {
        self.handler.on_print(cp, 0, false);
    }

    #[inline(always)]
    fn execute(&mut self, c: u8) {
        if c > 0x7F {
            self.esc_dispatch(ParserEsc {
                intermediates: [0u8; MAX_INTERMEDIATE],
                intermediates_len: 0,
                final_byte: c - 0x40,
            });
            return;
        }

        let c0 = C0::from_u8(c);
        match c0 {
            C0::NUL | C0::SOH | C0::STX => {}
            C0::ENQ => self.handler.on_enquiry(),
            C0::BEL => self.handler.on_bell(),
            C0::BS => self.handler.on_backspace(),
            C0::HT => self.handler.on_horizontal_tab(1),
            C0::LF | C0::VT | C0::FF => self.handler.on_linefeed(),
            C0::CR => self.handler.on_carriage_return(),
            C0::SO => self.handler.on_invoke_charset(InvokeCharset {
                bank: ActiveSlot::GL,
                charset: CharsetSlot::G1,
                locking: false,
            }),
            C0::SI => self.handler.on_invoke_charset(InvokeCharset {
                bank: ActiveSlot::GL,
                charset: CharsetSlot::G0,
                locking: false,
            }),
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, csi: ParserCsi) {
        crate::stream_csi_dispatch::csi_dispatch(self, &csi);
    }

    fn esc_dispatch(&mut self, esc: ParserEsc) {
        crate::stream_esc_dispatch::esc_dispatch(self, &esc);
    }

    fn osc_dispatch(&mut self, osc: ParserOsc) {
        let cmd = crate::stream_osc_parse::parse(&osc);
        crate::stream_osc_dispatch::osc_dispatch(&mut self.handler, cmd);
    }
}
