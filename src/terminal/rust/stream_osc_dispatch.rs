#![allow(unused)]

use crate::constants::*;
use crate::early::*;
use crate::mouse_shape::*;
use crate::osc_types::*;
use crate::stream_handler::*;
use crate::stream_types::*;

#[inline]
pub fn osc_dispatch<H: StreamHandler>(handler: &mut H, cmd: Command<'_>) {
    match cmd {
        Command::SemanticPrompt(sp) => {
            handler.on_semantic_prompt(sp);
        }

        Command::ChangeWindowTitle(title) => {
            if !is_valid_utf8(title.as_bytes()) {
                return;
            }
            handler.on_window_title(WindowTitle { title });
        }

        Command::ChangeWindowIcon(_icon) => {}

        Command::ClipboardContents(clip) => {
            handler.on_clipboard_contents(ClipboardContents {
                kind: clip.kind,
                data: clip.data.as_bytes(),
            });
        }

        Command::ReportPwd(v) => {
            handler.on_report_pwd(ReportPwd { url: v.value });
        }

        Command::MouseShape(v) => {
            if let Some(shape) = MouseShape::from_string(v.value) {
                handler.on_mouse_shape(shape);
            }
        }

        Command::ColorOperation(v) => {
            handler.on_color_operation(ColorOperation {
                op: v.op,
                requests: v.requests,
                terminator: v.terminator,
            });
        }

        Command::KittyColorProtocol(v) => {
            handler.on_kitty_color_report(KittyColorReport {
                requests: v.requests,
                terminator: v.terminator,
            });
        }

        Command::ShowDesktopNotification(v) => {
            handler.on_show_desktop_notification(ShowDesktopNotification {
                title: v.title,
                body: v.body,
            });
        }

        Command::HyperlinkStart(v) => {
            handler.on_start_hyperlink(StartHyperlink {
                uri: v.uri,
                id: v.id,
            });
        }

        Command::HyperlinkEnd => {
            handler.on_end_hyperlink();
        }

        Command::ConemuProgressReport(v) => {
            handler.on_progress_report(ProgressReport {
                progress: v.progress,
            });
        }

        Command::ConemuSleep { .. }
        | Command::ConemuShowMessageBox(_)
        | Command::ConemuChangeTabTitle(_)
        | Command::ConemuWaitInput
        | Command::ConemuGuimacro(_)
        | Command::ConemuRunProcess(_)
        | Command::ConemuOutputEnvironmentVariable(_)
        | Command::ConemuXtermEmulation(_)
        | Command::ConemuComment(_)
        | Command::KittyTextSizing
        | Command::KittyClipboardProtocol
        | Command::ContextSignal => {}

        Command::Invalid => {}
    }
}

use crate::bytes_util::is_valid_utf8;
