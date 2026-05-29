//! Headless terminal session: config + PTY termio + Rust-owned VT.
//!
//! Port target: subset of `Surface.zig` termio ownership without renderer/input.

use std::time::Instant;

use ghostty_foundation::{FoundationError, FoundationResult};
use ghostty_termio::{
    CommandBuildError, CommandSpec, RustOwnedTerminalSink, SpawnPtyError, TermioHarness,
    TermioMessage, Winsize,
};

use crate::app_config::AppConfig;
use crate::session_command::command_from_config;
use crate::surface_id::SurfaceId;

const DEFAULT_COLS: u16 = 80;
const DEFAULT_ROWS: u16 = 24;

/// Options when spawning a [`SurfaceSession`].
#[derive(Debug, Clone)]
pub struct SurfaceSessionOptions {
    pub id: Option<SurfaceId>,
    pub winsize: Winsize,
    pub command: Option<CommandSpec>,
}

impl Default for SurfaceSessionOptions {
    fn default() -> Self {
        Self {
            id: None,
            winsize: Winsize {
                cols: DEFAULT_COLS,
                rows: DEFAULT_ROWS,
                x_pixels: 0,
                y_pixels: 0,
            },
            command: None,
        }
    }
}

/// Errors while creating or driving a headless session.
#[derive(Debug)]
pub enum SurfaceSessionError {
    Command(CommandBuildError),
    Spawn(SpawnPtyError),
    Terminal,
    Termio(FoundationError),
}

impl From<CommandBuildError> for SurfaceSessionError {
    fn from(err: CommandBuildError) -> Self {
        Self::Command(err)
    }
}

impl From<SpawnPtyError> for SurfaceSessionError {
    fn from(err: SpawnPtyError) -> Self {
        Self::Spawn(err)
    }
}

impl From<FoundationError> for SurfaceSessionError {
    fn from(err: FoundationError) -> Self {
        Self::Termio(err)
    }
}

/// Owns config, PTY harness, and Rust VT for one headless surface.
pub struct SurfaceSession {
    id: SurfaceId,
    config: AppConfig,
    harness: TermioHarness,
    terminal: RustOwnedTerminalSink,
}

impl SurfaceSession {
    pub fn spawn(
        config: AppConfig,
        opts: SurfaceSessionOptions,
    ) -> Result<Self, SurfaceSessionError> {
        let id = opts.id.unwrap_or_else(|| SurfaceId::from_raw(1).expect("non-zero id"));
        let winsize = opts.winsize;
        let spec = opts
            .command
            .unwrap_or_else(|| command_from_config(config.config()).expect("command spec"));
        let scrollback = config.config().scrollback_limit;
        let mut harness = TermioHarness::spawn(&spec, winsize)?;
        let mut terminal = RustOwnedTerminalSink::new(winsize.cols, winsize.rows, scrollback)
            .ok_or(SurfaceSessionError::Terminal)?;
        harness.drain_mailbox(&mut terminal)?;
        Ok(Self {
            id,
            config,
            harness,
            terminal,
        })
    }

    pub fn from_defaults() -> Result<Self, SurfaceSessionError> {
        Self::spawn(AppConfig::default(), SurfaceSessionOptions::default())
    }

    pub fn id(&self) -> SurfaceId {
        self.id
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn pid(&self) -> libc::pid_t {
        self.harness.pid()
    }

    pub fn winsize(&self) -> Winsize {
        self.harness.winsize()
    }

    /// Queue bytes for the PTY child (keyboard/input path).
    pub fn write(&mut self, bytes: &[u8]) -> FoundationResult<()> {
        self.harness
            .mailbox_mut()
            .push(TermioMessage::Write(bytes.to_vec()))?;
        self.harness.drain_mailbox(&mut self.terminal)
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> FoundationResult<()> {
        self.harness
            .mailbox_mut()
            .push(TermioMessage::Resize { cols, rows })?;
        self.harness.drain_mailbox(&mut self.terminal)
    }

    /// One iteration: drain mailbox, read PTY output into terminal state.
    pub fn tick(&mut self) -> FoundationResult<usize> {
        self.harness.drain_mailbox(&mut self.terminal)?;
        self.harness
            .pump_pty(&mut self.terminal)
            .map_err(|_| FoundationError::Unsupported)
    }

    pub fn run_until<F>(&mut self, deadline: Instant, mut done: F) -> FoundationResult<()>
    where
        F: FnMut(&mut Self) -> bool,
    {
        while !self.harness.is_shutdown() {
            self.harness.drain_mailbox(&mut self.terminal)?;
            let _ = self
                .harness
                .pump_pty(&mut self.terminal)
                .map_err(|_| FoundationError::Unsupported)?;
            if done(self) {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            let timeout_ms = remaining.as_millis().min(i32::MAX as u128) as i32;
            if !self
                .harness
                .poll_readable(timeout_ms)
                .map_err(|_| FoundationError::Unsupported)?
            {
                continue;
            }
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> FoundationResult<()> {
        self.harness
            .mailbox_mut()
            .push(TermioMessage::Shutdown)?;
        self.harness.drain_mailbox(&mut self.terminal)
    }

    pub fn cell_codepoint(&self, x: u16, y: u16) -> Option<u32> {
        self.terminal.cell_codepoint(x, y)
    }

    pub fn contains_text(&self, needle: &str) -> bool {
        self.terminal.contains_text(needle)
    }
}

#[cfg(all(unix, test, feature = "rust-vt"))]
mod tests {
    use std::time::{Duration, Instant};

    use ghostty_config::Config;
    use ghostty_termio::{CommandBuilder, CommandSpec};

    use super::*;
    use crate::AppConfig;

    fn echo_cat_command() -> CommandSpec {
        CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("cat")
            .build()
            .expect("cat spec")
    }

    #[test]
    fn spawn_default_shell() {
        let session = SurfaceSession::from_defaults().expect("spawn");
        assert!(session.pid() > 0);
        assert_eq!(session.id().get(), 1);
    }

    #[test]
    fn write_echo_round_trip() {
        let mut session = SurfaceSession::spawn(
            AppConfig::default(),
            SurfaceSessionOptions {
                command: Some(echo_cat_command()),
                ..Default::default()
            },
        )
        .expect("spawn");

        session.write(b"ping\n").expect("write");
        let deadline = Instant::now() + Duration::from_secs(3);
        session
            .run_until(deadline, |s| s.contains_text("ping"))
            .expect("run");
        assert!(session.contains_text("ping"));
    }

    #[test]
    fn child_output_reaches_terminal() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("printf 'core-vt'")
            .build()
            .expect("spec");

        let mut session = SurfaceSession::spawn(
            AppConfig::default(),
            SurfaceSessionOptions {
                command: Some(spec),
                ..Default::default()
            },
        )
        .expect("spawn");

        let deadline = Instant::now() + Duration::from_secs(3);
        session
            .run_until(deadline, |s| s.contains_text("core-vt"))
            .expect("run");
        assert!(session.contains_text("core-vt"));
    }

    #[test]
    fn config_command_used() {
        let mut cfg = Config::with_defaults();
        cfg.command = Some("printf cfg-cmd".to_string());
        let mut session = SurfaceSession::spawn(
            AppConfig::new(cfg),
            SurfaceSessionOptions::default(),
        )
        .expect("spawn");

        let deadline = Instant::now() + Duration::from_secs(3);
        session
            .run_until(deadline, |s| s.contains_text("cfg-cmd"))
            .expect("run");
        assert!(session.contains_text("cfg-cmd"));
    }

    #[test]
    fn resize_updates_terminal() {
        let mut session = SurfaceSession::spawn(
            AppConfig::default(),
            SurfaceSessionOptions {
                command: Some(echo_cat_command()),
                ..Default::default()
            },
        )
        .expect("spawn");

        session.resize(120, 40).expect("resize");
        assert_eq!(session.winsize().cols, 120);
        assert_eq!(session.winsize().rows, 40);
        session.write(b"x").expect("write");
    }
}
