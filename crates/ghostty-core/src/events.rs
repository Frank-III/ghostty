//! App / surface / apprt event tags (subset of `App.Message`, `apprt/surface.Message`, `apprt/action.zig`).
//!
//! Full payloads and C unions (`ghostty_action_s`) are deferred; tags align with
//! `include/ghostty.h` where noted.

/// `apprt.action.Target.Key` / `ghostty_target_tag_e`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionTargetTag {
    App = 0,
    Surface = 1,
}

/// Subset of `apprt.action.Action.Key` / `ghostty_action_tag_e` (bootstrap only).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionTag {
    Quit = 0,
    NewWindow = 1,
    NewTab = 2,
    CloseTab = 3,
    SetTitle = 32,
    PresentTerminal = 21,
    RingBell = 50,
}

/// Messages delivered to the app thread (`App.zig` `Message`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    OpenConfig,
    Quit,
    CloseSurface {
        id: crate::SurfaceId,
    },
    NewWindow,
    /// Surface-scoped event surfaced on the app thread during tick.
    Surface {
        id: crate::SurfaceId,
        event: SurfaceEvent,
    },
}

/// Messages delivered to a single surface (`apprt/surface.zig` `Message`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceEvent {
    SetTitle,
    TitleChanged { title: String },
    Close,
    ChildExited { exit_code: u32 },
    FocusChanged { focused: bool },
    PresentSurface,
    RedrawRequested,
    RingBell,
    ClipboardRead {
        clipboard: crate::RuntimeClipboard,
    },
    ClipboardWrite {
        clipboard: crate::RuntimeClipboard,
        data: Vec<u8>,
    },
    ColorChanged {
        kind: i32,
        color: ghostty_config::RgbColor,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_target_tag_discriminants() {
        assert_eq!(ActionTargetTag::App as i32, 0);
        assert_eq!(ActionTargetTag::Surface as i32, 1);
    }

    #[test]
    fn action_tag_order_matches_bootstrap_subset() {
        assert_eq!(ActionTag::Quit as u32, 0);
        assert_eq!(ActionTag::SetTitle as u32, 32);
        assert_eq!(ActionTag::RingBell as u32, 50);
    }
}
