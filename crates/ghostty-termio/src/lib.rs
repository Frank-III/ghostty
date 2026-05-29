//! PTY and subprocess I/O for the Ghostty Rust port (Phase 3).
//!
//! Port targets:
//! - `src/pty.zig`
//! - `src/Command.zig`
//! - `src/os/shell.zig`
//! - `src/termio/` (Exec, Termio thread — harness started in `harness.rs`)

pub mod command;
#[cfg(unix)]
pub mod exec;
pub mod harness;
pub mod mode;
pub mod pty;
pub mod shell;
pub mod termio;
pub mod vt_sink;
pub mod winsize;

#[cfg(unix)]
pub mod exec_config;

#[cfg(unix)]
pub mod r#loop;

#[cfg(unix)]
pub mod stream_handler;

#[cfg(unix)]
pub mod surface_mailbox;

#[cfg(unix)]
pub mod renderer_mailbox;

#[cfg(unix)]
pub mod shell_integration;

#[cfg(unix)]
pub mod spawn;

#[cfg(unix)]
pub mod stream;

#[cfg(unix)]
pub mod thread;

pub use command::{
    process_exit_from_wait_status, CommandBuildError, CommandBuilder, CommandSpec, ProcessExit,
};
pub use ghostty_foundation::{FoundationError, FoundationResult};
pub use harness::{TermioHarness, TermioSink};
pub use mode::PtyMode;
pub use shell::{shell_escape, ShellCommandBuilder};
pub use termio::{TermioMailbox, TermioMessage};
pub use vt_sink::{VtResizeFn, VtSink, VtWriteFn};
pub use winsize::Winsize;

#[cfg(unix)]
pub use exec::{ChildWatcher, ExecSpawn};
#[cfg(unix)]
pub use exec_config::{command_from_termio_config, is_abnormal_exit};
#[cfg(unix)]
pub use r#loop::{TermioDrain, TermioLoop};
#[cfg(unix)]
pub use stream::PtyStream;
#[cfg(unix)]
pub use thread::{TermioThreadEvent, TermioThreadHandle};

#[cfg(unix)]
pub use stream_handler::StreamHandler;
#[cfg(unix)]
pub use surface_mailbox::{SurfaceMailbox, SurfaceMessage};
#[cfg(unix)]
pub use renderer_mailbox::{RendererMailbox, RendererMessage};
#[cfg(unix)]
pub use shell_integration::{
    apply_shell_integration, detect_shell, IntegratedShell, ShellIntegrationContext,
};
#[cfg(unix)]
pub use spawn::{spawn_on_pty, spawn_pty_command, SpawnError, SpawnPtyError};

#[cfg(feature = "rust-vt")]
pub use vt_sink::rust_owned::RustOwnedTerminalSink;

#[cfg(unix)]
pub use pty::{PosixPty, PtyOpenError};

#[cfg(not(unix))]
pub use pty::PtyOpenError;
