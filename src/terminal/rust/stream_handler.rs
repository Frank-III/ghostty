#![allow(unused)]
use crate::ansi::*;
use crate::charsets::*;
use crate::constants::*;
use crate::csi::*;
use crate::device_attributes::*;
use crate::device_status::*;
use crate::early::*;
use crate::kitty_key::*;
use crate::mode_def::*;
use crate::mouse_shape::*;
use crate::stream_types::*;
use crate::vt_parser::*;

pub trait StreamHandler {
    fn on_action(&mut self, action: StreamAction<'_>) {
        match action {
            StreamAction::Print(v) => self.on_print(v.cp, 0, false),
            StreamAction::PrintRepeat(n) => self.on_print_repeat(n),
            StreamAction::Bell => self.on_bell(),
            StreamAction::Backspace => self.on_backspace(),
            StreamAction::HorizontalTab(n) => self.on_horizontal_tab(n),
            StreamAction::HorizontalTabBack(n) => self.on_horizontal_tab_back(n),
            StreamAction::Linefeed => self.on_linefeed(),
            StreamAction::CarriageReturn => self.on_carriage_return(),
            StreamAction::Enquiry => self.on_enquiry(),
            StreamAction::InvokeCharset(v) => self.on_invoke_charset(v),
            StreamAction::CursorUp(v) => self.on_cursor_up(v),
            StreamAction::CursorDown(v) => self.on_cursor_down(v),
            StreamAction::CursorLeft(v) => self.on_cursor_left(v),
            StreamAction::CursorRight(v) => self.on_cursor_right(v),
            StreamAction::CursorCol(v) => self.on_cursor_col(v),
            StreamAction::CursorRow(v) => self.on_cursor_row(v),
            StreamAction::CursorColRelative(v) => self.on_cursor_col_relative(v),
            StreamAction::CursorRowRelative(v) => self.on_cursor_row_relative(v),
            StreamAction::CursorPos(v) => self.on_cursor_pos(v),
            StreamAction::CursorStyle(v) => self.on_cursor_style(v),
            StreamAction::EraseDisplayBelow(v) => self.on_erase_display_below(v),
            StreamAction::EraseDisplayAbove(v) => self.on_erase_display_above(v),
            StreamAction::EraseDisplayComplete(v) => self.on_erase_display_complete(v),
            StreamAction::EraseDisplayScrollback(v) => self.on_erase_display_scrollback(v),
            StreamAction::EraseDisplayScrollComplete(v) => self.on_erase_display_scroll_complete(v),
            StreamAction::EraseLineRight(v) => self.on_erase_line_right(v),
            StreamAction::EraseLineLeft(v) => self.on_erase_line_left(v),
            StreamAction::EraseLineComplete(v) => self.on_erase_line_complete(v),
            StreamAction::EraseLineRightUnlessPendingWrap(v) => {
                self.on_erase_line_right_unless_pending_wrap(v)
            }
            StreamAction::DeleteChars(v) => self.on_delete_chars(v),
            StreamAction::EraseChars(v) => self.on_erase_chars(v),
            StreamAction::InsertLines(v) => self.on_insert_lines(v),
            StreamAction::InsertBlanks(v) => self.on_insert_blanks(v),
            StreamAction::DeleteLines(v) => self.on_delete_lines(v),
            StreamAction::ScrollUp(v) => self.on_scroll_up(v),
            StreamAction::ScrollDown(v) => self.on_scroll_down(v),
            StreamAction::TabClearCurrent => self.on_tab_clear_current(),
            StreamAction::TabClearAll => self.on_tab_clear_all(),
            StreamAction::TabSet => self.on_tab_set(),
            StreamAction::TabReset => self.on_tab_reset(),
            StreamAction::Index => self.on_index(),
            StreamAction::NextLine => self.on_next_line(),
            StreamAction::ReverseIndex => self.on_reverse_index(),
            StreamAction::FullReset => self.on_full_reset(),
            StreamAction::SetMode(v) => self.on_set_mode(v),
            StreamAction::ResetMode(v) => self.on_reset_mode(v),
            StreamAction::SaveMode(v) => self.on_save_mode(v),
            StreamAction::RestoreMode(v) => self.on_restore_mode(v),
            StreamAction::RequestMode(v) => self.on_request_mode(v),
            StreamAction::RequestModeUnknown(v) => self.on_request_mode_unknown(v),
            StreamAction::TopAndBottomMargin(v) => self.on_top_and_bottom_margin(v),
            StreamAction::LeftAndRightMargin(v) => self.on_left_and_right_margin(v),
            StreamAction::LeftAndRightMarginAmbiguous => self.on_left_and_right_margin_ambiguous(),
            StreamAction::SaveCursor => self.on_save_cursor(),
            StreamAction::RestoreCursor => self.on_restore_cursor(),
            StreamAction::ModifyKeyFormat(v) => self.on_modify_key_format(v),
            StreamAction::MouseShiftCapture(v) => self.on_mouse_shift_capture(v),
            StreamAction::ProtectedModeOff => self.on_protected_mode_off(),
            StreamAction::ProtectedModeIso => self.on_protected_mode_iso(),
            StreamAction::ProtectedModeDec => self.on_protected_mode_dec(),
            StreamAction::SizeReport(v) => self.on_size_report(v),
            StreamAction::TitlePush(v) => self.on_title_push(v),
            StreamAction::TitlePop(v) => self.on_title_pop(v),
            StreamAction::Xtversion => self.on_xtversion(),
            StreamAction::DeviceAttributes(v) => self.on_device_attributes(v),
            StreamAction::DeviceStatus(v) => self.on_device_status(v),
            StreamAction::KittyKeyboardQuery => self.on_kitty_keyboard_query(),
            StreamAction::KittyKeyboardPush(v) => self.on_kitty_keyboard_push(v),
            StreamAction::KittyKeyboardPop(v) => self.on_kitty_keyboard_pop(v),
            StreamAction::KittyKeyboardSet(v) => self.on_kitty_keyboard_set(v),
            StreamAction::KittyKeyboardSetOr(v) => self.on_kitty_keyboard_set_or(v),
            StreamAction::KittyKeyboardSetNot(v) => self.on_kitty_keyboard_set_not(v),
            StreamAction::DcsHook(v) => self.on_dcs_hook(v),
            StreamAction::DcsPut(v) => self.on_dcs_put(v),
            StreamAction::DcsUnhook => self.on_dcs_unhook(),
            StreamAction::ApcStart => self.on_apc_start(),
            StreamAction::ApcEnd => self.on_apc_end(),
            StreamAction::ApcPut(v) => self.on_apc_put(v),
            StreamAction::EndHyperlink => self.on_end_hyperlink(),
            StreamAction::ActiveStatusDisplay(v) => self.on_active_status_display(v),
            StreamAction::Decaln => self.on_decaln(),
            StreamAction::WindowTitle(v) => self.on_window_title(v),
            StreamAction::ReportPwd(v) => self.on_report_pwd(v),
            StreamAction::ShowDesktopNotification(v) => self.on_show_desktop_notification(v),
            StreamAction::ProgressReport(v) => self.on_progress_report(v),
            StreamAction::StartHyperlink(v) => self.on_start_hyperlink(v),
            StreamAction::ClipboardContents(v) => self.on_clipboard_contents(v),
            StreamAction::MouseShape(v) => self.on_mouse_shape(v),
            StreamAction::ConfigureCharset(v) => self.on_configure_charset(v),
            StreamAction::SetAttribute(v) => self.on_set_attribute(v),
            StreamAction::KittyColorReport(v) => self.on_kitty_color_report(v),
            StreamAction::ColorOperation(v) => self.on_color_operation(v),
            StreamAction::SemanticPrompt(v) => self.on_semantic_prompt(v),
        }
    }

    fn on_print(&mut self, _cp: u32, _width: u8, _is_wide: bool) {}
    fn on_print_repeat(&mut self, _count: usize) {}
    fn on_bell(&mut self) {}
    fn on_backspace(&mut self) {}
    fn on_horizontal_tab(&mut self, _count: u16) {}
    fn on_horizontal_tab_back(&mut self, _count: u16) {}
    fn on_linefeed(&mut self) {}
    fn on_carriage_return(&mut self) {}
    fn on_enquiry(&mut self) {}
    fn on_invoke_charset(&mut self, _v: InvokeCharset) {}
    fn on_cursor_up(&mut self, _v: CursorMovement) {}
    fn on_cursor_down(&mut self, _v: CursorMovement) {}
    fn on_cursor_left(&mut self, _v: CursorMovement) {}
    fn on_cursor_right(&mut self, _v: CursorMovement) {}
    fn on_cursor_col(&mut self, _v: CursorMovement) {}
    fn on_cursor_row(&mut self, _v: CursorMovement) {}
    fn on_cursor_col_relative(&mut self, _v: CursorMovement) {}
    fn on_cursor_row_relative(&mut self, _v: CursorMovement) {}
    fn on_cursor_pos(&mut self, _v: CursorPos) {}
    fn on_cursor_style(&mut self, _v: CursorStyle) {}
    fn on_erase_display_below(&mut self, _v: bool) {}
    fn on_erase_display_above(&mut self, _v: bool) {}
    fn on_erase_display_complete(&mut self, _v: bool) {}
    fn on_erase_display_scrollback(&mut self, _v: bool) {}
    fn on_erase_display_scroll_complete(&mut self, _v: bool) {}
    fn on_erase_line_right(&mut self, _v: bool) {}
    fn on_erase_line_left(&mut self, _v: bool) {}
    fn on_erase_line_complete(&mut self, _v: bool) {}
    fn on_erase_line_right_unless_pending_wrap(&mut self, _v: bool) {}
    fn on_delete_chars(&mut self, _v: usize) {}
    fn on_erase_chars(&mut self, _v: usize) {}
    fn on_insert_lines(&mut self, _v: usize) {}
    fn on_insert_blanks(&mut self, _v: usize) {}
    fn on_delete_lines(&mut self, _v: usize) {}
    fn on_scroll_up(&mut self, _v: usize) {}
    fn on_scroll_down(&mut self, _v: usize) {}
    fn on_tab_clear_current(&mut self) {}
    fn on_tab_clear_all(&mut self) {}
    fn on_tab_set(&mut self) {}
    fn on_tab_reset(&mut self) {}
    fn on_index(&mut self) {}
    fn on_next_line(&mut self) {}
    fn on_reverse_index(&mut self) {}
    fn on_full_reset(&mut self) {}
    fn on_set_mode(&mut self, _v: Mode) {}
    fn on_reset_mode(&mut self, _v: Mode) {}
    fn on_save_mode(&mut self, _v: Mode) {}
    fn on_restore_mode(&mut self, _v: Mode) {}
    fn on_request_mode(&mut self, _v: Mode) {}
    fn on_request_mode_unknown(&mut self, _v: RawMode) {}
    fn on_top_and_bottom_margin(&mut self, _v: Margin) {}
    fn on_left_and_right_margin(&mut self, _v: Margin) {}
    fn on_left_and_right_margin_ambiguous(&mut self) {}
    fn on_save_cursor(&mut self) {}
    fn on_restore_cursor(&mut self) {}
    fn on_modify_key_format(&mut self, _v: ModifyKeyFormat) {}
    fn on_mouse_shift_capture(&mut self, _v: bool) {}
    fn on_protected_mode_off(&mut self) {}
    fn on_protected_mode_iso(&mut self) {}
    fn on_protected_mode_dec(&mut self) {}
    fn on_size_report(&mut self, _v: SizeReportStyle) {}
    fn on_title_push(&mut self, _v: u16) {}
    fn on_title_pop(&mut self, _v: u16) {}
    fn on_xtversion(&mut self) {}
    fn on_device_attributes(&mut self, _v: DeviceAttributeReq) {}
    fn on_device_status(&mut self, _v: DeviceStatus) {}
    fn on_kitty_keyboard_query(&mut self) {}
    fn on_kitty_keyboard_push(&mut self, _v: KittyKeyboardFlags) {}
    fn on_kitty_keyboard_pop(&mut self, _v: u16) {}
    fn on_kitty_keyboard_set(&mut self, _v: KittyKeyboardFlags) {}
    fn on_kitty_keyboard_set_or(&mut self, _v: KittyKeyboardFlags) {}
    fn on_kitty_keyboard_set_not(&mut self, _v: KittyKeyboardFlags) {}
    fn on_dcs_hook(&mut self, _action: ParserDcs) {}
    fn on_dcs_put(&mut self, _code: u8) {}
    fn on_dcs_unhook(&mut self) {}
    fn on_apc_start(&mut self) {}
    fn on_apc_end(&mut self) {}
    fn on_apc_put(&mut self, _code: u8) {}
    fn on_end_hyperlink(&mut self) {}
    fn on_active_status_display(&mut self, _v: StatusDisplay) {}
    fn on_decaln(&mut self) {}
    fn on_window_title(&mut self, _v: WindowTitle<'_>) {}
    fn on_report_pwd(&mut self, _v: ReportPwd<'_>) {}
    fn on_show_desktop_notification(&mut self, _v: ShowDesktopNotification<'_>) {}
    fn on_progress_report(&mut self, _v: ProgressReport) {}
    fn on_start_hyperlink(&mut self, _v: StartHyperlink<'_>) {}
    fn on_clipboard_contents(&mut self, _v: ClipboardContents<'_>) {}
    fn on_mouse_shape(&mut self, _v: MouseShape) {}
    fn on_configure_charset(&mut self, _v: ConfigureCharset) {}
    fn on_set_attribute(&mut self, _v: SgrAttribute) {}
    fn on_kitty_color_report(&mut self, _v: KittyColorReport<'_>) {}
    fn on_color_operation(&mut self, _v: ColorOperation<'_>) {}
    fn on_semantic_prompt(&mut self, _v: SemanticPrompt<'_>) {}
    fn on_raw_action(&mut self, _tag: ParserActionTag) -> bool {
        false
    }
}
