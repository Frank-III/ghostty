//! Headless terminal session: config + PTY termio + Rust-owned VT.
//!
//! Port target: subset of `Surface.zig` termio ownership without renderer/input.

use std::time::{Duration, Instant};

use ghostty_foundation::{FoundationError, FoundationResult};
use ghostty_termio::{
    ClipboardKind, CommandBuildError, CommandSpec, RustOwnedTerminalSink, SpawnPtyError,
    SurfaceMessage, TermioLoop, TermioMessage, TermioSink, Winsize,
};

use crate::app_config::AppConfig;
use crate::events::SurfaceEvent;
use crate::runtime::RuntimeClipboard;
use crate::session_command::command_from_config;
use crate::surface_id::SurfaceId;

#[cfg(feature = "rust-vt")]
use ghostty_config::DerivedFontConfig;
#[cfg(feature = "rust-vt")]
use ghostty_font::metrics::{calc, FaceMetrics};
#[cfg(feature = "rust-vt")]
use ghostty_font::{
    descriptor_from_font_family, select_primary, Atlas, AtlasFormat, DesiredSize, DiscoveryError,
    FontSession, GlyphCache, RenderOptions,
};
use ghostty_renderer::build_cell_backgrounds;
#[cfg(feature = "rust-vt")]
use ghostty_renderer::build_cell_texts;
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
    /// Dropped after [`termio`](Self::termio) so the background thread stops before VT teardown.
    terminal: RustOwnedTerminalSink,
    termio: TermioLoop,
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
    #[cfg(feature = "rust-vt")]
    font_atlas: Option<Atlas>,
    #[cfg(feature = "rust-vt")]
    font_session: Option<FontSession>,
    #[cfg(feature = "rust-vt")]
    glyph_cache: GlyphCache,
    #[cfg(feature = "rust-vt")]
    render_opts: Option<RenderOptions>,
    #[cfg(feature = "rust-vt")]
    atlas_upload_generation: usize,
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
        let font_session = open_font_session(config.config());
        let (cell_width_px, cell_height_px) = font_session
            .as_ref()
            .map(|session| {
                let metrics = session.grid_metrics();
                (metrics.cell_width, metrics.cell_height)
            })
            .unwrap_or_else(|| cell_size_from_config(config.config()));
        let render_opts = font_session.as_ref().map(|session| {
            let font = DerivedFontConfig::from(config.config());
            RenderOptions {
                grid_metrics: session.grid_metrics(),
                cell_width: None,
                thicken: font.font_thicken,
                thicken_strength: font.font_thicken_strength,
            }
        });
        let stream_config = ghostty_config::DerivedStreamConfig::from(config.config());
        let mut termio = TermioLoop::spawn(&spec, winsize, stream_config)?;
        let mut terminal = RustOwnedTerminalSink::new(winsize.cols, winsize.rows, scrollback)
            .ok_or(SurfaceSessionError::Terminal)?;
        terminal.bind_session(&mut termio);
        termio.tick(&mut terminal)?;
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
            host_renderer: None,
            #[cfg(feature = "rust-vt")]
            font_atlas: None,
            #[cfg(feature = "rust-vt")]
            font_session,
            #[cfg(feature = "rust-vt")]
            glyph_cache: GlyphCache::default(),
            #[cfg(feature = "rust-vt")]
            render_opts,
            #[cfg(feature = "rust-vt")]
            atlas_upload_generation: 0,
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

    /// Feed escape sequences directly into the rust-owned VT (test / host path).
    #[cfg(feature = "rust-vt")]
    pub fn write_vt_input(&mut self, bytes: &[u8]) -> FoundationResult<()> {
        self.terminal.bind_session(&mut self.termio);
        self.terminal.write_terminal(bytes);
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
        self.terminal.bind_session(&mut self.termio);
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
        self.ensure_host_renderer();
        let snap = self.cell_snapshot();
        self.warm_glyph_cache_from_snapshot(&snap);
        self.sync_font_atlas_upload();
        let mut prep = if let Some(renderer) = &mut self.host_renderer {
            renderer
                .draw_snapshot(&snap, &mut self.damage)
                .ok()
                .map(|p| p.clone())
                .unwrap_or_else(|| prepare_draw_frame(&snap, &mut self.damage))
        } else {
            prepare_draw_frame(&snap, &mut self.damage)
        };
        prep.text_cells = build_cell_texts(
            &snap,
            &self.glyph_cache,
            self.cell_width_px,
            self.cell_height_px,
        );
        let renderer_cfg = ghostty_config::DerivedRendererConfig::from(self.config.config());
        let default_bg = ghostty_renderer::color::Rgb::new(
            renderer_cfg.background.r,
            renderer_cfg.background.g,
            renderer_cfg.background.b,
        );
        prep.bg_cells =
            build_cell_backgrounds(&snap, default_bg, self.cell_width_px, self.cell_height_px);
        self.last_frame = Some(prep.clone());
        prep
    }

    /// Issue draw passes for the last [`prepare_draw`](Self::prepare_draw) and clear damage.
    #[cfg(feature = "rust-vt")]
    pub fn finish_draw(&mut self) {
        if let (Some(renderer), Some(prep)) = (&mut self.host_renderer, self.last_frame.take()) {
            let _ = renderer.present_frame(&prep, &mut self.damage);
            return;
        }
        finish_draw_frame(&mut self.damage);
    }

    /// Statistics from the most recent GPU draw pass when a host renderer is attached.
    #[cfg(feature = "rust-vt")]
    pub fn last_draw_pass(&self) -> Option<ghostty_renderer::DrawPassStats> {
        self.host_renderer.as_ref().and_then(|r| r.last_draw_pass())
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
                    if let Some(([fr, fg, fb], [br, bg, bb])) = self.terminal.cell_colors_rgb(x, y)
                    {
                        snap.set_foreground(x, y, ghostty_renderer::color::Rgb::new(fr, fg, fb));
                        snap.set_background(x, y, ghostty_renderer::color::Rgb::new(br, bg, bb));
                    }
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

    /// Lazily allocate the font atlas.
    #[cfg(feature = "rust-vt")]
    fn ensure_font_atlas(&mut self) {
        if self.font_atlas.is_some() {
            return;
        }
        self.font_atlas = Some(Atlas::new(512, AtlasFormat::Grayscale));
    }

    /// Rasterize visible codepoints into the atlas cache.
    #[cfg(feature = "rust-vt")]
    fn warm_glyph_cache_from_snapshot(&mut self, snap: &CellSnapshot) {
        if self.font_session.is_none() || self.render_opts.is_none() {
            return;
        }
        self.ensure_font_atlas();
        let codepoints: Vec<u32> = snap.codepoints.iter().filter_map(|cp| *cp).collect();
        let Some(atlas) = self.font_atlas.as_mut() else {
            return;
        };
        let Some(session) = self.font_session.as_ref() else {
            return;
        };
        let Some(opts) = self.render_opts.as_ref() else {
            return;
        };
        let _ = self
            .glyph_cache
            .warm_snapshot(session, atlas, codepoints, opts);
    }

    #[cfg(feature = "rust-vt")]
    fn ensure_host_renderer(&mut self) {
        if self.host_renderer.is_some() {
            return;
        }
        let winsize = self.winsize();
        self.host_renderer = HostRenderer::for_host(renderer_size(
            winsize,
            self.cell_width_px,
            self.cell_height_px,
        ))
        .ok();
    }

    /// Upload the font atlas to the host renderer when modified.
    #[cfg(feature = "rust-vt")]
    fn sync_font_atlas_upload(&mut self) {
        self.ensure_font_atlas();
        self.ensure_host_renderer();
        let Some(atlas) = self.font_atlas.as_ref() else {
            return;
        };
        let generation = atlas.modified_generation();
        if generation == self.atlas_upload_generation {
            return;
        }
        if let Some(renderer) = self.host_renderer.as_mut() {
            if renderer.upload_atlas(atlas).is_ok() {
                self.atlas_upload_generation = generation;
            }
        }
    }

    /// Generation counter of the last atlas upload (test/diagnostics).
    #[cfg(feature = "rust-vt")]
    pub fn atlas_upload_generation(&self) -> usize {
        self.atlas_upload_generation
    }

    #[cfg(feature = "rust-vt")]
    pub fn glyph_cache_len(&self) -> usize {
        self.glyph_cache.len()
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
        for msg in self.termio.drain_surface_mailbox() {
            match msg {
                SurfaceMessage::SetTitle(title) => {
                    events.push(SurfaceEvent::TitleChanged { title });
                }
                SurfaceMessage::RedrawRequested => {
                    events.push(SurfaceEvent::RedrawRequested);
                }
                SurfaceMessage::Close => events.push(SurfaceEvent::Close),
                SurfaceMessage::ChildExited { exit_code } => {
                    events.push(SurfaceEvent::ChildExited { exit_code });
                }
                SurfaceMessage::ReportTitle => events.push(SurfaceEvent::SetTitle),
                SurfaceMessage::RingBell => events.push(SurfaceEvent::RingBell),
                SurfaceMessage::ClipboardRead { clipboard } => {
                    events.push(SurfaceEvent::ClipboardRead {
                        clipboard: runtime_clipboard(clipboard),
                    });
                }
                SurfaceMessage::ClipboardWrite { clipboard, data } => {
                    events.push(SurfaceEvent::ClipboardWrite {
                        clipboard: runtime_clipboard(clipboard),
                        data,
                    });
                }
                SurfaceMessage::ColorChange { kind, color } => {
                    events.push(SurfaceEvent::ColorChanged { kind, color });
                }
            }
        }
        events
    }

    pub fn run_until<F>(&mut self, deadline: Instant, mut done: F) -> FoundationResult<()>
    where
        F: FnMut(&mut Self) -> bool,
    {
        loop {
            self.tick()?;
            if done(self) {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            if self.termio.is_shutdown() {
                break;
            }
            std::thread::sleep(remaining.min(std::time::Duration::from_millis(16)));
        }

        // Short-lived children can exit before the last PTY bytes are drained.
        for _ in 0..64 {
            if self.tick()? == 0 {
                break;
            }
            if done(self) {
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> FoundationResult<()> {
        self.termio.shutdown()?;
        self.tick().map(|_| ())
    }

    /// Stop the PTY thread and drain pending output.
    pub fn shutdown_and_drain(&mut self) -> FoundationResult<()> {
        self.terminal.detach_vt_effects();
        self.termio.shutdown_and_drain(&mut self.terminal)
    }

    /// Queue a termio thread message (write/resize/title/redraw).
    pub fn push_termio(&mut self, msg: TermioMessage) -> FoundationResult<()> {
        self.termio.push(msg)
    }

    pub fn cell_codepoint(&self, x: u16, y: u16) -> Option<u32> {
        self.terminal.cell_codepoint(x, y)
    }

    pub fn cell_fg_rgb(&self, x: u16, y: u16) -> Option<[u8; 3]> {
        self.terminal.cell_fg_rgb(x, y)
    }

    pub fn cell_bg_rgb(&self, x: u16, y: u16) -> Option<[u8; 3]> {
        self.terminal.cell_bg_rgb(x, y)
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

impl Drop for SurfaceSession {
    fn drop(&mut self) {
        self.terminal.detach_vt_effects();
        let _ = self.termio.shutdown_and_drain(&mut self.terminal);
    }
}

fn runtime_clipboard(kind: ClipboardKind) -> RuntimeClipboard {
    match kind {
        ClipboardKind::Standard | ClipboardKind::Primary => RuntimeClipboard::Standard,
        ClipboardKind::Selection => RuntimeClipboard::Selection,
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
fn open_font_session(config: &ghostty_config::Config) -> Option<FontSession> {
    let font = DerivedFontConfig::from(config);
    let desc = descriptor_from_font_family(font.font_family.as_deref(), font.font_size);
    let discovered = select_primary(&desc).ok()?;
    FontSession::open(&discovered, DesiredSize::new(font.font_size)).ok()
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
