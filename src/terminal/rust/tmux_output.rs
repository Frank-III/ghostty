pub const OUTPUT_PARSE_ERROR: i32 = -1;
pub const OUTPUT_MISSING_ENTRY: i32 = -2;
pub const OUTPUT_EXTRA_ENTRY: i32 = -3;
pub const OUTPUT_FORMAT_ERROR: i32 = -4;

#[repr(C)]
pub enum Variable {
    AlternateOn = 0,
    AlternateSavedX,
    AlternateSavedY,
    BracketedPaste,
    CursorBlinking,
    CursorColour,
    CursorFlag,
    CursorShape,
    CursorX,
    CursorY,
    FocusFlag,
    InsertFlag,
    KeypadCursorFlag,
    KeypadFlag,
    MouseAllFlag,
    MouseAnyFlag,
    MouseButtonFlag,
    MouseSgrFlag,
    MouseStandardFlag,
    MouseUtf8Flag,
    OriginFlag,
    PaneId,
    PaneTabs,
    ScrollRegionLower,
    ScrollRegionUpper,
    SessionId,
    Version,
    WindowId,
    WindowWidth,
    WindowHeight,
    WindowLayout,
    WrapFlag,
}

pub const VARIABLE_COUNT: usize = 32;

pub fn variable_name(v: Variable) -> &'static [u8] {
    match v {
        Variable::AlternateOn => b"alternate_on",
        Variable::AlternateSavedX => b"alternate_saved_x",
        Variable::AlternateSavedY => b"alternate_saved_y",
        Variable::BracketedPaste => b"bracketed_paste",
        Variable::CursorBlinking => b"cursor_blinking",
        Variable::CursorColour => b"cursor_colour",
        Variable::CursorFlag => b"cursor_flag",
        Variable::CursorShape => b"cursor_shape",
        Variable::CursorX => b"cursor_x",
        Variable::CursorY => b"cursor_y",
        Variable::FocusFlag => b"focus_flag",
        Variable::InsertFlag => b"insert_flag",
        Variable::KeypadCursorFlag => b"keypad_cursor_flag",
        Variable::KeypadFlag => b"keypad_flag",
        Variable::MouseAllFlag => b"mouse_all_flag",
        Variable::MouseAnyFlag => b"mouse_any_flag",
        Variable::MouseButtonFlag => b"mouse_button_flag",
        Variable::MouseSgrFlag => b"mouse_sgr_flag",
        Variable::MouseStandardFlag => b"mouse_standard_flag",
        Variable::MouseUtf8Flag => b"mouse_utf8_flag",
        Variable::OriginFlag => b"origin_flag",
        Variable::PaneId => b"pane_id",
        Variable::PaneTabs => b"pane_tabs",
        Variable::ScrollRegionLower => b"scroll_region_lower",
        Variable::ScrollRegionUpper => b"scroll_region_upper",
        Variable::SessionId => b"session_id",
        Variable::Version => b"version",
        Variable::WindowId => b"window_id",
        Variable::WindowWidth => b"window_width",
        Variable::WindowHeight => b"window_height",
        Variable::WindowLayout => b"window_layout",
        Variable::WrapFlag => b"wrap_flag",
    }
}

pub fn variable_is_bool(v: Variable) -> bool {
    matches!(
        v,
        Variable::AlternateOn
            | Variable::BracketedPaste
            | Variable::CursorBlinking
            | Variable::CursorFlag
            | Variable::FocusFlag
            | Variable::InsertFlag
            | Variable::KeypadCursorFlag
            | Variable::KeypadFlag
            | Variable::MouseAllFlag
            | Variable::MouseAnyFlag
            | Variable::MouseButtonFlag
            | Variable::MouseSgrFlag
            | Variable::MouseStandardFlag
            | Variable::MouseUtf8Flag
            | Variable::OriginFlag
            | Variable::WrapFlag
    )
}

pub fn variable_is_usize(v: Variable) -> bool {
    matches!(
        v,
        Variable::AlternateSavedX
            | Variable::AlternateSavedY
            | Variable::CursorX
            | Variable::CursorY
            | Variable::ScrollRegionLower
            | Variable::ScrollRegionUpper
            | Variable::SessionId
            | Variable::WindowId
            | Variable::PaneId
            | Variable::WindowWidth
            | Variable::WindowHeight
    )
}

pub fn variable_is_string(v: Variable) -> bool {
    matches!(
        v,
        Variable::CursorColour
            | Variable::CursorShape
            | Variable::PaneTabs
            | Variable::Version
            | Variable::WindowLayout
    )
}

fn parse_usize_str(s: &[u8]) -> Option<usize> {
    if s.is_empty() {
        return None;
    }
    let mut result: usize = 0;
    let mut i: usize = 0;
    while i < s.len() {
        let c = unsafe { *s.get_unchecked(i) };
        if c < b'0' || c > b'9' {
            return None;
        }
        let digit = (c - b'0') as usize;
        result = result.checked_mul(10)?;
        result = result.checked_add(digit)?;
        i += 1;
    }
    Some(result)
}

pub fn parse_variable_bool(value: &[u8]) -> bool {
    if value.len() == 1 {
        let c = unsafe { *value.get_unchecked(0) };
        c == b'1'
    } else {
        false
    }
}

pub fn parse_variable_usize(v: Variable, value: &[u8]) -> Result<usize, i32> {
    match v {
        Variable::AlternateSavedX
        | Variable::AlternateSavedY
        | Variable::CursorX
        | Variable::CursorY
        | Variable::ScrollRegionLower
        | Variable::ScrollRegionUpper
        | Variable::WindowWidth
        | Variable::WindowHeight => {
            parse_usize_str(value).ok_or(OUTPUT_FORMAT_ERROR)
        }
        Variable::SessionId => {
            if value.len() < 2 {
                return Err(OUTPUT_FORMAT_ERROR);
            }
            if unsafe { *value.get_unchecked(0) } != b'$' {
                return Err(OUTPUT_FORMAT_ERROR);
            }
            parse_usize_str(unsafe { value.get_unchecked(1..) })
                .ok_or(OUTPUT_FORMAT_ERROR)
        }
        Variable::WindowId => {
            if value.len() < 2 {
                return Err(OUTPUT_FORMAT_ERROR);
            }
            if unsafe { *value.get_unchecked(0) } != b'@' {
                return Err(OUTPUT_FORMAT_ERROR);
            }
            parse_usize_str(unsafe { value.get_unchecked(1..) })
                .ok_or(OUTPUT_FORMAT_ERROR)
        }
        Variable::PaneId => {
            if value.len() < 2 {
                return Err(OUTPUT_FORMAT_ERROR);
            }
            if unsafe { *value.get_unchecked(0) } != b'%' {
                return Err(OUTPUT_FORMAT_ERROR);
            }
            parse_usize_str(unsafe { value.get_unchecked(1..) })
                .ok_or(OUTPUT_FORMAT_ERROR)
        }
        _ => Err(OUTPUT_FORMAT_ERROR),
    }
}

pub fn parse_variable_string<'a>(v: Variable, value: &'a [u8]) -> Result<&'a [u8], i32> {
    match v {
        Variable::CursorColour
        | Variable::CursorShape
        | Variable::PaneTabs
        | Variable::Version
        | Variable::WindowLayout => Ok(value),
        _ => Err(OUTPUT_FORMAT_ERROR),
    }
}

pub fn format_variable_name(v: Variable, out: *mut u8, out_len: usize, out_written: *mut usize) -> i32 {
    let name = variable_name(v);
    let prefix = b"#{";
    let suffix = b"}";
    let total = prefix.len() + name.len() + suffix.len();
    if total > out_len {
        return OUTPUT_PARSE_ERROR;
    }
    unsafe {
        let mut pos: usize = 0;
        let mut i: usize = 0;
        while i < prefix.len() {
            *out.add(pos) = *prefix.get_unchecked(i);
            pos += 1;
            i += 1;
        }
        i = 0;
        while i < name.len() {
            *out.add(pos) = *name.get_unchecked(i);
            pos += 1;
            i += 1;
        }
        i = 0;
        while i < suffix.len() {
            *out.add(pos) = *suffix.get_unchecked(i);
            pos += 1;
            i += 1;
        }
        *out_written = pos;
    }
    0
}

pub fn format_variables(
    vars: *const Variable,
    vars_len: usize,
    delimiter: u8,
    out: *mut u8,
    out_len: usize,
    out_written: *mut usize,
) -> i32 {
    let mut pos: usize = 0;
    let mut i: usize = 0;
    while i < vars_len {
        if i > 0 {
            if pos >= out_len {
                return OUTPUT_PARSE_ERROR;
            }
            unsafe { *out.add(pos) = delimiter; }
            pos += 1;
        }
        let v = unsafe { core::ptr::read(vars.add(i)) };
        let mut written: usize = 0;
        let remaining = if pos < out_len { out_len - pos } else { 0 };
        let result = format_variable_name(
            v,
            unsafe { out.add(pos) },
            remaining,
            &mut written,
        );
        if result < 0 {
            return result;
        }
        pos += written;
        i += 1;
    }
    unsafe { *out_written = pos; }
    0
}

#[repr(C)]
pub struct ParsedPaneState {
    pub pane_id: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_flag: bool,
    pub cursor_shape_ptr: *const u8,
    pub cursor_shape_len: usize,
    pub cursor_colour_ptr: *const u8,
    pub cursor_colour_len: usize,
    pub cursor_blinking: bool,
    pub alternate_on: bool,
    pub alternate_saved_x: usize,
    pub alternate_saved_y: usize,
    pub insert_flag: bool,
    pub wrap_flag: bool,
    pub keypad_flag: bool,
    pub keypad_cursor_flag: bool,
    pub origin_flag: bool,
    pub mouse_all_flag: bool,
    pub mouse_any_flag: bool,
    pub mouse_button_flag: bool,
    pub mouse_standard_flag: bool,
    pub mouse_utf8_flag: bool,
    pub mouse_sgr_flag: bool,
    pub focus_flag: bool,
    pub bracketed_paste: bool,
    pub scroll_region_upper: usize,
    pub scroll_region_lower: usize,
    pub pane_tabs_ptr: *const u8,
    pub pane_tabs_len: usize,
}

pub const LIST_PANES_VARS: [Variable; 27] = [
    Variable::PaneId,
    Variable::CursorX,
    Variable::CursorY,
    Variable::CursorFlag,
    Variable::CursorShape,
    Variable::CursorColour,
    Variable::CursorBlinking,
    Variable::AlternateOn,
    Variable::AlternateSavedX,
    Variable::AlternateSavedY,
    Variable::InsertFlag,
    Variable::WrapFlag,
    Variable::KeypadFlag,
    Variable::KeypadCursorFlag,
    Variable::OriginFlag,
    Variable::MouseAllFlag,
    Variable::MouseAnyFlag,
    Variable::MouseButtonFlag,
    Variable::MouseStandardFlag,
    Variable::MouseUtf8Flag,
    Variable::MouseSgrFlag,
    Variable::FocusFlag,
    Variable::BracketedPaste,
    Variable::ScrollRegionUpper,
    Variable::ScrollRegionLower,
    Variable::PaneTabs,
    Variable::Version,
];

pub const LIST_PANES_DELIM: u8 = b';';

#[repr(C)]
pub struct ParsedWindowInfo {
    pub session_id: usize,
    pub window_id: usize,
    pub window_width: usize,
    pub window_height: usize,
    pub window_layout_ptr: *const u8,
    pub window_layout_len: usize,
}

pub const LIST_WINDOWS_VARS: [Variable; 5] = [
    Variable::SessionId,
    Variable::WindowId,
    Variable::WindowWidth,
    Variable::WindowHeight,
    Variable::WindowLayout,
];

pub const LIST_WINDOWS_DELIM: u8 = b' ';

pub const TMUX_VERSION_VARS: [Variable; 1] = [Variable::Version];
pub const TMUX_VERSION_DELIM: u8 = b' ';

fn split_at_delim(s: &[u8], delim: u8) -> (&[u8], &[u8]) {
    let mut i: usize = 0;
    while i < s.len() {
        if unsafe { *s.get_unchecked(i) } == delim {
            return (
                unsafe { s.get_unchecked(..i) },
                unsafe { s.get_unchecked(i + 1..) },
            );
        }
        i += 1;
    }
    (s, &[])
}

fn count_delims(s: &[u8], delim: u8) -> usize {
    let mut count: usize = 0;
    let mut i: usize = 0;
    while i < s.len() {
        if unsafe { *s.get_unchecked(i) } == delim {
            count += 1;
        }
        i += 1;
    }
    count
}

pub fn parse_pane_state_line(line: &[u8], out: *mut ParsedPaneState) -> i32 {
    if out.is_null() {
        return OUTPUT_PARSE_ERROR;
    }
    let expected_fields = 27;
    let delim_count = count_delims(line, LIST_PANES_DELIM);
    if delim_count + 1 != expected_fields {
        return OUTPUT_MISSING_ENTRY;
    }

    let mut remaining = line;

    macro_rules! next_field {
        () => {{
            let (field, rest) = split_at_delim(remaining, LIST_PANES_DELIM);
            remaining = rest;
            field
        }};
    }

    let pane_id_str = next_field!();
    let cursor_x_str = next_field!();
    let cursor_y_str = next_field!();
    let cursor_flag_str = next_field!();
    let cursor_shape_str = next_field!();
    let cursor_colour_str = next_field!();
    let cursor_blinking_str = next_field!();
    let alternate_on_str = next_field!();
    let alternate_saved_x_str = next_field!();
    let alternate_saved_y_str = next_field!();
    let insert_flag_str = next_field!();
    let wrap_flag_str = next_field!();
    let keypad_flag_str = next_field!();
    let keypad_cursor_flag_str = next_field!();
    let origin_flag_str = next_field!();
    let mouse_all_flag_str = next_field!();
    let mouse_any_flag_str = next_field!();
    let mouse_button_flag_str = next_field!();
    let mouse_standard_flag_str = next_field!();
    let mouse_utf8_flag_str = next_field!();
    let mouse_sgr_flag_str = next_field!();
    let focus_flag_str = next_field!();
    let bracketed_paste_str = next_field!();
    let scroll_region_upper_str = next_field!();
    let scroll_region_lower_str = next_field!();
    let pane_tabs_str = next_field!();
    let _version_str = next_field!();
    let _ = remaining;

    unsafe {
        (*out).pane_id = match parse_variable_usize(Variable::PaneId, pane_id_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).cursor_x = match parse_variable_usize(Variable::CursorX, cursor_x_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).cursor_y = match parse_variable_usize(Variable::CursorY, cursor_y_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).cursor_flag = parse_variable_bool(cursor_flag_str);
        (*out).cursor_shape_ptr = cursor_shape_str.as_ptr();
        (*out).cursor_shape_len = cursor_shape_str.len();
        (*out).cursor_colour_ptr = cursor_colour_str.as_ptr();
        (*out).cursor_colour_len = cursor_colour_str.len();
        (*out).cursor_blinking = parse_variable_bool(cursor_blinking_str);
        (*out).alternate_on = parse_variable_bool(alternate_on_str);
        (*out).alternate_saved_x = match parse_variable_usize(Variable::AlternateSavedX, alternate_saved_x_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).alternate_saved_y = match parse_variable_usize(Variable::AlternateSavedY, alternate_saved_y_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).insert_flag = parse_variable_bool(insert_flag_str);
        (*out).wrap_flag = parse_variable_bool(wrap_flag_str);
        (*out).keypad_flag = parse_variable_bool(keypad_flag_str);
        (*out).keypad_cursor_flag = parse_variable_bool(keypad_cursor_flag_str);
        (*out).origin_flag = parse_variable_bool(origin_flag_str);
        (*out).mouse_all_flag = parse_variable_bool(mouse_all_flag_str);
        (*out).mouse_any_flag = parse_variable_bool(mouse_any_flag_str);
        (*out).mouse_button_flag = parse_variable_bool(mouse_button_flag_str);
        (*out).mouse_standard_flag = parse_variable_bool(mouse_standard_flag_str);
        (*out).mouse_utf8_flag = parse_variable_bool(mouse_utf8_flag_str);
        (*out).mouse_sgr_flag = parse_variable_bool(mouse_sgr_flag_str);
        (*out).focus_flag = parse_variable_bool(focus_flag_str);
        (*out).bracketed_paste = parse_variable_bool(bracketed_paste_str);
        (*out).scroll_region_upper = match parse_variable_usize(Variable::ScrollRegionUpper, scroll_region_upper_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).scroll_region_lower = match parse_variable_usize(Variable::ScrollRegionLower, scroll_region_lower_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).pane_tabs_ptr = pane_tabs_str.as_ptr();
        (*out).pane_tabs_len = pane_tabs_str.len();
    }

    0
}

pub fn parse_window_info_line(line: &[u8], out: *mut ParsedWindowInfo) -> i32 {
    if out.is_null() {
        return OUTPUT_PARSE_ERROR;
    }
    let expected_fields = 5;
    let delim_count = count_delims(line, LIST_WINDOWS_DELIM);
    if delim_count + 1 != expected_fields {
        return OUTPUT_MISSING_ENTRY;
    }

    let mut remaining = line;

    macro_rules! next_field {
        () => {{
            let (field, rest) = split_at_delim(remaining, LIST_WINDOWS_DELIM);
            remaining = rest;
            field
        }};
    }

    let session_id_str = next_field!();
    let window_id_str = next_field!();
    let window_width_str = next_field!();
    let window_height_str = next_field!();
    let window_layout_str = remaining;

    unsafe {
        (*out).session_id = match parse_variable_usize(Variable::SessionId, session_id_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).window_id = match parse_variable_usize(Variable::WindowId, window_id_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).window_width = match parse_variable_usize(Variable::WindowWidth, window_width_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).window_height = match parse_variable_usize(Variable::WindowHeight, window_height_str) {
            Ok(v) => v,
            Err(_) => return OUTPUT_FORMAT_ERROR,
        };
        (*out).window_layout_ptr = window_layout_str.as_ptr();
        (*out).window_layout_len = window_layout_str.len();
    }

    0
}

pub fn parse_version_line(line: &[u8], out_ptr: *mut *const u8, out_len: *mut usize) -> i32 {
    if out_ptr.is_null() || out_len.is_null() {
        return OUTPUT_PARSE_ERROR;
    }
    let trimmed = trim_whitespace(line);
    if trimmed.is_empty() {
        return OUTPUT_FORMAT_ERROR;
    }
    unsafe {
        *out_ptr = trimmed.as_ptr();
        *out_len = trimmed.len();
    }
    0
}

fn trim_whitespace(s: &[u8]) -> &[u8] {
    if s.is_empty() {
        return s;
    }
    let mut start: usize = 0;
    while start < s.len() {
        let c = unsafe { *s.get_unchecked(start) };
        if c != b' ' && c != b'\t' && c != b'\r' && c != b'\n' {
            break;
        }
        start += 1;
    }
    let mut end = s.len();
    while end > start {
        let c = unsafe { *s.get_unchecked(end - 1) };
        if c != b' ' && c != b'\t' && c != b'\r' && c != b'\n' {
            break;
        }
        end -= 1;
    }
    if start >= end {
        return &[];
    }
    unsafe { s.get_unchecked(start..end) }
}
