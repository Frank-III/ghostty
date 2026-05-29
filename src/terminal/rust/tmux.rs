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
pub use layout::{Checksum, Layout, LayoutContent};
pub use output::{ParsedPaneState, ParsedWindowInfo, Variable};
pub use viewer::{
    Action, ActionTag, Command, CommandTag, Viewer, ViewerPane, ViewerState, ViewerWindow,
};
