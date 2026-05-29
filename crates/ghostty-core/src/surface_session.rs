//! Headless terminal session: config + PTY termio + Rust-owned VT.
//!
//! Port target: subset of `Surface.zig` termio ownership without renderer/input.

use std::time::Instant;

use ghostty_foundation::{FoundationError, FoundationResult};
use ghostty_termio::{
    CommandBuildError, CommandSpec, RustOwnedTerminalSink, SpawnPtyError, TermioLoop,
    TermioMessage, Winsize,
};

use crate::app_config::AppConfig;
use crate::events::SurfaceEvent;
use crate::session_command::command_from_config;
use crate::surface_id::SurfaceId;

#[cfg(feature = "rust-vt")]
use ghostty_config::DerivedFontConfig;
#[cfg(feature = "rust-vt")]
use ghostty_font::metrics::{calc, FaceMetrics};
#[cfg(feature = "rust-vt")]
use ghostty_font::{descriptor_from_font_family, select_primary, DiscoveryError};
#[cfg(feature = "rust-vt")]
use ghostty_renderer::cells::CellSnapshot;
#[cfg(feature = "rust-vt")]
use ghostty_renderer::damage::{DamageRect, DamageState};
#[cfg(feature = "rust-vt")]
use ghostty_renderer::frame::{finish_draw_frame, prepare_draw_frame, FramePrep};
#[cfg(feature = "rust-vt")]
use ghostty_renderer::size::GridSize;
#[cfg(feature = "rust-vt")]
use ghostty_renderer::{CellSize, HostRenderer, Padding, ScreenSize, Size};

const DEFAULT_COLS: u16 = 80;
const DEFAULT_ROWS: u16 = 24;

/// Options when spawning a [`SurfaceSession`].
#[derive(Debug, Clone)]
pub struct SurfaceSessionOptions {
    pub id: Option<SurfaceId>,
    pub winsize: Winsize,
    pub command: Option<CommandSpec>,
    pub resources_dir: Option<std::path::PathBuf>,
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
            resources_dir: None,
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

/// Owns config, production termio loop, and Rust VT for one headless surface.
pub struct SurfaceSession {
    id: SurfaceId,
    config: AppConfig,
    termio: TermioLoop,
    terminal: RustOwnedTerminalSink,
    #[cfg(feature = "rust-vt")]
    cell_width_px: u32,
    #[cfg(feature = "rust-vt")]
    cell_height_px: u32,
    #[cfg(feature = "rust-vt")]
    damage: DamageState,
    #[cfg(feature = "rust-vt")]
    last_frame: Option<FramePrep>,
    #[cfg(feature = "rust-vt")]
    host_renderer: Option<HostRenderer>,
    pending_title: Option<String>,
    pending_redraw: bool,
}

impl SurfaceSession {
    pub fn spawn(
        config: AppConfig,
        opts: SurfaceSessionOptions,
    ) -> Result<Self, SurfaceSessionError> {
        let id = opts
            .id
            .unwrap_or_else(|| SurfaceId::from_raw(1).expect("non-zero id"));
        let winsize = opts.winsize;
        let spec = opts.command.unwrap_or_else(|| {
            command_from_config(config.config(), opts.resources_dir.as_deref())
                .expect("command spec")
        });
        let scrollback = config.config().scrollback_limit;
        let (cell_width_px, cell_height_px) = cell_size_from_config(config.config());
        let stream_config = ghostty_config::DerivedStreamConfig::from(config.config());
        let mut termio = TermioLoop::spawn(&spec, winsize, stream_config)?;
        let mut terminal = RustOwnedTerminalSink::new(winsize.cols, winsize.rows, scrollback)
            .ok_or(SurfaceSessionError::Terminal)?;
        termio.tick(&mut terminal)?;
        let host_renderer =
            HostRenderer::for_host(renderer_size(winsize, cell_width_px, cell_height_px)).ok();
        Ok(Self {
            id,
            config,
            termio,
            terminal,
            cell_width_px,
            cell_height_px,
            damage: DamageState::default(),
            #[cfg(feature = "rust-vt")]
            last_frame: None,
            #[cfg(feature = "rust-vt")]
            host_renderer,
            pending_title: None,
            pending_redraw: false,
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
        self.termio.pid()
    }

    pub fn winsize(&self) -> Winsize {
        self.termio.winsize()
    }

    #[cfg(feature = "rust-vt")]
    pub fn cell_size_px(&self) -> (u32, u32) {
        (self.cell_width_px, self.cell_height_px)
    }

    #[cfg(feature = "rust-vt")]
    pub fn damage(&self) -> &DamageState {
        &self.damage
    }

    /// Queue bytes for the PTY child (keyboard/input path).
    pub fn write(&mut self, bytes: &[u8]) -> FoundationResult<()> {
        self.termio.push(TermioMessage::Write(bytes.to_vec()))?;
        self.tick().map(|_| ())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> FoundationResult<()> {
        self.termio.push(TermioMessage::Resize { cols, rows })?;
        self.tick().map(|_| ())?;
        #[cfg(feature = "rust-vt")]
        self.damage.mark_full();
        Ok(())
    }

    /// One iteration: drain thread events into terminal state.
    pub fn tick(&mut self) -> FoundationResult<usize> {
        let drain = self.termio.tick(&mut self.terminal)?;
        if let Some(title) = drain.set_title {
            self.pending_title = Some(title);
        }
        if drain.redraw_requested {
            self.pending_redraw = true;
        }
        #[cfg(feature = "rust-vt")]
        if drain.pty_bytes > 0 {
            let ws = self.winsize();
            let size = GridSize {
                columns: ws.cols,
                rows: ws.rows,
            };
            self.damage.mark_rect(DamageRect::full_screen(size));
        }
        Ok(drain.pty_bytes)
    }

    /// Prepare a draw frame from the current terminal grid (CPU path until GPU lands).
    #[cfg(feature = "rust-vt")]
    pub fn prepare_draw(&mut self) -> FramePrep {
        let snap = self.cell_snapshot();
        if let Some(renderer) = &mut self.host_renderer {
            if let Ok(prep) = renderer.draw_snapshot(&snap, &mut self.damage) {
                let prep = prep.clone();
                self.last_frame = Some(prep.clone());
                return prep;
            }
        }
        let prep = prepare_draw_frame(&snap, &mut self.damage);
        self.last_frame = Some(prep.clone());
        prep
    }

    /// Mark the last prepared frame as presented.
    #[cfg(feature = "rust-vt")]
    pub fn finish_draw(&mut self) {
        finish_draw_frame(&mut self.damage);
        self.last_frame = None;
    }

    /// Last frame prep from the most recent [`prepare_draw`](Self::prepare_draw).
    #[cfg(feature = "rust-vt")]
    pub fn last_frame_prep(&self) -> Option<&FramePrep> {
        self.last_frame.as_ref()
    }

    /// Read the visible VT grid into a renderer cell snapshot.
    #[cfg(feature = "rust-vt")]
    pub fn cell_snapshot(&self) -> CellSnapshot {
        let ws = self.winsize();
        let grid = GridSize {
            columns: ws.cols,
            rows: ws.rows,
        };
        let mut snap = CellSnapshot::empty(grid);
        for y in 0..ws.rows {
            for x in 0..ws.cols {
                if let Some(cp) = self.cell_codepoint(x, y) {
                    snap.set(x, y, cp);
                }
            }
        }
        snap
    }

    /// Snapshot plus damage merge for the renderer draw path.
    #[cfg(feature = "rust-vt")]
    pub fn snapshot_for_render(&self) -> CellSnapshot {
        self.cell_snapshot()
    }

    /// Apply the current grid snapshot to renderer damage state (legacy; prefer [`prepare_draw`](Self::prepare_draw)).
    #[cfg(feature = "rust-vt")]
    pub fn rebuild_render_cells(&mut self) {
        let _ = self.prepare_draw();
    }

    /// Resolve a primary font from config (discovery path or metadata-only fallback).
    #[cfg(feature = "rust-vt")]
    pub fn discover_font(&self) -> Result<ghostty_font::DiscoveredFont, DiscoveryError> {
        let font = DerivedFontConfig::from(self.config.config());
        let desc = descriptor_from_font_family(font.font_family.as_deref(), font.font_size);
        select_primary(&desc)
    }

    /// Take pending surface events produced by the last termio drain(s).
    pub fn drain_session_events(&mut self) -> Vec<SurfaceEvent> {
        let mut events = Vec::new();
        if let Some(title) = self.pending_title.take() {
            events.push(SurfaceEvent::TitleChanged { title });
        }
        if self.pending_redraw {
            self.pending_redraw = false;
            events.push(SurfaceEvent::RedrawRequested);
        }
        events
    }

    pub fn run_until<F>(&mut self, deadline: Instant, mut done: F) -> FoundationResult<()>
    where
        F: FnMut(&mut Self) -> bool,
    {
        while !self.termio.is_shutdown() {
            self.tick()?;
            if done(self) {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            std::thread::sleep(remaining.min(std::time::Duration::from_millis(16)));
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> FoundationResult<()> {
        self.termio.shutdown()?;
        self.tick().map(|_| ())
    }

    /// Queue a termio thread message (write/resize/title/redraw).
    pub fn push_termio(&mut self, msg: TermioMessage) -> FoundationResult<()> {
        self.termio.push(msg)
    }

    pub fn cell_codepoint(&self, x: u16, y: u16) -> Option<u32> {
        self.terminal.cell_codepoint(x, y)
    }

    pub fn poll_child_exit(&mut self) -> Option<u32> {
        self.termio.poll_child_exit()
    }

    pub fn is_shutdown(&self) -> bool {
        self.termio.is_shutdown()
    }

    pub fn contains_text(&self, needle: &str) -> bool {
        self.terminal.contains_text(needle)
    }
}

#[cfg(feature = "rust-vt")]
fn renderer_size(winsize: Winsize, cell_width_px: u32, cell_height_px: u32) -> Size {
    Size {
        screen: ScreenSize {
            width: u32::from(winsize.cols).saturating_mul(cell_width_px),
            height: u32::from(winsize.rows).saturating_mul(cell_height_px),
        },
        cell: CellSize {
            width: cell_width_px,
            height: cell_height_px,
        },
        padding: Padding::default(),
    }
}

#[cfg(feature = "rust-vt")]
fn cell_size_from_config(config: &ghostty_config::Config) -> (u32, u32) {
    let font = DerivedFontConfig::from(config);
    let px = f64::from(font.font_size);
    let face = FaceMetrics {
        px_per_em: px,
        cell_width: px * 0.6,
        ascent: px * 0.8,
        descent: -(px * 0.2),
        line_gap: 0.0,
        ..FaceMetrics::default()
    };
    let metrics = calc(face);
    (metrics.cell_width, metrics.cell_height)
}

impl std::fmt::Debug for SurfaceSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceSession")
            .field("id", &self.id)
            .field("winsize", &self.winsize())
            .finish_non_exhaustive()
    }
}
