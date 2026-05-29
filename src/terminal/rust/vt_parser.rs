use crate::constants::*;
use crate::early::*;

pub const MAX_INTERMEDIATE: usize = 4;
pub const MAX_PARAMS: usize = 24;
pub const MAX_OSC_BUF: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Ground = 0,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiIntermediate,
    CsiParam,
    CsiIgnore,
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsPassthrough,
    DcsIgnore,
    OscString,
    SosPmApcString,
}

const NUM_STATES: usize = 14;

impl State {
    fn from_u8(v: u8) -> Self {
        if (v as usize) < NUM_STATES {
            unsafe { core::mem::transmute(v) }
        } else {
            State::Ground
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransitionAction {
    None = 0,
    Ignore,
    Print,
    Execute,
    Collect,
    Param,
    EscDispatch,
    CsiDispatch,
    Put,
    OscPut,
    ApcPut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub state: State,
    pub action: TransitionAction,
}

#[derive(Debug, Clone, Copy)]
pub struct ParserCsi {
    pub intermediates: [u8; MAX_INTERMEDIATE],
    pub intermediates_len: u8,
    pub params: [u16; MAX_PARAMS],
    pub params_sep: u32,
    pub params_len: u8,
    pub final_byte: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParserEsc {
    pub intermediates: [u8; MAX_INTERMEDIATE],
    pub intermediates_len: u8,
    pub final_byte: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParserDcs {
    pub intermediates: [u8; MAX_INTERMEDIATE],
    pub intermediates_len: u8,
    pub params: [u16; MAX_PARAMS],
    pub params_len: u8,
    pub final_byte: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ParserOsc {
    pub data: [u8; MAX_OSC_BUF],
    pub data_len: u16,
    pub terminator: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum ParserActionTag {
    Print = 0,
    Execute,
    CsiDispatch,
    EscDispatch,
    OscDispatch,
    DcsHook,
    DcsPut,
    DcsUnhook,
    ApcStart,
    ApcPut,
    ApcEnd,
    None,
}

pub struct ParserAction {
    pub tag: ParserActionTag,
    pub byte: u8,
    pub csi: ParserCsi,
    pub esc: ParserEsc,
    pub dcs: ParserDcs,
    pub osc: ParserOsc,
}

impl ParserAction {
    fn none_action() -> Self {
        Self {
            tag: ParserActionTag::None,
            byte: 0,
            csi: unsafe { core::mem::zeroed() },
            esc: unsafe { core::mem::zeroed() },
            dcs: unsafe { core::mem::zeroed() },
            osc: unsafe { core::mem::zeroed() },
        }
    }
}

pub struct ParserActions {
    pub actions: [ParserAction; 3],
}

pub struct VtParser {
    pub state: State,
    pub intermediates: [u8; MAX_INTERMEDIATE],
    pub intermediates_idx: u8,
    pub params: [u16; MAX_PARAMS],
    pub params_sep: u32,
    pub params_idx: u8,
    pub param_acc: u16,
    pub param_acc_idx: u8,
    pub osc_buf: [u8; MAX_OSC_BUF],
    pub osc_len: u16,
}

impl VtParser {
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            intermediates: [0u8; MAX_INTERMEDIATE],
            intermediates_idx: 0,
            params: [0u16; MAX_PARAMS],
            params_sep: 0,
            params_idx: 0,
            param_acc: 0,
            param_acc_idx: 0,
            osc_buf: [0u8; MAX_OSC_BUF],
            osc_len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.intermediates_idx = 0;
        self.params_idx = 0;
        self.params_sep = 0;
        self.param_acc = 0;
        self.param_acc_idx = 0;
    }

    fn reset_osc(&mut self) {
        self.osc_len = 0;
    }

    fn osc_put(&mut self, c: u8) {
        let idx = self.osc_len as usize;
        if idx < MAX_OSC_BUF {
            unsafe {
                *self.osc_buf.get_unchecked_mut(idx) = c;
            }
            self.osc_len += 1;
        }
    }

    fn osc_end(&mut self, c: u8) -> Option<ParserAction> {
        let osc = ParserOsc {
            data: self.osc_buf,
            data_len: self.osc_len.min(MAX_OSC_BUF as u16),
            terminator: c,
        };
        Some(ParserAction {
            tag: ParserActionTag::OscDispatch,
            byte: 0,
            csi: unsafe { core::mem::zeroed() },
            esc: unsafe { core::mem::zeroed() },
            dcs: unsafe { core::mem::zeroed() },
            osc,
        })
    }

    fn collect(&mut self, c: u8) {
        let idx = self.intermediates_idx as usize;
        if idx >= MAX_INTERMEDIATE {
            return;
        }
        unsafe {
            *self.intermediates.get_unchecked_mut(idx) = c;
        }
        self.intermediates_idx += 1;
    }

    fn do_param(&mut self, c: u8) {
        if c == b';' || c == b':' {
            let idx = self.params_idx as usize;
            if idx >= MAX_PARAMS {
                return;
            }
            unsafe {
                *self.params.get_unchecked_mut(idx) = self.param_acc;
            }
            if c == b':' {
                self.params_sep |= 1u32 << (self.params_idx as u32);
            }
            self.params_idx += 1;
            self.param_acc = 0;
            self.param_acc_idx = 0;
            return;
        }
        if c >= b'0' && c <= b'9' {
            self.param_acc = self
                .param_acc
                .wrapping_mul(10)
                .wrapping_add((c - b'0') as u16);
            self.param_acc_idx = 1;
        }
    }

    fn make_csi(&self, c: u8) -> Option<ParserAction> {
        if (self.params_idx as usize) >= MAX_PARAMS {
            return None;
        }
        let mut params = [0u16; MAX_PARAMS];
        let mut plen = self.params_idx;
        let psep = self.params_sep;
        for i in 0..(plen as usize) {
            unsafe {
                *params.get_unchecked_mut(i) = *self.params.get_unchecked(i);
            }
        }
        if self.param_acc_idx > 0 {
            unsafe {
                *params.get_unchecked_mut(plen as usize) = self.param_acc;
            }
            plen += 1;
        }
        if c != b'm' && psep != 0 {
            return None;
        }
        let mut intermediates = [0u8; MAX_INTERMEDIATE];
        for i in 0..(self.intermediates_idx as usize) {
            unsafe {
                *intermediates.get_unchecked_mut(i) = *self.intermediates.get_unchecked(i);
            }
        }
        let csi = ParserCsi {
            intermediates,
            intermediates_len: self.intermediates_idx,
            params,
            params_sep: psep,
            params_len: plen,
            final_byte: c,
        };
        Some(ParserAction {
            tag: ParserActionTag::CsiDispatch,
            byte: 0,
            csi,
            esc: unsafe { core::mem::zeroed() },
            dcs: unsafe { core::mem::zeroed() },
            osc: unsafe { core::mem::zeroed() },
        })
    }

    fn make_esc(&self, c: u8) -> ParserAction {
        let mut intermediates = [0u8; MAX_INTERMEDIATE];
        for i in 0..(self.intermediates_idx as usize) {
            unsafe {
                *intermediates.get_unchecked_mut(i) = *self.intermediates.get_unchecked(i);
            }
        }
        let esc = ParserEsc {
            intermediates,
            intermediates_len: self.intermediates_idx,
            final_byte: c,
        };
        ParserAction {
            tag: ParserActionTag::EscDispatch,
            byte: 0,
            csi: unsafe { core::mem::zeroed() },
            esc,
            dcs: unsafe { core::mem::zeroed() },
            osc: unsafe { core::mem::zeroed() },
        }
    }

    fn make_dcs_hook(&self, c: u8) -> Option<ParserAction> {
        if (self.params_idx as usize) >= MAX_PARAMS {
            return None;
        }
        let mut params = [0u16; MAX_PARAMS];
        let mut plen = self.params_idx;
        for i in 0..(plen as usize) {
            unsafe {
                *params.get_unchecked_mut(i) = *self.params.get_unchecked(i);
            }
        }
        if self.param_acc_idx > 0 {
            unsafe {
                *params.get_unchecked_mut(plen as usize) = self.param_acc;
            }
            plen += 1;
        }
        let mut intermediates = [0u8; MAX_INTERMEDIATE];
        for i in 0..(self.intermediates_idx as usize) {
            unsafe {
                *intermediates.get_unchecked_mut(i) = *self.intermediates.get_unchecked(i);
            }
        }
        let dcs = ParserDcs {
            intermediates,
            intermediates_len: self.intermediates_idx,
            params,
            params_len: plen,
            final_byte: c,
        };
        Some(ParserAction {
            tag: ParserActionTag::DcsHook,
            byte: 0,
            csi: unsafe { core::mem::zeroed() },
            esc: unsafe { core::mem::zeroed() },
            dcs,
            osc: unsafe { core::mem::zeroed() },
        })
    }

    fn do_action(&mut self, action: TransitionAction, c: u8) -> Option<ParserAction> {
        match action {
            TransitionAction::None | TransitionAction::Ignore => None,
            TransitionAction::Print => Some(ParserAction {
                tag: ParserActionTag::Print,
                byte: c,
                csi: unsafe { core::mem::zeroed() },
                esc: unsafe { core::mem::zeroed() },
                dcs: unsafe { core::mem::zeroed() },
                osc: unsafe { core::mem::zeroed() },
            }),
            TransitionAction::Execute => Some(ParserAction {
                tag: ParserActionTag::Execute,
                byte: c,
                csi: unsafe { core::mem::zeroed() },
                esc: unsafe { core::mem::zeroed() },
                dcs: unsafe { core::mem::zeroed() },
                osc: unsafe { core::mem::zeroed() },
            }),
            TransitionAction::Collect => {
                self.collect(c);
                None
            }
            TransitionAction::Param => {
                self.do_param(c);
                None
            }
            TransitionAction::OscPut => {
                self.osc_put(c);
                None
            }
            TransitionAction::CsiDispatch => self.make_csi(c),
            TransitionAction::EscDispatch => Some(self.make_esc(c)),
            TransitionAction::Put => Some(ParserAction {
                tag: ParserActionTag::DcsPut,
                byte: c,
                csi: unsafe { core::mem::zeroed() },
                esc: unsafe { core::mem::zeroed() },
                dcs: unsafe { core::mem::zeroed() },
                osc: unsafe { core::mem::zeroed() },
            }),
            TransitionAction::ApcPut => Some(ParserAction {
                tag: ParserActionTag::ApcPut,
                byte: c,
                csi: unsafe { core::mem::zeroed() },
                esc: unsafe { core::mem::zeroed() },
                dcs: unsafe { core::mem::zeroed() },
                osc: unsafe { core::mem::zeroed() },
            }),
        }
    }

    pub fn next(&mut self, c: u8) -> [Option<ParserAction>; 3] {
        let effect = unsafe {
            *TABLE
                .get_unchecked(c as usize)
                .get_unchecked(self.state as usize)
        };
        let next_state = effect.state;
        let action = effect.action;

        let exit_action: Option<ParserAction> = if self.state == next_state {
            None
        } else {
            match self.state {
                State::OscString => {
                    if c == 0x07 || c == 0x1B || c == 0x9C {
                        self.osc_end(c)
                    } else {
                        None
                    }
                }
                State::DcsPassthrough => Some(ParserAction {
                    tag: ParserActionTag::DcsUnhook,
                    byte: 0,
                    csi: unsafe { core::mem::zeroed() },
                    esc: unsafe { core::mem::zeroed() },
                    dcs: unsafe { core::mem::zeroed() },
                    osc: unsafe { core::mem::zeroed() },
                }),
                State::SosPmApcString => Some(ParserAction {
                    tag: ParserActionTag::ApcEnd,
                    byte: 0,
                    csi: unsafe { core::mem::zeroed() },
                    esc: unsafe { core::mem::zeroed() },
                    dcs: unsafe { core::mem::zeroed() },
                    osc: unsafe { core::mem::zeroed() },
                }),
                _ => None,
            }
        };

        let transition_action = self.do_action(action, c);

        let entry_action: Option<ParserAction> = if self.state == next_state {
            None
        } else {
            match next_state {
                State::Escape | State::DcsEntry | State::CsiEntry => {
                    self.clear();
                    None
                }
                State::OscString => {
                    self.reset_osc();
                    None
                }
                State::DcsPassthrough => {
                    if (self.params_idx as usize) >= MAX_PARAMS {
                        None
                    } else if self.param_acc_idx > 0 {
                        let idx = self.params_idx as usize;
                        unsafe {
                            *self.params.get_unchecked_mut(idx) = self.param_acc;
                        }
                        self.params_idx += 1;
                        let hook = ParserAction {
                            tag: ParserActionTag::DcsHook,
                            byte: 0,
                            csi: unsafe { core::mem::zeroed() },
                            esc: unsafe { core::mem::zeroed() },
                            dcs: ParserDcs {
                                intermediates: self.intermediates,
                                intermediates_len: self.intermediates_idx,
                                params: self.params,
                                params_len: self.params_idx,
                                final_byte: c,
                            },
                            osc: unsafe { core::mem::zeroed() },
                        };
                        Some(hook)
                    } else {
                        self.make_dcs_hook(c)
                    }
                }
                State::SosPmApcString => Some(ParserAction {
                    tag: ParserActionTag::ApcStart,
                    byte: 0,
                    csi: unsafe { core::mem::zeroed() },
                    esc: unsafe { core::mem::zeroed() },
                    dcs: unsafe { core::mem::zeroed() },
                    osc: unsafe { core::mem::zeroed() },
                }),
                _ => None,
            }
        };

        self.state = next_state;

        [exit_action, transition_action, entry_action]
    }
}

impl Default for VtParser {
    fn default() -> Self {
        Self::new()
    }
}

const T: Transition = Transition {
    state: State::Ground,
    action: TransitionAction::None,
};

const fn t(s: State, a: TransitionAction) -> Transition {
    Transition {
        state: s,
        action: a,
    }
}

const TABLE: [[Transition; NUM_STATES]; 256] = {
    let mut table = [T; NUM_STATES * 256];
    let g = State::Ground;
    let esc = State::Escape;
    let esc_inter = State::EscapeIntermediate;
    let csi_entry = State::CsiEntry;
    let csi_inter = State::CsiIntermediate;
    let csi_param = State::CsiParam;
    let csi_ignore = State::CsiIgnore;
    let dcs_entry = State::DcsEntry;
    let dcs_param = State::DcsParam;
    let dcs_inter = State::DcsIntermediate;
    let dcs_pass = State::DcsPassthrough;
    let dcs_ignore = State::DcsIgnore;
    let osc_string = State::OscString;
    let spa_string = State::SosPmApcString;

    let none = TransitionAction::None;
    let ig = TransitionAction::Ignore;
    let pr = TransitionAction::Print;
    let ex = TransitionAction::Execute;
    let co = TransitionAction::Collect;
    let pa = TransitionAction::Param;
    let ed = TransitionAction::EscDispatch;
    let cd = TransitionAction::CsiDispatch;
    let pu = TransitionAction::Put;
    let op = TransitionAction::OscPut;
    let ap = TransitionAction::ApcPut;

    let mut i: usize = 0;
    while i < 256 {
        let mut s: usize = 0;
        while s < NUM_STATES {
            table[i * NUM_STATES + s] = t(g, none);
            s += 1;
        }
        i += 1;
    }

    // "anywhere" transitions (applied first, overridden by specific states below)
    // These are handled by applying to all states, then overriding per-state below.
    // Actually in the Zig code, specific state transitions override "anywhere" ones
    // because they're set after. We need to set "anywhere" first, then override.

    // anywhere => ground (execute)
    i = 0;
    while i < NUM_STATES {
        table[0x18 * NUM_STATES + i] = t(g, ex);
        table[0x1A * NUM_STATES + i] = t(g, ex);
        table[0x99 * NUM_STATES + i] = t(g, ex);
        table[0x9A * NUM_STATES + i] = t(g, ex);
        table[0x9C * NUM_STATES + i] = t(g, none);

        // anywhere => escape
        table[0x1B * NUM_STATES + i] = t(esc, none);

        // anywhere => sos_pm_apc_string
        table[0x98 * NUM_STATES + i] = t(spa_string, none);
        table[0x9E * NUM_STATES + i] = t(spa_string, none);
        table[0x9F * NUM_STATES + i] = t(spa_string, none);

        // anywhere => csi_entry
        table[0x9B * NUM_STATES + i] = t(csi_entry, none);

        // anywhere => dcs_entry
        table[0x90 * NUM_STATES + i] = t(dcs_entry, none);

        // anywhere => osc_string
        table[0x9D * NUM_STATES + i] = t(osc_string, none);
        i += 1;
    }

    // anywhere: ranges 0x80-0x8F, 0x91-0x97 => ground, execute
    i = 0x80;
    while i <= 0x8F {
        let mut s: usize = 0;
        while s < NUM_STATES {
            table[i * NUM_STATES + s] = t(g, ex);
            s += 1;
        }
        i += 1;
    }
    i = 0x91;
    while i <= 0x97 {
        let mut s: usize = 0;
        while s < NUM_STATES {
            table[i * NUM_STATES + s] = t(g, ex);
            s += 1;
        }
        i += 1;
    }

    // ground state
    table[0x19 * NUM_STATES + (g as usize)] = t(g, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + (g as usize)] = t(g, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + (g as usize)] = t(g, ex);
        i += 1;
    }
    i = 0x20;
    while i <= 0x7F {
        table[i * NUM_STATES + (g as usize)] = t(g, pr);
        i += 1;
    }

    // escape_intermediate
    let si = esc_inter as usize;
    table[0x19 * NUM_STATES + si] = t(esc_inter, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + si] = t(esc_inter, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + si] = t(esc_inter, ex);
        i += 1;
    }
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + si] = t(esc_inter, co);
        i += 1;
    }
    table[0x7F * NUM_STATES + si] = t(esc_inter, ig);
    i = 0x30;
    while i <= 0x7E {
        table[i * NUM_STATES + si] = t(g, ed);
        i += 1;
    }

    // sos_pm_apc_string
    let ss = spa_string as usize;
    table[0x19 * NUM_STATES + ss] = t(spa_string, ap);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + ss] = t(spa_string, ap);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + ss] = t(spa_string, ap);
        i += 1;
    }
    i = 0x20;
    while i <= 0x7F {
        table[i * NUM_STATES + ss] = t(spa_string, ap);
        i += 1;
    }

    // escape
    let se = esc as usize;
    table[0x19 * NUM_STATES + se] = t(esc, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + se] = t(esc, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + se] = t(esc, ex);
        i += 1;
    }
    table[0x7F * NUM_STATES + se] = t(esc, ig);
    i = 0x30;
    while i <= 0x4F {
        table[i * NUM_STATES + se] = t(g, ed);
        i += 1;
    }
    i = 0x51;
    while i <= 0x57 {
        table[i * NUM_STATES + se] = t(g, ed);
        i += 1;
    }
    i = 0x60;
    while i <= 0x7E {
        table[i * NUM_STATES + se] = t(g, ed);
        i += 1;
    }
    table[0x59 * NUM_STATES + se] = t(g, ed);
    table[0x5A * NUM_STATES + se] = t(g, ed);
    table[0x5C * NUM_STATES + se] = t(g, ed);
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + se] = t(esc_inter, co);
        i += 1;
    }
    table[0x58 * NUM_STATES + se] = t(spa_string, none);
    table[0x5E * NUM_STATES + se] = t(spa_string, none);
    table[0x5F * NUM_STATES + se] = t(spa_string, none);
    table[0x50 * NUM_STATES + se] = t(dcs_entry, none);
    table[0x5B * NUM_STATES + se] = t(csi_entry, none);
    table[0x5D * NUM_STATES + se] = t(osc_string, none);

    // dcs_entry
    let sd = dcs_entry as usize;
    table[0x19 * NUM_STATES + sd] = t(dcs_entry, ig);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sd] = t(dcs_entry, ig);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sd] = t(dcs_entry, ig);
        i += 1;
    }
    table[0x7F * NUM_STATES + sd] = t(dcs_entry, ig);
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + sd] = t(dcs_inter, co);
        i += 1;
    }
    table[0x3A * NUM_STATES + sd] = t(dcs_ignore, none);
    i = 0x30;
    while i <= 0x39 {
        table[i * NUM_STATES + sd] = t(dcs_param, pa);
        i += 1;
    }
    table[0x3B * NUM_STATES + sd] = t(dcs_param, pa);
    i = 0x3C;
    while i <= 0x3F {
        table[i * NUM_STATES + sd] = t(dcs_param, co);
        i += 1;
    }
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + sd] = t(dcs_pass, none);
        i += 1;
    }

    // dcs_intermediate
    let sdi = dcs_inter as usize;
    table[0x19 * NUM_STATES + sdi] = t(dcs_inter, ig);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sdi] = t(dcs_inter, ig);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sdi] = t(dcs_inter, ig);
        i += 1;
    }
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + sdi] = t(dcs_inter, co);
        i += 1;
    }
    table[0x7F * NUM_STATES + sdi] = t(dcs_inter, ig);
    i = 0x30;
    while i <= 0x3F {
        table[i * NUM_STATES + sdi] = t(dcs_ignore, none);
        i += 1;
    }
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + sdi] = t(dcs_pass, none);
        i += 1;
    }

    // dcs_ignore
    let sdig = dcs_ignore as usize;
    table[0x19 * NUM_STATES + sdig] = t(dcs_ignore, ig);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sdig] = t(dcs_ignore, ig);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sdig] = t(dcs_ignore, ig);
        i += 1;
    }

    // dcs_param
    let sdp = dcs_param as usize;
    table[0x19 * NUM_STATES + sdp] = t(dcs_param, ig);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sdp] = t(dcs_param, ig);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sdp] = t(dcs_param, ig);
        i += 1;
    }
    i = 0x30;
    while i <= 0x39 {
        table[i * NUM_STATES + sdp] = t(dcs_param, pa);
        i += 1;
    }
    table[0x3B * NUM_STATES + sdp] = t(dcs_param, pa);
    table[0x7F * NUM_STATES + sdp] = t(dcs_param, ig);
    table[0x3A * NUM_STATES + sdp] = t(dcs_ignore, none);
    i = 0x3C;
    while i <= 0x3F {
        table[i * NUM_STATES + sdp] = t(dcs_ignore, none);
        i += 1;
    }
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + sdp] = t(dcs_inter, co);
        i += 1;
    }
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + sdp] = t(dcs_pass, none);
        i += 1;
    }

    // dcs_passthrough
    let sdpass = dcs_pass as usize;
    table[0x19 * NUM_STATES + sdpass] = t(dcs_pass, pu);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sdpass] = t(dcs_pass, pu);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sdpass] = t(dcs_pass, pu);
        i += 1;
    }
    i = 0x20;
    while i <= 0x7E {
        table[i * NUM_STATES + sdpass] = t(dcs_pass, pu);
        i += 1;
    }
    table[0x7F * NUM_STATES + sdpass] = t(dcs_pass, ig);

    // csi_param
    let scp = csi_param as usize;
    table[0x19 * NUM_STATES + scp] = t(csi_param, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + scp] = t(csi_param, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + scp] = t(csi_param, ex);
        i += 1;
    }
    i = 0x30;
    while i <= 0x39 {
        table[i * NUM_STATES + scp] = t(csi_param, pa);
        i += 1;
    }
    table[0x3A * NUM_STATES + scp] = t(csi_param, pa);
    table[0x3B * NUM_STATES + scp] = t(csi_param, pa);
    table[0x7F * NUM_STATES + scp] = t(csi_param, ig);
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + scp] = t(g, cd);
        i += 1;
    }
    i = 0x3C;
    while i <= 0x3F {
        table[i * NUM_STATES + scp] = t(csi_ignore, none);
        i += 1;
    }
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + scp] = t(csi_inter, co);
        i += 1;
    }

    // csi_ignore
    let sci = csi_ignore as usize;
    table[0x19 * NUM_STATES + sci] = t(csi_ignore, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sci] = t(csi_ignore, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sci] = t(csi_ignore, ex);
        i += 1;
    }
    i = 0x20;
    while i <= 0x3F {
        table[i * NUM_STATES + sci] = t(csi_ignore, ig);
        i += 1;
    }
    table[0x7F * NUM_STATES + sci] = t(csi_ignore, ig);
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + sci] = t(g, none);
        i += 1;
    }

    // csi_intermediate
    let scint = csi_inter as usize;
    table[0x19 * NUM_STATES + scint] = t(csi_inter, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + scint] = t(csi_inter, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + scint] = t(csi_inter, ex);
        i += 1;
    }
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + scint] = t(csi_inter, co);
        i += 1;
    }
    table[0x7F * NUM_STATES + scint] = t(csi_inter, ig);
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + scint] = t(g, cd);
        i += 1;
    }
    i = 0x30;
    while i <= 0x3F {
        table[i * NUM_STATES + scint] = t(csi_ignore, none);
        i += 1;
    }

    // csi_entry
    let sce = csi_entry as usize;
    table[0x19 * NUM_STATES + sce] = t(csi_entry, ex);
    i = 0;
    while i <= 0x17 {
        table[i * NUM_STATES + sce] = t(csi_entry, ex);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sce] = t(csi_entry, ex);
        i += 1;
    }
    table[0x7F * NUM_STATES + sce] = t(csi_entry, ig);
    i = 0x40;
    while i <= 0x7E {
        table[i * NUM_STATES + sce] = t(g, cd);
        i += 1;
    }
    table[0x3A * NUM_STATES + sce] = t(csi_ignore, none);
    i = 0x20;
    while i <= 0x2F {
        table[i * NUM_STATES + sce] = t(csi_inter, co);
        i += 1;
    }
    i = 0x30;
    while i <= 0x39 {
        table[i * NUM_STATES + sce] = t(csi_param, pa);
        i += 1;
    }
    table[0x3B * NUM_STATES + sce] = t(csi_param, pa);
    i = 0x3C;
    while i <= 0x3F {
        table[i * NUM_STATES + sce] = t(csi_param, co);
        i += 1;
    }

    // osc_string
    let sos = osc_string as usize;
    table[0x19 * NUM_STATES + sos] = t(osc_string, ig);
    i = 0;
    while i <= 0x06 {
        table[i * NUM_STATES + sos] = t(osc_string, ig);
        i += 1;
    }
    i = 0x08;
    while i <= 0x17 {
        table[i * NUM_STATES + sos] = t(osc_string, ig);
        i += 1;
    }
    i = 0x1C;
    while i <= 0x1F {
        table[i * NUM_STATES + sos] = t(osc_string, ig);
        i += 1;
    }
    // OSC payload bytes (matches Zig parse_table: range 0x20..0xFF osc_put).
    i = 0x20;
    while i <= 0xFF {
        table[i * NUM_STATES + sos] = t(osc_string, op);
        if i == 0xFF {
            break;
        }
        i += 1;
    }
    // XTerm accepts BEL or ST for terminating OSC sequences.
    table[0x07 * NUM_STATES + sos] = t(g, none);
    // ST may begin with ESC (handled by the anywhere => escape transition).
    table[0x1B * NUM_STATES + sos] = t(esc, none);

    // Build 2D array from flat
    let mut result = [[T; NUM_STATES]; 256];
    let mut row: usize = 0;
    while row < 256 {
        let mut col: usize = 0;
        while col < NUM_STATES {
            result[row][col] = table[row * NUM_STATES + col];
            col += 1;
        }
        row += 1;
    }
    result
};
