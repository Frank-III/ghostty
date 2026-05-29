//! VT stream side effects → termio/surface/renderer mailboxes.
//!
//! Port target: `src/termio/stream_handler.zig` (mailbox wiring without full parser).

use ghostty_config::DerivedStreamConfig;
use ghostty_foundation::FoundationResult;

use crate::renderer_mailbox::{RendererMailbox, RendererMessage};
use crate::surface_mailbox::{SurfaceMailbox, SurfaceMessage};
use crate::termio::{TermioMailbox, TermioMessage};

/// Bridges escape-parser side effects to apprt/renderer mailboxes.
#[derive(Debug)]
pub struct StreamHandler {
    config: DerivedStreamConfig,
    termio_mailbox: TermioMailbox,
    surface_mailbox: SurfaceMailbox,
    renderer_mailbox: RendererMailbox,
    _default_cursor: bool,
    termio_messaged: bool,
}

impl StreamHandler {
    pub fn new(config: DerivedStreamConfig) -> Self {
        Self {
            config,
            termio_mailbox: TermioMailbox::new(64),
            surface_mailbox: SurfaceMailbox::new(64),
            renderer_mailbox: RendererMailbox::new(64),
            _default_cursor: true,
            termio_messaged: false,
        }
    }

    pub fn config(&self) -> &DerivedStreamConfig {
        &self.config
    }

    pub fn change_config(&mut self, config: DerivedStreamConfig) {
        self.config = config;
    }

    pub fn termio_messaged(&self) -> bool {
        self.termio_messaged
    }

    pub fn take_termio_messaged(&mut self) -> bool {
        let v = self.termio_messaged;
        self.termio_messaged = false;
        v
    }

    pub fn surface_mailbox(&mut self) -> &mut SurfaceMailbox {
        &mut self.surface_mailbox
    }

    pub fn renderer_mailbox(&mut self) -> &mut RendererMailbox {
        &mut self.renderer_mailbox
    }

    pub fn send_termio(&mut self, msg: TermioMessage) -> FoundationResult<()> {
        self.termio_messaged = true;
        self.termio_mailbox.push(msg)
    }

    pub fn send_surface(&mut self, msg: SurfaceMessage) -> FoundationResult<()> {
        self.surface_mailbox.push(msg)
    }

    pub fn send_renderer(&mut self, msg: RendererMessage) -> FoundationResult<()> {
        self.renderer_mailbox.push(msg)
    }

    pub fn on_set_title(&mut self, title: String) -> FoundationResult<()> {
        self.send_termio(TermioMessage::SetTitle(title.clone()))?;
        self.send_surface(SurfaceMessage::SetTitle(title))
    }

    pub fn on_redraw_requested(&mut self) -> FoundationResult<()> {
        self.send_termio(TermioMessage::RedrawRequested)?;
        self.send_surface(SurfaceMessage::RedrawRequested)?;
        self.send_renderer(RendererMessage::Redraw)
    }

    pub fn clipboard_write_allowed(&self) -> bool {
        !matches!(self.config.clipboard_write, ghostty_config::ClipboardAccess::Deny)
    }

    pub fn osc_color_report_enabled(&self) -> bool {
        !matches!(
            self.config.osc_color_report_format,
            ghostty_config::OscColorReportFormat::None
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghostty_config::Config;

    #[test]
    fn set_title_reaches_surface_mailbox() {
        let mut handler = StreamHandler::new((&Config::with_defaults()).into());
        handler.on_set_title("tab".into()).unwrap();
        assert!(matches!(
            handler.surface_mailbox.pop(),
            Some(SurfaceMessage::SetTitle(_))
        ));
    }

    #[test]
    fn change_config_updates_clipboard_policy() {
        let mut cfg = Config::with_defaults();
        cfg.clipboard_write = ghostty_config::ClipboardAccess::Deny;
        let mut handler = StreamHandler::new((&cfg).into());
        assert!(!handler.clipboard_write_allowed());
        cfg.clipboard_write = ghostty_config::ClipboardAccess::Allow;
        handler.change_config((&cfg).into());
        assert!(handler.clipboard_write_allowed());
    }
}
