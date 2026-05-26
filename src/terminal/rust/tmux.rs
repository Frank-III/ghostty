//! Types and functions related to tmux protocols.
//!
//! Port of `src/terminal/tmux.zig`. This module re-exports types from
//! tmux sub-modules (control, layout, output, viewer). Stub structs are
//! provided until those sub-modules are ported.

pub struct ControlParser;
pub struct ControlNotification;
pub struct Layout;
pub struct Viewer;
pub mod output {}
