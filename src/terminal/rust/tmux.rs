//! Types and functions related to tmux protocols.
//!
//! Port of `src/terminal/tmux/`. This module re-exports types from
//! tmux sub-modules (control, layout, output, viewer).

#[path = "tmux_control.rs"]
pub mod control;
#[path = "tmux_layout.rs"]
pub mod layout;
#[path = "tmux_output.rs"]
pub mod output;
#[path = "tmux_viewer.rs"]
pub mod viewer;

pub use control::{ControlParser, Notification, NotificationTag, ParserState};
pub use layout::{Layout, LayoutContent, Checksum};
pub use output::{Variable, ParsedPaneState, ParsedWindowInfo};
pub use viewer::{Viewer, ViewerState, Action, ActionTag, ViewerWindow, ViewerPane, Command, CommandTag};
