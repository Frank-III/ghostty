//! Cursor style resolution for rendering.
//!
//! Port target: `src/renderer/cursor.zig`.

/// Renderer cursor styles (`renderer.CursorStyle`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    BlockHollow,
    Bar,
    Underline,
    Lock,
}

/// Terminal-facing cursor style before renderer mapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerminalCursorStyle {
    #[default]
    Block,
    Bar,
    BlockHollow,
    Underline,
}

impl CursorStyle {
    pub fn from_terminal(term: TerminalCursorStyle) -> Self {
        match term {
            TerminalCursorStyle::Bar => CursorStyle::Bar,
            TerminalCursorStyle::Block => CursorStyle::Block,
            TerminalCursorStyle::BlockHollow => CursorStyle::BlockHollow,
            TerminalCursorStyle::Underline => CursorStyle::Underline,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CursorStyleOptions {
    pub preedit: bool,
    pub focused: bool,
    pub blink_visible: bool,
}

/// Snapshot of terminal cursor fields used by `cursor.style`.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderCursorState {
    pub viewport_visible: bool,
    pub visible: bool,
    pub blinking: bool,
    pub password_input: bool,
    pub visual_style: TerminalCursorStyle,
}

/// Returns the cursor style to draw, or `None` when hidden.
pub fn resolve_style(state: &RenderCursorState, opts: CursorStyleOptions) -> Option<CursorStyle> {
    if !state.viewport_visible {
        return None;
    }
    if opts.preedit {
        return Some(CursorStyle::Block);
    }
    if state.password_input {
        return Some(CursorStyle::Lock);
    }
    if !state.visible {
        return None;
    }
    if !opts.focused {
        return Some(CursorStyle::BlockHollow);
    }
    if state.blinking && !opts.blink_visible {
        return None;
    }
    Some(CursorStyle::from_terminal(state.visual_style))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar_cursor() -> RenderCursorState {
        RenderCursorState {
            viewport_visible: true,
            visible: true,
            blinking: true,
            password_input: false,
            visual_style: TerminalCursorStyle::Bar,
        }
    }

    #[test]
    fn focused_blink_off_hides_bar() {
        let state = bar_cursor();
        assert_eq!(
            resolve_style(
                &state,
                CursorStyleOptions {
                    focused: true,
                    blink_visible: false,
                    ..Default::default()
                }
            ),
            None
        );
    }

    #[test]
    fn unfocused_uses_hollow() {
        let state = bar_cursor();
        assert_eq!(
            resolve_style(
                &state,
                CursorStyleOptions {
                    focused: false,
                    blink_visible: false,
                    ..Default::default()
                }
            ),
            Some(CursorStyle::BlockHollow)
        );
    }

    #[test]
    fn preedit_forces_block() {
        let state = RenderCursorState {
            visible: false,
            ..bar_cursor()
        };
        assert_eq!(
            resolve_style(
                &state,
                CursorStyleOptions {
                    preedit: true,
                    ..Default::default()
                }
            ),
            Some(CursorStyle::Block)
        );
    }
}
