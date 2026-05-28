//! Pseudo-terminal types (`src/pty.zig`).

pub use crate::mode::PtyMode;
pub use crate::winsize::Winsize;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::{PosixPty, PtyOpenError};

#[cfg(not(unix))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyOpenError {
    UnsupportedPlatform,
}

/// Process metadata query kinds (`pty.zig` `ProcessInfo`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessInfoKind {
    ForegroundPid,
    TtyName,
}
