//! PTY line discipline flags (`src/pty.zig` `Mode`).

/// Canonical mode and echo, shared across platforms.
///
/// Defaults match Zig: canonical and echo enabled (typical interactive shell).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtyMode {
    /// `ICANON` on POSIX.
    pub canonical: bool,
    /// `ECHO` on POSIX.
    pub echo: bool,
}

impl Default for PtyMode {
    fn default() -> Self {
        Self::interactive_shell()
    }
}

impl PtyMode {
    pub const fn interactive_shell() -> Self {
        Self {
            canonical: true,
            echo: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_interactive_shell() {
        assert_eq!(PtyMode::default(), PtyMode::interactive_shell());
    }
}
