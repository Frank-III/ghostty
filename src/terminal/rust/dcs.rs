use core::ffi::c_void;
use crate::early::*;
use crate::constants::*;
use crate::vt_parser::*;
use crate::tmux::*;

pub const DCS_DEFAULT_MAX_BYTES: usize = 1024 * 1024;
pub const DCS_DECRQSS_MAX_LEN: usize = 2;
pub const DCS_XTGETTCAP_INITIAL_CAPACITY: usize = 128;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcsDecrqss {
    None = 0,
    Sgr,
    Decscusr,
    Decstbm,
    Decslrm,
}

impl Default for DcsDecrqss {
    fn default() -> Self {
        Self::None
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcsCommandTag {
    XXTGETTCAP = 0,
    DECRQSS,
    Tmux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DcsCommand {
    pub tag: DcsCommandTag,
    pub decrqss: DcsDecrqss,
    pub data_ptr: *mut u8,
    pub data_len: usize,
}

impl Default for DcsCommand {
    fn default() -> Self {
        Self {
            tag: DcsCommandTag::XXTGETTCAP,
            decrqss: DcsDecrqss::None,
            data_ptr: core::ptr::null_mut(),
            data_len: 0,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcsStateTag {
    Inactive = 0,
    Ignore,
    XTGETTCAP,
    DECRQSS,
    Tmux,
}

#[derive(Debug, Clone, Copy)]
pub struct DcsDecrqssState {
    pub data: [u8; DCS_DECRQSS_MAX_LEN],
    pub len: u8,
}

impl Default for DcsDecrqssState {
    fn default() -> Self {
        Self {
            data: [0u8; DCS_DECRQSS_MAX_LEN],
            len: 0,
        }
    }
}

pub struct DcsState {
    pub tag: DcsStateTag,
    pub xtgettcap_buf: *mut u8,
    pub xtgettcap_len: usize,
    pub xtgettcap_cap: usize,
    pub decrqss: DcsDecrqssState,
    pub tmux_parser: *mut c_void,
}

impl Default for DcsState {
    fn default() -> Self {
        Self::inactive()
    }
}

impl DcsState {
    pub const fn inactive() -> Self {
        Self {
            tag: DcsStateTag::Inactive,
            xtgettcap_buf: core::ptr::null_mut(),
            xtgettcap_len: 0,
            xtgettcap_cap: 0,
            decrqss: DcsDecrqssState {
                data: [0u8; DCS_DECRQSS_MAX_LEN],
                len: 0,
            },
            tmux_parser: core::ptr::null_mut(),
        }
    }

    pub const fn ignore() -> Self {
        Self {
            tag: DcsStateTag::Ignore,
            xtgettcap_buf: core::ptr::null_mut(),
            xtgettcap_len: 0,
            xtgettcap_cap: 0,
            decrqss: DcsDecrqssState {
                data: [0u8; DCS_DECRQSS_MAX_LEN],
                len: 0,
            },
            tmux_parser: core::ptr::null_mut(),
        }
    }

    pub const fn xtgettcap(buf: *mut u8, cap: usize) -> Self {
        Self {
            tag: DcsStateTag::XTGETTCAP,
            xtgettcap_buf: buf,
            xtgettcap_len: 0,
            xtgettcap_cap: cap,
            decrqss: DcsDecrqssState {
                data: [0u8; DCS_DECRQSS_MAX_LEN],
                len: 0,
            },
            tmux_parser: core::ptr::null_mut(),
        }
    }

    pub const fn decrqss() -> Self {
        Self {
            tag: DcsStateTag::DECRQSS,
            xtgettcap_buf: core::ptr::null_mut(),
            xtgettcap_len: 0,
            xtgettcap_cap: 0,
            decrqss: DcsDecrqssState {
                data: [0u8; DCS_DECRQSS_MAX_LEN],
                len: 0,
            },
            tmux_parser: core::ptr::null_mut(),
        }
    }

    pub const fn tmux(parser: *mut c_void) -> Self {
        Self {
            tag: DcsStateTag::Tmux,
            xtgettcap_buf: core::ptr::null_mut(),
            xtgettcap_len: 0,
            xtgettcap_cap: 0,
            decrqss: DcsDecrqssState {
                data: [0u8; DCS_DECRQSS_MAX_LEN],
                len: 0,
            },
            tmux_parser: parser,
        }
    }
}

pub struct DcsHandler {
    pub state: DcsState,
    pub max_bytes: usize,
}

impl Default for DcsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DcsHandler {
    pub const fn new() -> Self {
        Self {
            state: DcsState::inactive(),
            max_bytes: DCS_DEFAULT_MAX_BYTES,
        }
    }

    pub fn discard(&mut self) {
        self.state = DcsState::inactive();
    }

    pub fn hook(
        &mut self,
        buf: *mut u8,
        buf_cap: usize,
        tmux_parser: *mut c_void,
        dcs: &ParserDcs,
        tmux_enabled: bool,
    ) -> Option<DcsCommand> {
        assert_eq!(self.state.tag, DcsStateTag::Inactive);

        self.state = DcsState::ignore();

        let hook = self.try_hook(buf, buf_cap, tmux_parser, dcs, tmux_enabled);

        match hook {
            Some((state, cmd)) => {
                self.state = state;
                cmd
            }
            None => None,
        }
    }

    fn try_hook(
        &self,
        buf: *mut u8,
        buf_cap: usize,
        tmux_parser: *mut c_void,
        dcs: &ParserDcs,
        tmux_enabled: bool,
    ) -> Option<(DcsState, Option<DcsCommand>)> {
        match dcs.intermediates_len {
            0 => {
                match dcs.final_byte {
                    b'p' => {
                        if !tmux_enabled {
                            return None;
                        }
                        if dcs.params_len != 1 || dcs.params[0] != 1000 {
                            return None;
                        }
                        Some((
                            DcsState::tmux(tmux_parser),
                            Some(DcsCommand {
                                tag: DcsCommandTag::Tmux,
                                ..Default::default()
                            }),
                        ))
                    }
                    _ => None,
                }
            }
            1 => {
                match dcs.intermediates[0] {
                    b'+' => {
                        match dcs.final_byte {
                            b'q' => Some((
                                DcsState::xtgettcap(buf, buf_cap),
                                None,
                            )),
                            _ => None,
                        }
                    }
                    b'$' => {
                        match dcs.final_byte {
                            b'q' => Some((
                                DcsState::decrqss(),
                                None,
                            )),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn put(&mut self, byte: u8) -> Option<DcsCommand> {
        match self.try_put(byte) {
            Ok(cmd) => cmd,
            Err(()) => {
                self.discard();
                self.state = DcsState::ignore();
                None
            }
        }
    }

    fn try_put(&mut self, byte: u8) -> Result<Option<DcsCommand>, ()> {
        match self.state.tag {
            DcsStateTag::Inactive | DcsStateTag::Ignore => Ok(None),
            DcsStateTag::Tmux => {
                Err(())
            }
            DcsStateTag::XTGETTCAP => {
                if self.state.xtgettcap_len >= self.state.xtgettcap_cap
                    || self.state.xtgettcap_len >= self.max_bytes
                {
                    return Err(());
                }
                unsafe {
                    *self.state.xtgettcap_buf.add(self.state.xtgettcap_len) = byte;
                }
                self.state.xtgettcap_len += 1;
                Ok(None)
            }
            DcsStateTag::DECRQSS => {
                if (self.state.decrqss.len as usize) >= DCS_DECRQSS_MAX_LEN {
                    return Err(());
                }
                self.state.decrqss.data[self.state.decrqss.len as usize] = byte;
                self.state.decrqss.len += 1;
                Ok(None)
            }
        }
    }

    pub fn unhook(&mut self) -> Option<DcsCommand> {
        let cmd = match self.state.tag {
            DcsStateTag::Inactive | DcsStateTag::Ignore => None,
            DcsStateTag::Tmux => {
                Some(DcsCommand {
                    tag: DcsCommandTag::Tmux,
                    ..Default::default()
                })
            }
            DcsStateTag::XTGETTCAP => {
                unsafe {
                    let buf = self.state.xtgettcap_buf;
                    if !buf.is_null() {
                        let len = self.state.xtgettcap_len;
                        for i in 0..len {
                            let b = *buf.add(i);
                            *buf.add(i) = ascii_to_upper(b);
                        }
                    }
                }
                Some(DcsCommand {
                    tag: DcsCommandTag::XXTGETTCAP,
                    data_ptr: self.state.xtgettcap_buf,
                    data_len: self.state.xtgettcap_len,
                    ..Default::default()
                })
            }
            DcsStateTag::DECRQSS => {
                let decrqss = match self.state.decrqss.len {
                    0 => DcsDecrqss::None,
                    1 => match self.state.decrqss.data[0] {
                        b'm' => DcsDecrqss::Sgr,
                        b'r' => DcsDecrqss::Decstbm,
                        b's' => DcsDecrqss::Decslrm,
                        _ => DcsDecrqss::None,
                    },
                    2 => match self.state.decrqss.data[0] {
                        b' ' => match self.state.decrqss.data[1] {
                            b'q' => DcsDecrqss::Decscusr,
                            _ => DcsDecrqss::None,
                        },
                        _ => DcsDecrqss::None,
                    },
                    _ => DcsDecrqss::None,
                };
                Some(DcsCommand {
                    tag: DcsCommandTag::DECRQSS,
                    decrqss,
                    ..Default::default()
                })
            }
        };
        self.state = DcsState::inactive();
        cmd
    }
}

fn ascii_to_upper(b: u8) -> u8 {
    if b >= b'a' && b <= b'z' {
        b - 32
    } else {
        b
    }
}

pub struct XTGETTCAPIterator {
    pub data: *const u8,
    pub len: usize,
    pub pos: usize,
}

impl XTGETTCAPIterator {
    pub const fn new(data: *const u8, len: usize) -> Self {
        Self { data, len, pos: 0 }
    }
}

impl Iterator for XTGETTCAPIterator {
    type Item = (*const u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        unsafe {
            let remaining_len = self.len - self.pos;
            let remaining = self.data.add(self.pos);
            let mut idx = 0usize;
            while idx < remaining_len {
                if *remaining.add(idx) == b';' {
                    break;
                }
                idx += 1;
            }
            self.pos += idx + 1;
            Some((remaining, idx))
        }
    }
}
