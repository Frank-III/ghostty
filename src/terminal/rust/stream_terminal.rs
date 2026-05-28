#![allow(unused)]

use core::ffi::c_void;
use core::ptr;

use crate::ansi::{CursorStyle, ModifyKeyFormat, ProtectedMode, StatusDisplay};
#[cfg(ghostty_vt_terminal_owned)]
use crate::bytes_util::subslice_len;
use crate::charsets::*;
#[cfg(ghostty_vt_terminal_owned)]
use crate::constants::{
    GhosttySizeReportSize, SIZE_REPORT_CSI_14_T, SIZE_REPORT_CSI_16_T, SIZE_REPORT_CSI_18_T,
};
use crate::csi::SizeReportStyle;
use crate::cursor_style::CursorVisualStyle;
use crate::device_attributes::*;
#[cfg(ghostty_vt_terminal_owned)]
use crate::device_status::DeviceStatusRequest;
use crate::device_status::*;
#[cfg(ghostty_vt_terminal_owned)]
use crate::early::GHOSTTY_SUCCESS;
#[cfg(ghostty_vt_terminal_owned)]
use crate::kitty_graphics_command::parse_command_string;
#[cfg(ghostty_vt_terminal_owned)]
use crate::kitty_graphics_exec::{execute as execute_kitty_graphics, ExecContext};
#[cfg(ghostty_vt_terminal_owned)]
use crate::kitty_graphics_storage::ImageStorage;
use crate::kitty_key::*;
#[cfg(ghostty_vt_terminal_owned)]
use crate::mode_def::ModeReportState;
use crate::mode_def::*;
#[cfg(ghostty_vt_terminal_owned)]
use crate::mode_report_encode::mode_report_encode;
use crate::mouse_shape::*;
use crate::screen_set::ScreenKey;
use crate::screen_types::Screen;
use crate::sgr_attribute::{Attribute, Name};
#[cfg(ghostty_vt_terminal_owned)]
use crate::simple_write::{write_bytes, write_decimal};
#[cfg(ghostty_vt_terminal_owned)]
use crate::size_report::size_report_encode_impl;
use crate::size_types::CellCountInt;
use crate::stream_handler::StreamHandler;
use crate::stream_types::*;
use crate::style_types::{Underline, RGB};
#[cfg(ghostty_vt_terminal_owned)]
use crate::terminal_effects;
use crate::terminal_types::{
    MouseShiftCapture, SwitchScreenMode, Terminal, TerminalFlags, TerminalScrollingRegion,
    TABSTOP_INTERVAL,
};
use crate::vt_parser::*;

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

    #[cfg(ghostty_vt_terminal_owned)]
    fn effects_wrapper(&self) -> *mut core::ffi::c_void {
        self.term().effects_wrapper
    }

    #[cfg(ghostty_vt_terminal_owned)]
    fn write_pty(&self, data: &[u8]) {
        unsafe {
            terminal_effects::write_pty(self.effects_wrapper(), data);
        }
    }

    #[cfg(ghostty_vt_terminal_owned)]
    fn send_mode_report(&self, tag: ModeTag, state: ModeReportState) {
        let wrapper = self.effects_wrapper();
        if wrapper.is_null() {
            return;
        }
        let mut buf = [0u8; 64];
        let mut written = 0usize;
        let rc = unsafe {
            mode_report_encode(
                tag.to_u16(),
                state as core::ffi::c_int,
                buf.as_mut_ptr(),
                buf.len(),
                &mut written,
            )
        };
        if rc == GHOSTTY_SUCCESS && written > 0 {
            self.write_pty(subslice_len(&buf, written));
        }
    }

    #[cfg(ghostty_vt_terminal_owned)]
    fn report_cursor_position(&self) {
        let term = self.term();
        let screen = term.active();
        if screen.is_null() {
            return;
        }
        let origin = term.mode_get(ModeTag {
            value: 6,
            ansi: false,
        });
        let (x, y) = unsafe {
            let s = &*screen;
            if origin {
                (
                    s.cursor.x.saturating_sub(term.scrolling_region.left),
                    s.cursor.y.saturating_sub(term.scrolling_region.top),
                )
            } else {
                (s.cursor.x, s.cursor.y)
            }
        };
        let mut buf = [0u8; 32];
        let mut off = 0usize;
        unsafe {
            write_bytes(buf.as_mut_ptr(), &mut off, b"\x1B[");
            write_decimal(buf.as_mut_ptr(), &mut off, (y as u64) + 1);
            write_bytes(buf.as_mut_ptr(), &mut off, b";");
            write_decimal(buf.as_mut_ptr(), &mut off, (x as u64) + 1);
            write_bytes(buf.as_mut_ptr(), &mut off, b"R");
        }
        self.write_pty(subslice_len(&buf, off));
    }

    #[cfg(ghostty_vt_terminal_owned)]
    fn report_kitty_keyboard(&self) {
        let screen = self.term().active();
        if screen.is_null() {
            return;
        }
        let flags = unsafe { (*screen).kitty_keyboard.current().value() };
        let mut buf = [0u8; 16];
        let mut off = 0usize;
        unsafe {
            write_bytes(buf.as_mut_ptr(), &mut off, b"\x1B[?");
            write_decimal(buf.as_mut_ptr(), &mut off, flags as u64);
            write_bytes(buf.as_mut_ptr(), &mut off, b"u");
        }
        self.write_pty(subslice_len(&buf, off));
    }

    #[cfg(ghostty_vt_terminal_owned)]
    fn report_size_csi_21t(&self) {
        let title = unsafe { self.term().get_title_slice() }.unwrap_or(b"");
        let mut buf = [0u8; 1088];
        let mut off = 0usize;
        unsafe {
            write_bytes(buf.as_mut_ptr(), &mut off, b"\x1b]l");
            let title_len = title.len().min(buf.len() - off - 2);
            core::ptr::copy_nonoverlapping(title.as_ptr(), buf.as_mut_ptr().add(off), title_len);
            off += title_len;
            write_bytes(buf.as_mut_ptr(), &mut off, b"\x1b\\");
        }
        self.write_pty(subslice_len(&buf, off));
    }

    fn active(&self) -> *mut Screen {
        self.term().active()
    }

    fn set_mode_by_tag(&mut self, tag: ModeTag, enabled: bool) {
        let term = self.term_mut();
        if !tag.ansi {
            match tag.value {
                47 => {
                    term.switch_screen_mode(SwitchScreenMode::Mode47, enabled);
                    term.mode_set(tag, enabled);
                    return;
                }
                1047 => {
                    term.switch_screen_mode(SwitchScreenMode::Mode1047, enabled);
                    term.mode_set(tag, enabled);
                    return;
                }
                1049 => {
                    term.switch_screen_mode(SwitchScreenMode::Mode1049, enabled);
                    term.mode_set(tag, enabled);
                    return;
                }
                _ => {}
            }
        }
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

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_bell(&mut self) {
        unsafe {
            terminal_effects::bell(self.effects_wrapper());
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
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

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_enquiry(&mut self) {
        unsafe {
            terminal_effects::report_enquiry(self.effects_wrapper());
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
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
        self.term_mut()
            .cursor_set_cell(v.row as usize, v.col as usize);
    }

    fn on_cursor_style(&mut self, v: CursorStyle) {
        let term = self.term_mut();
        let screen = term.active();
        if screen.is_null() {
            return;
        }
        let blink = matches!(
            v,
            CursorStyle::BLINKING_BLOCK
                | CursorStyle::BLINKING_UNDERLINE
                | CursorStyle::BLINKING_BAR
        );
        term.mode_set(
            ModeTag {
                value: 12,
                ansi: false,
            },
            blink,
        );

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
        self.term_mut()
            .erase_display(crate::csi::EraseDisplay::Below, v);
    }

    fn on_erase_display_above(&mut self, v: bool) {
        self.term_mut()
            .erase_display(crate::csi::EraseDisplay::Above, v);
    }

    fn on_erase_display_complete(&mut self, v: bool) {
        self.term_mut()
            .erase_display(crate::csi::EraseDisplay::Complete, v);
    }

    fn on_erase_display_scrollback(&mut self, v: bool) {
        self.term_mut()
            .erase_display(crate::csi::EraseDisplay::Scrollback, v);
    }

    fn on_erase_display_scroll_complete(&mut self, v: bool) {
        self.term_mut()
            .erase_display(crate::csi::EraseDisplay::ScrollComplete, v);
    }

    fn on_erase_line_right(&mut self, v: bool) {
        self.term_mut().erase_line(crate::csi::EraseLine::Right, v);
    }

    fn on_erase_line_left(&mut self, v: bool) {
        self.term_mut().erase_line(crate::csi::EraseLine::Left, v);
    }

    fn on_erase_line_complete(&mut self, v: bool) {
        self.term_mut()
            .erase_line(crate::csi::EraseLine::Complete, v);
    }

    fn on_erase_line_right_unless_pending_wrap(&mut self, v: bool) {
        self.term_mut()
            .erase_line(crate::csi::EraseLine::RightUnlessPendingWrap, v);
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
        self.term_mut().full_reset();
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

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_request_mode(&mut self, v: Mode) {
        let (_, state) = self.term().modes.get_report(v.mode);
        self.send_mode_report(v.mode, state);
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_request_mode(&mut self, _v: Mode) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_request_mode_unknown(&mut self, v: RawMode) {
        let tag = ModeTag {
            value: v.mode,
            ansi: v.ansi,
        };
        let (_, state) = self.term().modes.get_report(tag);
        self.send_mode_report(tag, state);
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
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
            term.mode_get(ModeTag {
                value: 69,
                ansi: false,
            })
        };
        if enabled {
            self.on_left_and_right_margin(Margin {
                top_left: 0,
                bottom_right: 0,
            });
        }
    }

    fn on_save_cursor(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            (*screen).cursor_save();
        }
    }

    fn on_restore_cursor(&mut self) {
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            (*screen).cursor_restore();
        }
    }

    fn on_modify_key_format(&mut self, v: ModifyKeyFormat) {
        let val = matches!(
            v,
            ModifyKeyFormat::OTHER_KEYS_NUMERIC | ModifyKeyFormat::OTHER_KEYS_NUMERIC_EXCEPT
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

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_size_report(&mut self, v: SizeReportStyle) {
        if v == SizeReportStyle::Csi21t {
            self.report_size_csi_21t();
            return;
        }
        let wrapper = self.effects_wrapper();
        if wrapper.is_null() {
            return;
        }
        let mut size = GhosttySizeReportSize {
            rows: 0,
            columns: 0,
            cell_width: 0,
            cell_height: 0,
        };
        if !unsafe { terminal_effects::query_size(wrapper, &mut size) } {
            return;
        }
        let style = match v {
            SizeReportStyle::Csi14t => SIZE_REPORT_CSI_14_T,
            SizeReportStyle::Csi16t => SIZE_REPORT_CSI_16_T,
            SizeReportStyle::Csi18t => SIZE_REPORT_CSI_18_T,
            SizeReportStyle::Csi21t => return,
        };
        let mut buf = [0u8; 256];
        let mut written = 0usize;
        let rc = unsafe {
            size_report_encode_impl(style, size, buf.as_mut_ptr(), buf.len(), &mut written)
        };
        if rc == GHOSTTY_SUCCESS && written > 0 {
            self.write_pty(subslice_len(&buf, written));
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_size_report(&mut self, _v: SizeReportStyle) {}

    fn on_title_push(&mut self, _v: u16) {}
    fn on_title_pop(&mut self, _v: u16) {}
    #[cfg(ghostty_vt_terminal_owned)]
    fn on_xtversion(&mut self) {
        unsafe {
            terminal_effects::report_xtversion(self.effects_wrapper());
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_xtversion(&mut self) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_device_attributes(&mut self, v: DeviceAttributeReq) {
        unsafe {
            terminal_effects::report_device_attributes(self.effects_wrapper(), v as u8);
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_device_attributes(&mut self, _v: DeviceAttributeReq) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_device_status(&mut self, v: DeviceStatus) {
        let Some(req) = DeviceStatusRequest::from_int(v.request, v.question) else {
            return;
        };
        match req {
            DeviceStatusRequest::OperatingStatus => self.write_pty(b"\x1B[0n"),
            DeviceStatusRequest::CursorPosition => self.report_cursor_position(),
            DeviceStatusRequest::ColorScheme => unsafe {
                terminal_effects::report_color_scheme(self.effects_wrapper());
            },
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_device_status(&mut self, _v: DeviceStatus) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_kitty_keyboard_query(&mut self) {
        self.report_kitty_keyboard();
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
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
                (*screen)
                    .kitty_keyboard
                    .set(KittySetMode::Set, KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_kitty_keyboard_set_or(&mut self, v: KittyKeyboardFlags) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen)
                    .kitty_keyboard
                    .set(KittySetMode::Or, KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_kitty_keyboard_set_not(&mut self, v: KittyKeyboardFlags) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen)
                    .kitty_keyboard
                    .set(KittySetMode::Not, KittyKeyFlags::new(v.flags));
            }
        }
    }

    fn on_dcs_hook(&mut self, _action: ParserDcs) {}
    fn on_dcs_put(&mut self, _code: u8) {}
    fn on_dcs_unhook(&mut self) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_apc_start(&mut self) {
        let term = self.term_mut();
        term.apc_state = crate::apc::ApcStateTag::Identify;
        term.apc_len = 0;
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_apc_start(&mut self) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_apc_end(&mut self) {
        let term = self.term_mut();
        if term.apc_state != crate::apc::ApcStateTag::Kitty {
            term.apc_state = crate::apc::ApcStateTag::Inactive;
            term.apc_len = 0;
            return;
        }

        let input_len = term.apc_len;
        term.apc_state = crate::apc::ApcStateTag::Inactive;
        term.apc_len = 0;

        let screen = term.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            let storage = (*screen).kitty_images as *mut ImageStorage;
            if storage.is_null() {
                return;
            }
            let scratch = (*storage).scratch_buf_mut();
            let scratch_cap = (*storage).scratch_cap();
            let Some(cmd) = parse_command_string(scratch, input_len, scratch, scratch_cap) else {
                return;
            };
            let mut ctx = ExecContext {
                storage,
                terminal: self.terminal,
            };
            let _ = execute_kitty_graphics(&mut ctx, &cmd);
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_apc_end(&mut self) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_apc_put(&mut self, code: u8) {
        let term = self.term_mut();
        match term.apc_state {
            crate::apc::ApcStateTag::Inactive => {}
            crate::apc::ApcStateTag::Ignore => {}
            crate::apc::ApcStateTag::Identify => {
                if code == b'G' {
                    if term
                        .apc_max_bytes
                        .get(crate::apc::ApcProtocol::Kitty)
                        .is_some()
                    {
                        term.apc_state = crate::apc::ApcStateTag::Kitty;
                        term.apc_len = 0;
                    } else {
                        term.apc_state = crate::apc::ApcStateTag::Ignore;
                    }
                } else {
                    term.apc_state = crate::apc::ApcStateTag::Ignore;
                }
            }
            crate::apc::ApcStateTag::Kitty => {
                let max = term
                    .apc_max_bytes
                    .get(crate::apc::ApcProtocol::Kitty)
                    .unwrap_or_else(|| {
                        crate::apc::apc_protocol_default_max_bytes(crate::apc::ApcProtocol::Kitty)
                    });
                if term.apc_len >= max {
                    term.apc_state = crate::apc::ApcStateTag::Ignore;
                    return;
                }
                let screen = term.active();
                if screen.is_null() {
                    term.apc_state = crate::apc::ApcStateTag::Ignore;
                    return;
                }
                unsafe {
                    let storage = (*screen).kitty_images as *mut ImageStorage;
                    if storage.is_null() {
                        term.apc_state = crate::apc::ApcStateTag::Ignore;
                        return;
                    }
                    let cap = (*storage).scratch_cap();
                    if term.apc_len >= cap {
                        term.apc_state = crate::apc::ApcStateTag::Ignore;
                        return;
                    }
                    ptr::write((*storage).scratch_buf_mut().add(term.apc_len), code);
                    term.apc_len += 1;
                }
            }
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_apc_put(&mut self, _code: u8) {}

    fn on_end_hyperlink(&mut self) {
        let screen = self.term_mut().active();
        if !screen.is_null() {
            unsafe {
                (*screen).end_hyperlink();
            }
        }
    }

    fn on_active_status_display(&mut self, v: StatusDisplay) {
        self.term_mut().status_display = v;
    }

    fn on_decaln(&mut self) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_window_title(&mut self, v: WindowTitle<'_>) {
        const MAX_TITLE_LEN: usize = 1024;
        let bytes = v.title.as_bytes();
        let len = bytes.len().min(MAX_TITLE_LEN);
        let title_bytes = unsafe { core::slice::from_raw_parts(bytes.as_ptr(), len) };
        let alloc = self.term_mut().bootstrap_alloc;
        if alloc.is_null() {
            return;
        }
        unsafe {
            let _ = self.term_mut().set_title_slice(alloc, title_bytes);
        }
        unsafe {
            terminal_effects::title_changed(self.effects_wrapper());
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_window_title(&mut self, _v: WindowTitle<'_>) {}

    #[cfg(ghostty_vt_terminal_owned)]
    fn on_report_pwd(&mut self, v: ReportPwd<'_>) {
        let alloc = self.term_mut().bootstrap_alloc;
        if alloc.is_null() {
            return;
        }
        unsafe {
            let _ = self.term_mut().set_pwd_slice(alloc, v.url.as_bytes());
        }
    }

    #[cfg(not(ghostty_vt_terminal_owned))]
    fn on_report_pwd(&mut self, _v: ReportPwd<'_>) {}
    fn on_show_desktop_notification(&mut self, _v: ShowDesktopNotification<'_>) {}
    fn on_progress_report(&mut self, _v: ProgressReport) {}
    fn on_clipboard_contents(&mut self, _v: ClipboardContents<'_>) {}

    fn on_start_hyperlink(&mut self, v: StartHyperlink<'_>) {
        let screen = self.term_mut().active();
        if screen.is_null() {
            return;
        }
        unsafe {
            let id = v.id.map(|s| s.as_bytes());
            let _ = (*screen).start_hyperlink(v.uri.as_bytes(), id);
        }
    }

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

    fn on_set_attribute(&mut self, v: SgrAttribute) {
        let attr = sgr_to_attribute(v);
        let screen = self.active();
        if screen.is_null() {
            return;
        }
        unsafe {
            let _ = (*screen).set_attribute(attr);
        }
    }

    fn on_kitty_color_report(&mut self, _v: KittyColorReport) {}

    fn on_color_operation(&mut self, _v: ColorOperation<'_>) {}

    fn on_semantic_prompt(&mut self, _v: SemanticPrompt<'_>) {}

    fn on_raw_action(&mut self, _tag: ParserActionTag) -> bool {
        false
    }
}

fn sgr_to_attribute(v: SgrAttribute) -> Attribute {
    match v.tag {
        SgrAttributeTag::None => Attribute::Unset,
        SgrAttributeTag::Bold => Attribute::Bold,
        SgrAttributeTag::Dim => Attribute::Faint,
        SgrAttributeTag::Italic => Attribute::Italic,
        SgrAttributeTag::Underline => Attribute::Underline(Underline::Single),
        SgrAttributeTag::DoubleUnderline => Attribute::Underline(Underline::Double),
        SgrAttributeTag::Blink | SgrAttributeTag::RapidBlink => Attribute::Blink,
        SgrAttributeTag::Reverse => Attribute::Inverse,
        SgrAttributeTag::Hidden => Attribute::Invisible,
        SgrAttributeTag::Strikethrough => Attribute::Strikethrough,
        SgrAttributeTag::Overline => Attribute::Overline,
        SgrAttributeTag::FgColorReset => Attribute::ResetFg,
        SgrAttributeTag::BgColorReset => Attribute::ResetBg,
        SgrAttributeTag::UnderlineColorReset => Attribute::ResetUnderlineColor,
        SgrAttributeTag::FgColor => fg_color_attribute(v.value),
        SgrAttributeTag::BgColor => bg_color_attribute(v.value),
        SgrAttributeTag::FgColorBright => bright_fg_color_attribute(v.value),
        SgrAttributeTag::BgColorBright => bright_bg_color_attribute(v.value),
        SgrAttributeTag::UnderlineColor => {
            let r = ((v.value >> 16) & 0xff) as u8;
            let g = ((v.value >> 8) & 0xff) as u8;
            let b = (v.value & 0xff) as u8;
            if v.value > 0xff {
                Attribute::UnderlineColor(RGB::new(r, g, b))
            } else {
                Attribute::UnderlineColor256(v.value as u8)
            }
        }
        SgrAttributeTag::Font => Attribute::Unknown(Default::default()),
    }
}

fn fg_color_attribute(value: u32) -> Attribute {
    let p = value as u16;
    if (30..=37).contains(&p) {
        if let Some(name) = Name::from_u8((p - 30) as u8) {
            return Attribute::Color8Fg(name);
        }
    }
    if value > 0xff {
        let r = ((value >> 16) & 0xff) as u8;
        let g = ((value >> 8) & 0xff) as u8;
        let b = (value & 0xff) as u8;
        Attribute::DirectColorFg(RGB::new(r, g, b))
    } else {
        Attribute::Palette256Fg(value as u8)
    }
}

fn bg_color_attribute(value: u32) -> Attribute {
    let p = value as u16;
    if (40..=47).contains(&p) {
        if let Some(name) = Name::from_u8((p - 40) as u8) {
            return Attribute::Color8Bg(name);
        }
    }
    if value > 0xff {
        let r = ((value >> 16) & 0xff) as u8;
        let g = ((value >> 8) & 0xff) as u8;
        let b = (value & 0xff) as u8;
        Attribute::DirectColorBg(RGB::new(r, g, b))
    } else {
        Attribute::Palette256Bg(value as u8)
    }
}

fn bright_fg_color_attribute(value: u32) -> Attribute {
    let p = value as u16;
    if (90..=97).contains(&p) {
        if let Some(name) = Name::from_u8((p - 90) as u8) {
            return Attribute::Color8BrightFg(name);
        }
    }
    Attribute::Unknown(Default::default())
}

fn bright_bg_color_attribute(value: u32) -> Attribute {
    let p = value as u16;
    if (100..=107).contains(&p) {
        if let Some(name) = Name::from_u8((p - 100) as u8) {
            return Attribute::Color8BrightBg(name);
        }
    }
    Attribute::Unknown(Default::default())
}
