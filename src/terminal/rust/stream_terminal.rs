#![allow(unused)]

use core::ffi::c_void;
use core::ptr;

use crate::stream_handler::StreamHandler;
use crate::stream_types::*;
use crate::vt_parser::*;
use crate::mode_def::*;
use crate::charsets::*;
use crate::device_attributes::*;
use crate::device_status::*;
use crate::kitty_key::*;
use crate::mouse_shape::*;
use crate::ansi::{CursorStyle, ModifyKeyFormat, ProtectedMode, StatusDisplay};
use crate::csi::SizeReportStyle;
use crate::screen_types::Screen;
use crate::terminal_types::{MouseShiftCapture, Terminal, TerminalFlags, TerminalScrollingRegion, TABSTOP_INTERVAL};
use crate::screen_set::ScreenKey;
use crate::cursor_style::CursorVisualStyle;
use crate::size_types::CellCountInt;

pub struct StreamTerminal {
    pub terminal: *mut c_void,
}

impl StreamTerminal {
    pub fn new(terminal: *mut c_void) -> Self {
        Self { terminal }
    }

    fn term(&self) -> &Terminal {
        unsafe { &*(self.terminal as *const Terminal) }
    }

    fn term_mut(&mut self) -> &mut Terminal {
        unsafe { &mut *(self.terminal as *mut Terminal) }
    }

    fn active(&self) -> *mut Screen {
        self.term().active()
    }

    fn set_mode_by_tag(&mut self, tag: ModeTag, enabled: bool) {
        let term = self.term_mut();
        term.mode_set(tag, enabled);
        if tag.value == 6 && !tag.ansi && enabled {
            term.cursor_set_cell(1, 1);
        }
    }

    fn horizontal_tab(&mut self, count: u16) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        for _ in 0..count {
            let right = self.term_mut().scrolling_region.right;
            let x = unsafe { (*screen).cursor.x };
            if x >= right {
                break;
            }
            let mut new_x = x + 1;
            unsafe {
                (*screen).cursor.x = new_x;
                (*screen).cursor.pending_wrap = false;
            }
            if self.term_mut().tabstops.get(new_x as usize) {
                continue;
            }
            while new_x < right {
                new_x += 1;
                unsafe {
                    (*screen).cursor.x = new_x;
                }
                if self.term_mut().tabstops.get(new_x as usize) {
                    break;
                }
            }
        }
    }

    fn horizontal_tab_back(&mut self, count: u16) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let left_limit = 0u16;
        for _ in 0..count {
            let x = unsafe { (*screen).cursor.x };
            if x <= left_limit {
                break;
            }
            let mut new_x = x - 1;
            unsafe {
                (*screen).cursor.x = new_x;
                (*screen).cursor.pending_wrap = false;
            }
            if self.term_mut().tabstops.get(new_x as usize) {
                continue;
            }
            while new_x > left_limit {
                new_x -= 1;
                unsafe {
                    (*screen).cursor.x = new_x;
                }
                if self.term_mut().tabstops.get(new_x as usize) {
                    break;
                }
            }
        }
    }
}

impl StreamHandler for StreamTerminal {
    fn on_print(&mut self, cp: u32, _width: u8, _is_wide: bool) {
        self.term_mut().print(cp);
    }

    fn on_print_repeat(&mut self, count: usize) {
        self.term_mut().print_repeat(count);
    }

    fn on_bell(&mut self) {}

    fn on_backspace(&mut self) {
        self.term_mut().backspace();
    }

    fn on_horizontal_tab(&mut self, count: u16) {
        self.horizontal_tab(count);
    }

    fn on_horizontal_tab_back(&mut self, count: u16) {
        self.horizontal_tab_back(count);
    }

    fn on_linefeed(&mut self) {
        self.term_mut().linefeed();
    }

    fn on_carriage_return(&mut self) {
        self.term_mut().carriage_return();
    }

    fn on_enquiry(&mut self) {}

    fn on_invoke_charset(&mut self, v: InvokeCharset) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let s = unsafe { &mut *screen };
        if v.locking {
            match v.bank {
                ActiveSlot::GL => s.charset.gl = v.charset,
                ActiveSlot::GR => s.charset.gr = v.charset,
            }
        } else {
            s.charset.single_shift = Some(v.charset);
        }
    }

    fn on_cursor_up(&mut self, v: CursorMovement) {
        self.term_mut().cursor_up(v.value as usize);
    }

    fn on_cursor_down(&mut self, v: CursorMovement) {
        self.term_mut().cursor_down(v.value as usize);
    }

    fn on_cursor_left(&mut self, v: CursorMovement) {
        self.term_mut().cursor_left(v.value as usize);
    }

    fn on_cursor_right(&mut self, v: CursorMovement) {
        self.term_mut().cursor_right(v.value as usize);
    }

    fn on_cursor_col(&mut self, v: CursorMovement) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let y = unsafe { (*screen).cursor.y } as usize + 1;
        self.term_mut().cursor_set_cell(y, v.value as usize);
    }

    fn on_cursor_row(&mut self, v: CursorMovement) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let x = unsafe { (*screen).cursor.x } as usize + 1;
        self.term_mut().cursor_set_cell(v.value as usize, x);
    }

    fn on_cursor_col_relative(&mut self, v: CursorMovement) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (cx, cy) = unsafe {
            let s = &*screen;
            (s.cursor.x as i32, s.cursor.y as usize)
        };
        let new_x = cx + v.value as i32;
        let new_x = if new_x < 0 { 0 } else { new_x } as usize;
        self.term_mut().cursor_set_cell(cy + 1, new_x + 1);
    }

    fn on_cursor_row_relative(&mut self, v: CursorMovement) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let (cx, cy) = unsafe {
            let s = &*screen;
            (s.cursor.x as usize, s.cursor.y as i32)
        };
        let new_y = cy + v.value as i32;
        let new_y = if new_y < 0 { 0 } else { new_y } as usize;
        self.term_mut().cursor_set_cell(new_y + 1, cx + 1);
    }

    fn on_cursor_pos(&mut self, v: CursorPos) {
        self.term_mut().cursor_set_cell(v.row as usize, v.col as usize);
    }

    fn on_cursor_style(&mut self, v: CursorStyle) {
        let term = self.term_mut();
        let screen = term.active();
        if screen.is_null() {
            return;
        }
        let blink = matches!(
            v,
            CursorStyle::BLINKING_BLOCK | CursorStyle::BLINKING_UNDERLINE | CursorStyle::BLINKING_BAR
        );
        term.mode_set(ModeTag { value: 12, ansi: false }, blink);

        let style = match v {
            CursorStyle::DEFAULT | CursorStyle::BLINKING_BLOCK | CursorStyle::STEADY_BLOCK => {
                CursorVisualStyle::Block
            }
            CursorStyle::BLINKING_UNDERLINE | CursorStyle::STEADY_UNDERLINE => {
                CursorVisualStyle::Underline
            }
            CursorStyle::BLINKING_BAR | CursorStyle::STEADY_BAR => CursorVisualStyle::Bar,
        };

        unsafe {
            (*screen).cursor.cursor_style = style;
        }
    }

    fn on_erase_display_below(&mut self, v: bool) {
        self.term_mut().erase_display(crate::csi::EraseDisplay::Below, v);
    }

    fn on_erase_display_above(&mut self, v: bool) {
        self.term_mut().erase_display(crate::csi::EraseDisplay::Above, v);
    }

    fn on_erase_display_complete(&mut self, v: bool) {
        self.term_mut().erase_display(crate::csi::EraseDisplay::Complete, v);
    }

    fn on_erase_display_scrollback(&mut self, v: bool) {
        self.term_mut().erase_display(crate::csi::EraseDisplay::Scrollback, v);
    }

    fn on_erase_display_scroll_complete(&mut self, v: bool) {
        self.term_mut().erase_display(crate::csi::EraseDisplay::ScrollComplete, v);
    }

    fn on_erase_line_right(&mut self, v: bool) {
        self.term_mut().erase_line(crate::csi::EraseLine::Right, v);
    }

    fn on_erase_line_left(&mut self, v: bool) {
        self.term_mut().erase_line(crate::csi::EraseLine::Left, v);
    }

    fn on_erase_line_complete(&mut self, v: bool) {
        self.term_mut().erase_line(crate::csi::EraseLine::Complete, v);
    }

    fn on_erase_line_right_unless_pending_wrap(&mut self, v: bool) {
        self.term_mut().erase_line(crate::csi::EraseLine::RightUnlessPendingWrap, v);
    }

    fn on_delete_chars(&mut self, _v: usize) {}

    fn on_erase_chars(&mut self, v: usize) {
        self.term_mut().erase_chars(v);
    }

    fn on_insert_lines(&mut self, v: usize) {
        self.term_mut().insert_lines(v);
    }

    fn on_insert_blanks(&mut self, _v: usize) {}

    fn on_delete_lines(&mut self, v: usize) {
        self.term_mut().delete_lines(v);
    }

    fn on_scroll_up(&mut self, v: usize) {
        self.term_mut().scroll_up(v);
    }

    fn on_scroll_down(&mut self, v: usize) {
        self.term_mut().scroll_down(v);
    }

    fn on_tab_clear_current(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let x = unsafe { (*screen).cursor.x };
        self.term_mut().tabstops.unset(x as usize);
    }

    fn on_tab_clear_all(&mut self) {
        self.term_mut().tabstops.clear();
    }

    fn on_tab_set(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        let x = unsafe { (*screen).cursor.x };
        self.term_mut().tabstops.set(x as usize);
    }

    fn on_tab_reset(&mut self) {
        self.term_mut().tabstops.reset(TABSTOP_INTERVAL as usize);
    }

    fn on_index(&mut self) {
        let term = self.term_mut();
        let screen = term.active();
        if screen.is_null() {
            return;
        }
        let bottom = term.scrolling_region.bottom;
        let cy = unsafe { (*screen).cursor.y };
        if cy >= bottom {
            term.scroll_up(1);
        } else {
            unsafe {
                (*screen).cursor.y = cy + 1;
            }
        }
    }

    fn on_next_line(&mut self) {
        self.on_index();
        self.on_carriage_return();
    }

    fn on_reverse_index(&mut self) {
        let term = self.term_mut();
        let screen = term.active();
        if screen.is_null() {
            return;
        }
        let top = term.scrolling_region.top;
        let cy = unsafe { (*screen).cursor.y };
        if cy <= top {
            term.scroll_down(1);
        } else {
            unsafe {
                (*screen).cursor.y = cy - 1;
            }
        }
    }

    fn on_full_reset(&mut self) {
        let term = self.term_mut();
        term.screens.active_key = ScreenKey::Primary;
        term.modes.reset();
        term.flags = TerminalFlags::default();
        term.previous_char = 0;
        term.has_previous_char = false;
        term.mouse_shape = MouseShape::default();
        term.status_display = StatusDisplay::MAIN;
        if term.rows > 0 && term.cols > 0 {
            term.scrolling_region.top = 0;
            term.scrolling_region.bottom = term.rows.saturating_sub(1);
            term.scrolling_region.left = 0;
            term.scrolling_region.right = term.cols.saturating_sub(1);
        } else {
            term.scrolling_region = TerminalScrollingRegion::default();
        }
        term.flags.dirty.clear = true;
    }

    fn on_set_mode(&mut self, v: Mode) {
        self.set_mode_by_tag(v.mode, true);
    }

    fn on_reset_mode(&mut self, v: Mode) {
        self.set_mode_by_tag(v.mode, false);
    }

    fn on_save_mode(&mut self, v: Mode) {
        self.term_mut().modes.save_by_tag(v.mode);
    }

    fn on_restore_mode(&mut self, v: Mode) {
        let restored = self.term_mut().modes.restore_by_tag(v.mode);
        self.set_mode_by_tag(v.mode, restored);
    }

    fn on_request_mode(&mut self, _v: Mode) {}

    fn on_request_mode_unknown(&mut self, _v: RawMode) {}

    fn on_top_and_bottom_margin(&mut self, v: Margin) {
        let term = self.term_mut();
        let rows = term.rows;
        let cols = term.cols;
        let top = v.top_left as CellCountInt;
        let bottom = if v.bottom_right == 0 {
            rows.saturating_sub(1)
        } else {
            (v.bottom_right as CellCountInt).saturating_sub(1)
        };
        if top < rows && bottom < rows && top < bottom {
            term.scrolling_region.top = top;
            term.scrolling_region.bottom = bottom;
            term.cursor_set_cell(1, 1);
        }
    }

    fn on_left_and_right_margin(&mut self, v: Margin) {
        let term = self.term_mut();
        let cols = term.cols;
        let left = v.top_left as CellCountInt;
        let right = if v.bottom_right == 0 {
            cols.saturating_sub(1)
        } else {
            (v.bottom_right as CellCountInt).saturating_sub(1)
        };
        if left < cols && right < cols && left < right {
            term.scrolling_region.left = left;
            term.scrolling_region.right = right;
            term.cursor_set_cell(1, 1);
        }
    }

    fn on_left_and_right_margin_ambiguous(&mut self) {
        let enabled = {
            let term = self.term_mut();
            term.mode_get(ModeTag { value: 69, ansi: false })
        };
        if enabled {
            self.on_left_and_right_margin(Margin {
                top_left: 0,
                bottom_right: 0,
            });
        }
    }

    fn on_save_cursor(&mut self) {}

    fn on_restore_cursor(&mut self) {}

    fn on_modify_key_format(&mut self, v: ModifyKeyFormat) {
        let val = matches!(
            v,
            ModifyKeyFormat::OTHER_KEYS_NUMERIC
                | ModifyKeyFormat::OTHER_KEYS_NUMERIC_EXCEPT
        );
        self.term_mut().flags.modify_other_keys_2 = val;
    }

    fn on_mouse_shift_capture(&mut self, v: bool) {
        self.term_mut().flags.mouse_shift_capture = if v {
            MouseShiftCapture::True
        } else {
            MouseShiftCapture::False
        };
    }

    fn on_protected_mode_off(&mut self) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).protected_mode = ProtectedMode::OFF;
            }
        }
    }

    fn on_protected_mode_iso(&mut self) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).protected_mode = ProtectedMode::ISO;
            }
        }
    }

    fn on_protected_mode_dec(&mut self) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).protected_mode = ProtectedMode::DEC;
            }
        }
    }

    fn on_size_report(&mut self, _v: SizeReportStyle) {}

    fn on_title_push(&mut self, _v: u16) {}
    fn on_title_pop(&mut self, _v: u16) {}
    fn on_xtversion(&mut self) {}
    fn on_device_attributes(&mut self, _v: DeviceAttributeReq) {}
    fn on_device_status(&mut self, _v: DeviceStatus) {}

    fn on_kitty_keyboard_query(&mut self) {}

    fn on_kitty_keyboard_push(&mut self, v: KittyKeyboardFlags) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).kitty_keyboard.push(KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_kitty_keyboard_pop(&mut self, v: u16) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).kitty_keyboard.pop(v as usize);
            }
        }
    }

    fn on_kitty_keyboard_set(&mut self, v: KittyKeyboardFlags) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).kitty_keyboard.set(KittySetMode::Set, KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_kitty_keyboard_set_or(&mut self, v: KittyKeyboardFlags) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).kitty_keyboard.set(KittySetMode::Or, KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_kitty_keyboard_set_not(&mut self, v: KittyKeyboardFlags) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).kitty_keyboard.set(KittySetMode::Not, KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_dcs_hook(&mut self, _action: ParserDcs) {}
    fn on_dcs_put(&mut self, _code: u8) {}
    fn on_dcs_unhook(&mut self) {}

    fn on_apc_start(&mut self) {}
    fn on_apc_end(&mut self) {}
    fn on_apc_put(&mut self, _code: u8) {}

    fn on_end_hyperlink(&mut self) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).cursor.hyperlink = ptr::null_mut();
                (*screen).cursor.hyperlink_id = 0;
                (*screen).cursor.hyperlink_implicit_id = 0;
            }
        }
    }

    fn on_active_status_display(&mut self, v: StatusDisplay) {
        self.term_mut().status_display = v;
    }

    fn on_decaln(&mut self) {}

    fn on_window_title(&mut self, _v: WindowTitle<'_>) {}
    fn on_report_pwd(&mut self, _v: ReportPwd<'_>) {}
    fn on_show_desktop_notification(&mut self, _v: ShowDesktopNotification<'_>) {}
    fn on_progress_report(&mut self, _v: ProgressReport) {}
    fn on_clipboard_contents(&mut self, _v: ClipboardContents<'_>) {}

    fn on_start_hyperlink(&mut self, _v: StartHyperlink<'_>) {}

    fn on_mouse_shape(&mut self, v: MouseShape) {
        self.term_mut().mouse_shape = v;
    }

    fn on_configure_charset(&mut self, v: ConfigureCharset) {
        let screen = self.term_mut().active();
        if screen.is_null() {
            return;
        }
        unsafe {
            (*screen).charset.charsets.set(v.slot, v.charset);
        }
    }

    fn on_set_attribute(&mut self, _v: SgrAttribute) {}

    fn on_kitty_color_report(&mut self, _v: KittyColorReport) {}

    fn on_color_operation(&mut self, _v: ColorOperation<'_>) {}

    fn on_semantic_prompt(&mut self, _v: SemanticPrompt<'_>) {}

    fn on_raw_action(&mut self, _tag: ParserActionTag) -> bool {
        false
    }
}
