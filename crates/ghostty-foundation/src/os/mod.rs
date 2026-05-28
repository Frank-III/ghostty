//! OS helpers ported from `src/os/`.

pub mod env;
pub mod string_encoding;
pub mod uri;

pub use env::{append_env, append_env_always, prepend_env};
pub use string_encoding::{
    printf_q_decode, url_percent_decode, url_percent_encode, DecodeError,
};
pub use uri::is_valid_mac_address;
