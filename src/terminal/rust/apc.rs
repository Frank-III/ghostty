use crate::constants::*;
use crate::early::*;
use core::ffi::c_void;

pub const APC_KITTY_DEFAULT_MAX_BYTES: usize = 65 * 1024 * 1024;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApcProtocol {
    Kitty = 0,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApcMaxBytes {
    pub kitty: Option<usize>,
}

impl Default for ApcMaxBytes {
    fn default() -> Self {
        Self::init_full()
    }
}

impl ApcMaxBytes {
    pub const fn init_full() -> Self {
        Self {
            kitty: Some(APC_KITTY_DEFAULT_MAX_BYTES),
        }
    }

    pub const fn init_full_with(value: usize) -> Self {
        Self { kitty: Some(value) }
    }

    pub fn get(&self, protocol: ApcProtocol) -> Option<usize> {
        match protocol {
            ApcProtocol::Kitty => self.kitty,
        }
    }

    pub fn put(&mut self, protocol: ApcProtocol, value: usize) {
        match protocol {
            ApcProtocol::Kitty => self.kitty = Some(value),
        }
    }

    pub fn remove(&mut self, protocol: ApcProtocol) {
        match protocol {
            ApcProtocol::Kitty => self.kitty = None,
        }
    }

    pub fn set_all(&mut self, value: Option<usize>) {
        self.kitty = value;
    }
}

pub fn apc_protocol_default_max_bytes(protocol: ApcProtocol) -> usize {
    match protocol {
        ApcProtocol::Kitty => APC_KITTY_DEFAULT_MAX_BYTES,
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApcCommandTag {
    Kitty = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApcCommand {
    pub tag: ApcCommandTag,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApcStateTag {
    Inactive = 0,
    Ignore,
    Identify,
    Kitty,
}

#[derive(Debug, Clone, Copy)]
pub struct ApcState {
    pub tag: ApcStateTag,
    pub kitty_bytes: usize,
    pub kitty_max_bytes: usize,
    pub kitty_parser: *mut c_void,
}

impl Default for ApcState {
    fn default() -> Self {
        Self::inactive()
    }
}

impl ApcState {
    pub const fn inactive() -> Self {
        Self {
            tag: ApcStateTag::Inactive,
            kitty_bytes: 0,
            kitty_max_bytes: 0,
            kitty_parser: core::ptr::null_mut(),
        }
    }

    pub const fn ignore() -> Self {
        Self {
            tag: ApcStateTag::Ignore,
            kitty_bytes: 0,
            kitty_max_bytes: 0,
            kitty_parser: core::ptr::null_mut(),
        }
    }

    pub const fn identify() -> Self {
        Self {
            tag: ApcStateTag::Identify,
            kitty_bytes: 0,
            kitty_max_bytes: 0,
            kitty_parser: core::ptr::null_mut(),
        }
    }

    pub const fn kitty(max_bytes: usize, parser: *mut c_void) -> Self {
        Self {
            tag: ApcStateTag::Kitty,
            kitty_bytes: 0,
            kitty_max_bytes: max_bytes,
            kitty_parser: parser,
        }
    }
}

pub struct ApcHandler {
    pub state: ApcState,
    pub max_bytes: ApcMaxBytes,
}

impl Default for ApcHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ApcHandler {
    pub const fn new() -> Self {
        Self {
            state: ApcState::inactive(),
            max_bytes: ApcMaxBytes::init_full(),
        }
    }

    pub fn deinit(&mut self) {
        self.state = ApcState::inactive();
    }

    pub fn start(&mut self) {
        self.state = ApcState::identify();
    }

    pub fn feed(&mut self, byte: u8, kitty_enabled: bool) {
        match self.state.tag {
            ApcStateTag::Inactive => unsafe { core::hint::unreachable_unchecked() },
            ApcStateTag::Ignore => {}
            ApcStateTag::Identify => match byte {
                b'G' => {
                    if kitty_enabled {
                        let max = self
                            .max_bytes
                            .get(ApcProtocol::Kitty)
                            .unwrap_or_else(|| apc_protocol_default_max_bytes(ApcProtocol::Kitty));
                        self.state = ApcState::kitty(max, core::ptr::null_mut());
                    } else {
                        self.state = ApcState::ignore();
                    }
                }
                _ => {
                    self.state = ApcState::ignore();
                }
            },
            ApcStateTag::Kitty => {
                if !kitty_enabled {
                    unsafe { core::hint::unreachable_unchecked() }
                }
            }
        }
    }

    pub fn feed_kitty_byte(&mut self) -> bool {
        if self.state.tag != ApcStateTag::Kitty {
            return true;
        }
        if self.state.kitty_bytes >= self.state.kitty_max_bytes {
            self.state = ApcState::ignore();
            return false;
        }
        self.state.kitty_bytes += 1;
        true
    }

    pub fn end(&mut self) -> Option<ApcCommand> {
        let cmd = match self.state.tag {
            ApcStateTag::Inactive => unsafe { core::hint::unreachable_unchecked() },
            ApcStateTag::Ignore | ApcStateTag::Identify => None,
            ApcStateTag::Kitty => Some(ApcCommand {
                tag: ApcCommandTag::Kitty,
            }),
        };
        self.state = ApcState::inactive();
        cmd
    }

    pub fn end_no_command(&mut self) {
        self.state = ApcState::inactive();
    }
}
