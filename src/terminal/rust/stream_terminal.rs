#![allow(unused)]

use core::ffi::c_void;

use crate::stream_handler::StreamHandler;
use crate::stream_types::*;
use crate::vt_parser::*;
use crate::mode_def::*;
use crate::charsets::*;
use crate::device_attributes::*;
use crate::device_status::*;
use crate::kitty_key::*;
use crate::mouse_shape::*;
use crate::ansi::{CursorStyle, ModifyKeyFormat, StatusDisplay};
use crate::csi::SizeReportStyle;

pub struct StreamTerminal {
    pub terminal: *mut c_void,
}

impl StreamTerminal {
    pub fn new(terminal: *mut c_void) -> Self {
        Self { terminal }
    }
}

impl StreamHandler for StreamTerminal {
    fn on_print(&mut self, _cp: u32, _width: u8, _is_wide: bool) {
        // TODO: write codepoint to screen at cursor position,
        // advance cursor, handle wrapping and wide chars
    }

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
    fn on_kitty_color_report(&mut self, _v: KittyColorReport) {}
    fn on_color_operation(&mut self, _v: ColorOperation<'_>) {}
    fn on_semantic_prompt(&mut self, _v: SemanticPrompt<'_>) {}
    fn on_raw_action(&mut self, _tag: ParserActionTag) -> bool { false }
}
