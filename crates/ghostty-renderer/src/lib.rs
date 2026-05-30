//! GPU renderer types and pure layout logic for the Ghostty Rust port (Phase 5).
//!
//! Port targets: `src/renderer/` (generic renderer, cell draw types, sizes, cursor).
//!
//! **GTK draw-thread constraint:** On Linux, `src/apprt/gtk/App.zig` sets
//! `must_draw_from_app_thread = true`, so `src/renderer/Thread.zig` posts
//! `redraw_surface` to the GTK main thread instead of calling `draw_frame` on the
//! renderer thread. Any future Rust renderer integration must keep redraws on the
//! UI thread for GTK.
//!
//! Deferred: shader pipelines, `cell.Contents` GPU buffers, font atlas upload, and full
//! `drawFrame` / `rebuildCells` GPU passes.

pub mod atlas_texture;
pub mod backend;
pub mod cell;
pub mod cells;
pub mod color;
pub mod cursor;
pub mod damage;
pub mod draw_backend;
pub mod frame;
pub mod generic;
pub mod host;
pub mod metal;
pub mod opengl;
pub mod size;
pub mod state;
pub mod webgl;

pub use atlas_texture::AtlasTexture;
pub use backend::Backend;
pub use cell::{
    constraint_width, is_covering, no_min_contrast, CellAtlas, CellBg, CellBgDraw, CellKey,
    CellText, CellTextBools, CellView,
};
pub use cells::{rebuild_cells, CellSnapshot};
pub use color::{
    linear_srgb_to_display_p3, shader_rgba, srgb_bytes_to_display_p3_bytes, Rgb, ShaderRgba,
};
pub use cursor::{
    resolve_style, CursorStyle, CursorStyleOptions, RenderCursorState, TerminalCursorStyle,
};
pub use damage::{DamageRect, DamageState};
pub use draw_backend::BackendRenderer;
pub use frame::{finish_draw_frame, prepare_draw_frame, FramePrep};
pub use generic::{
    GenericRenderer, GenericRendererStub, GraphicsApi, GraphicsError, StubGraphicsApi,
};
pub use host::HostRenderer;
pub use metal::{MetalGraphicsApi, MetalRenderer};
pub use opengl::{OpenGlGraphicsApi, OpenGlRenderer};
pub use size::{
    CellCount, CellSize, Coordinate, CoordinateSpace, GridSize, Padding, PaddingBalance,
    ScreenSize, Size,
};
pub use state::{GridPoint, Preedit, PreeditCodepoint, PreeditRange, RendererMouse, RendererState};
pub mod styled;
pub mod text;

pub use styled::{CellStyle, StyledCell, StyledCellSnapshot};
pub use text::{build_cell_backgrounds, build_cell_texts};
