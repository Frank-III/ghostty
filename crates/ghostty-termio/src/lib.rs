//! PTY and subprocess I/O for the Ghostty Rust port (Phase 3).
//!
//! Port targets:
//! - `src/pty.zig`
//! - `src/Command.zig`
//! - `src/os/shell.zig`
//! - `src/termio/` (Exec, Termio thread — harness started in `harness.rs`)

pub mod command;
pub mod harness;
pub mod mode;
pub mod pty;
pub mod shell;
pub mod termio;
pub mod vt_sink;
pub mod winsize;

#[cfg(unix)]
pub mod spawn;

pub use command::{process_exit_from_wait_status, CommandBuildError, CommandBuilder, CommandSpec, ProcessExit};
pub use harness::{TermioHarness, TermioSink};
pub use vt_sink::{VtResizeFn, VtSink, VtWriteFn};
pub use mode::PtyMode;
pub use shell::{shell_escape, ShellCommandBuilder};
pub use termio::{TermioMailbox, TermioMessage};
pub use winsize::Winsize;
pub use ghostty_foundation::{FoundationError, FoundationResult};

#[cfg(unix)]
pub use spawn::{spawn_on_pty, spawn_pty_command, SpawnError, SpawnPtyError};

#[cfg(unix)]
pub use pty::{PosixPty, PtyOpenError};

#[cfg(not(unix))]
pub use pty::PtyOpenError;
