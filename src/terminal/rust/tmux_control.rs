use core::ptr;
use crate::allocator::*;

const MAX_BUFFER_DEFAULT: usize = 1024 * 1024;

#[repr(C)]
pub enum ParserState {
    Idle = 0,
    Broken = 1,
    Notification = 2,
    Block = 3,
}

#[repr(C)]
pub struct ControlParser {
    pub state: ParserState,
    buf_ptr: *mut u8,
    buf_len: usize,
    buf_cap: usize,
    pub max_bytes: usize,
}

#[repr(C)]
pub enum NotificationTag {
    None = 0,
    Enter = 1,
    Exit = 2,
    BlockEnd = 3,
    BlockErr = 4,
    Output = 5,
    SessionChanged = 6,
    SessionsChanged = 7,
    LayoutChange = 8,
    WindowAdd = 9,
    WindowRenamed = 10,
    WindowPaneChanged = 11,
    ClientDetached = 12,
    ClientSessionChanged = 13,
}

#[repr(C)]
pub struct Notification {
    pub tag: NotificationTag,
    pub pane_id: usize,
    pub data_ptr: *const u8,
    pub data_len: usize,
    pub id: usize,
    pub name_ptr: *const u8,
    pub name_len: usize,
    pub window_id: usize,
    pub layout_ptr: *const u8,
    pub layout_len: usize,
    pub visible_layout_ptr: *const u8,
    pub visible_layout_len: usize,
    pub raw_flags_ptr: *const u8,
    pub raw_flags_len: usize,
    pub client_ptr: *const u8,
    pub client_len: usize,
    pub session_id: usize,
}

impl Notification {
    fn empty() -> Notification {
        Notification {
            tag: NotificationTag::None,
            pane_id: 0,
            data_ptr: ptr::null(),
            data_len: 0,
            id: 0,
            name_ptr: ptr::null(),
            name_len: 0,
            window_id: 0,
            layout_ptr: ptr::null(),
            layout_len: 0,
            visible_layout_ptr: ptr::null(),
            visible_layout_len: 0,
            raw_flags_ptr: ptr::null(),
            raw_flags_len: 0,
            client_ptr: ptr::null(),
            client_len: 0,
            session_id: 0,
        }
    }
}

pub const CONTROL_OK: i32 = 0;
pub const CONTROL_NO_NOTIFICATION: i32 = 1;
pub const CONTROL_OUT_OF_MEMORY: i32 = -1;
pub const CONTROL_BROKEN: i32 = -2;

impl ControlParser {
    pub fn init(_alloc: *const GhosttyAllocator) -> ControlParser {
        ControlParser {
            state: ParserState::Idle,
            buf_ptr: ptr::null_mut(),
            buf_len: 0,
            buf_cap: 0,
            max_bytes: MAX_BUFFER_DEFAULT,
        }
    }

    pub fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if matches!(self.state, ParserState::Broken) {
            return;
        }
        self.free_buf(alloc);
    }

    fn free_buf(&mut self, alloc: *const GhosttyAllocator) {
        if !self.buf_ptr.is_null() && self.buf_cap > 0 {
            unsafe { alloc_free_impl(alloc, self.buf_ptr, self.buf_cap); }
        }
        self.buf_ptr = ptr::null_mut();
        self.buf_len = 0;
        self.buf_cap = 0;
    }

    fn grow_buf(&mut self, alloc: *const GhosttyAllocator) -> bool {
        let new_cap = if self.buf_cap == 0 { 256 } else { self.buf_cap * 2 };
        if new_cap > self.max_bytes {
            return false;
        }
        let new_ptr = unsafe { alloc_alloc_impl(alloc, new_cap) };
        if new_ptr.is_null() {
            return false;
        }
        if !self.buf_ptr.is_null() && self.buf_len > 0 {
            unsafe {
                ptr::copy_nonoverlapping(self.buf_ptr, new_ptr, self.buf_len);
                alloc_free_impl(alloc, self.buf_ptr, self.buf_cap);
            }
        }
        self.buf_ptr = new_ptr;
        self.buf_cap = new_cap;
        true
    }

    fn push_byte(&mut self, byte: u8, alloc: *const GhosttyAllocator) -> bool {
        if self.buf_len >= self.buf_cap {
            if !self.grow_buf(alloc) {
                return false;
            }
        }
        unsafe { *self.buf_ptr.add(self.buf_len) = byte; }
        self.buf_len += 1;
        true
    }

    fn clear_retaining_capacity(&mut self) {
        self.buf_len = 0;
    }

    fn buf_slice(&self) -> (*const u8, usize) {
        (self.buf_ptr as *const u8, self.buf_len)
    }

    fn mark_broken(&mut self, alloc: *const GhosttyAllocator) {
        self.state = ParserState::Broken;
        self.free_buf(alloc);
    }

    pub fn put(
        &mut self,
        byte: u8,
        alloc: *const GhosttyAllocator,
        out: *mut Notification,
    ) -> i32 {
        if matches!(self.state, ParserState::Broken) {
            return CONTROL_NO_NOTIFICATION;
        }

        if self.buf_len >= self.max_bytes {
            self.mark_broken(alloc);
            return CONTROL_OUT_OF_MEMORY;
        }

        match self.state {
            ParserState::Broken => return CONTROL_NO_NOTIFICATION,

            ParserState::Idle => {
                if byte != b'%' {
                    self.mark_broken(alloc);
                    if !out.is_null() {
                        unsafe {
                            *out = Notification::empty();
                            (*out).tag = NotificationTag::Exit;
                        }
                    }
                    return CONTROL_OK;
                }
                self.clear_retaining_capacity();
                self.state = ParserState::Notification;
            }

            ParserState::Notification => {
                if byte == b'\n' {
                    let result = self.parse_notification(out);
                    if result < 0 && result != CONTROL_OUT_OF_MEMORY {
                        return CONTROL_NO_NOTIFICATION;
                    }
                    return result;
                }
            }

            ParserState::Block => {
                if byte == b'\n' {
                    let (buf_ptr, buf_len) = self.buf_slice();
                    if buf_ptr.is_null() || buf_len == 0 {
                        return CONTROL_NO_NOTIFICATION;
                    }

                    let last_newline = last_index_of_byte_raw(buf_ptr, buf_len, b'\n');
                    let line_start = match last_newline {
                        Some(idx) => idx + 1,
                        None => 0,
                    };

                    let line_ptr = unsafe { buf_ptr.add(line_start) };
                    let line_len = buf_len - line_start;

                    if let Some(is_err) = parse_block_terminator_raw(line_ptr, line_len) {
                        let output = trim_right_cr_lf_raw(buf_ptr, line_start);

                        self.state = ParserState::Idle;
                        if !out.is_null() {
                            unsafe {
                                *out = Notification::empty();
                                (*out).tag = if is_err {
                                    NotificationTag::BlockErr
                                } else {
                                    NotificationTag::BlockEnd
                                };
                                (*out).data_ptr = output.0;
                                (*out).data_len = output.1;
                            }
                        }
                        return CONTROL_OK;
                    }
                }
            }
        }

        if !self.push_byte(byte, alloc) {
            self.mark_broken(alloc);
            return CONTROL_OUT_OF_MEMORY;
        }

        CONTROL_NO_NOTIFICATION
    }

    fn parse_notification(&mut self, out: *mut Notification) -> i32 {
        let (buf_ptr, buf_len) = self.buf_slice();
        if buf_ptr.is_null() || buf_len == 0 {
            self.clear_retaining_capacity();
            self.state = ParserState::Idle;
            return CONTROL_NO_NOTIFICATION;
        }

        let line_ptr = buf_ptr;
        let mut line_len = buf_len;

        if unsafe { *buf_ptr.add(buf_len - 1) } == b'\r' {
            line_len = buf_len - 1;
        }

        let cmd_end = index_of_byte_raw(line_ptr, line_len, b' ').unwrap_or(line_len);

        if bytes_eq_raw(line_ptr, cmd_end, b"%begin") {
            self.state = ParserState::Block;
            self.clear_retaining_capacity();
            return CONTROL_NO_NOTIFICATION;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%output") {
            let result = parse_output_raw(line_ptr, line_len, out);
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%session-changed") {
            let result = parse_session_changed_raw(line_ptr, line_len, out);
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%sessions-changed") {
            if !bytes_eq_raw(line_ptr, line_len, b"%sessions-changed") {
                self.clear_retaining_capacity();
                self.state = ParserState::Idle;
                return CONTROL_NO_NOTIFICATION;
            }
            self.clear_retaining_capacity();
            self.state = ParserState::Idle;
            if !out.is_null() {
                unsafe {
                    *out = Notification::empty();
                    (*out).tag = NotificationTag::SessionsChanged;
                }
            }
            return CONTROL_OK;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%layout-change") {
            let result = parse_layout_change_raw(line_ptr, line_len, out);
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%window-add") {
            let result = parse_window_add_raw(line_ptr, line_len, out);
            self.clear_retaining_capacity();
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%window-renamed") {
            let result = parse_window_renamed_raw(line_ptr, line_len, out);
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%window-pane-changed") {
            let result = parse_window_pane_changed_raw(line_ptr, line_len, out);
            self.clear_retaining_capacity();
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%client-detached") {
            let result = parse_client_detached_raw(line_ptr, line_len, out);
            self.state = ParserState::Idle;
            return result;
        }

        if bytes_eq_raw(line_ptr, cmd_end, b"%client-session-changed") {
            let result = parse_client_session_changed_raw(line_ptr, line_len, out);
            self.state = ParserState::Idle;
            return result;
        }

        self.clear_retaining_capacity();
        self.state = ParserState::Idle;
        CONTROL_NO_NOTIFICATION
    }
}

fn parse_output_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%output %";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let after_prefix = unsafe { line_ptr.add(prefix.len()) };
    let after_prefix_len = line_len - prefix.len();

    let space_idx = match index_of_byte_raw(after_prefix, after_prefix_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let pane_id = match parse_usize_raw(after_prefix, space_idx) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let data_ptr = unsafe { after_prefix.add(space_idx + 1) };
    let data_len = after_prefix_len - space_idx - 1;

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::Output;
            (*out).pane_id = pane_id;
            (*out).data_ptr = data_ptr;
            (*out).data_len = data_len;
        }
    }
    CONTROL_OK
}

fn parse_session_changed_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%session-changed $";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let after_prefix = unsafe { line_ptr.add(prefix.len()) };
    let after_prefix_len = line_len - prefix.len();

    let space_idx = match index_of_byte_raw(after_prefix, after_prefix_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let id = match parse_usize_raw(after_prefix, space_idx) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let name_ptr = unsafe { after_prefix.add(space_idx + 1) };
    let name_len = after_prefix_len - space_idx - 1;

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::SessionChanged;
            (*out).id = id;
            (*out).name_ptr = name_ptr;
            (*out).name_len = name_len;
        }
    }
    CONTROL_OK
}

fn parse_layout_change_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%layout-change @";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let after_at = unsafe { line_ptr.add(prefix.len()) };
    let after_at_len = line_len - prefix.len();

    let space1 = match index_of_byte_raw(after_at, after_at_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let window_id = match parse_usize_raw(after_at, space1) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let after_id = unsafe { after_at.add(space1 + 1) };
    let after_id_len = after_at_len - space1 - 1;

    let space2 = match index_of_byte_raw(after_id, after_id_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };
    let layout_ptr = after_id;
    let layout_len = space2;

    let after_layout = unsafe { after_id.add(space2 + 1) };
    let after_layout_len = after_id_len - space2 - 1;

    let space3 = match index_of_byte_raw(after_layout, after_layout_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };
    let visible_layout_ptr = after_layout;
    let visible_layout_len = space3;

    let raw_flags_ptr = unsafe { after_layout.add(space3 + 1) };
    let raw_flags_len = after_layout_len - space3 - 1;

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::LayoutChange;
            (*out).window_id = window_id;
            (*out).layout_ptr = layout_ptr;
            (*out).layout_len = layout_len;
            (*out).visible_layout_ptr = visible_layout_ptr;
            (*out).visible_layout_len = visible_layout_len;
            (*out).raw_flags_ptr = raw_flags_ptr;
            (*out).raw_flags_len = raw_flags_len;
        }
    }
    CONTROL_OK
}

fn parse_window_add_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%window-add @";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let id_ptr = unsafe { line_ptr.add(prefix.len()) };
    let id_len = line_len - prefix.len();

    let id = match parse_usize_raw(id_ptr, id_len) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::WindowAdd;
            (*out).id = id;
        }
    }
    CONTROL_OK
}

fn parse_window_renamed_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%window-renamed @";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let after_at = unsafe { line_ptr.add(prefix.len()) };
    let after_at_len = line_len - prefix.len();

    let space_idx = match index_of_byte_raw(after_at, after_at_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let id = match parse_usize_raw(after_at, space_idx) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let name_ptr = unsafe { after_at.add(space_idx + 1) };
    let name_len = after_at_len - space_idx - 1;

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::WindowRenamed;
            (*out).id = id;
            (*out).name_ptr = name_ptr;
            (*out).name_len = name_len;
        }
    }
    CONTROL_OK
}

fn parse_window_pane_changed_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%window-pane-changed @";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let after_at = unsafe { line_ptr.add(prefix.len()) };
    let after_at_len = line_len - prefix.len();

    let space_idx = match index_of_byte_raw(after_at, after_at_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let window_id = match parse_usize_raw(after_at, space_idx) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let after_space = unsafe { after_at.add(space_idx + 1) };
    let after_space_len = after_at_len - space_idx - 1;

    if after_space_len == 0 || unsafe { *after_space } != b'%' {
        return CONTROL_NO_NOTIFICATION;
    }

    let pane_id_ptr = unsafe { after_space.add(1) };
    let pane_id_len = after_space_len - 1;

    let pane_id = match parse_usize_raw(pane_id_ptr, pane_id_len) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::WindowPaneChanged;
            (*out).window_id = window_id;
            (*out).pane_id = pane_id;
        }
    }
    CONTROL_OK
}

fn parse_client_detached_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%client-detached ";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let client_ptr = unsafe { line_ptr.add(prefix.len()) };
    let client_len = line_len - prefix.len();

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::ClientDetached;
            (*out).client_ptr = client_ptr;
            (*out).client_len = client_len;
        }
    }
    CONTROL_OK
}

fn parse_client_session_changed_raw(line_ptr: *const u8, line_len: usize, out: *mut Notification) -> i32 {
    let prefix = b"%client-session-changed ";
    if line_len < prefix.len() {
        return CONTROL_NO_NOTIFICATION;
    }
    if !bytes_eq_raw(line_ptr, prefix.len(), prefix) {
        return CONTROL_NO_NOTIFICATION;
    }

    let rest = unsafe { line_ptr.add(prefix.len()) };
    let rest_len = line_len - prefix.len();

    let space1 = match index_of_byte_raw(rest, rest_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };
    let client_ptr = rest;
    let client_len = space1;

    let after_client = unsafe { rest.add(space1 + 1) };
    let after_client_len = rest_len - space1 - 1;

    if after_client_len == 0 || unsafe { *after_client } != b'$' {
        return CONTROL_NO_NOTIFICATION;
    }

    let after_dollar = unsafe { after_client.add(1) };
    let after_dollar_len = after_client_len - 1;

    let space2 = match index_of_byte_raw(after_dollar, after_dollar_len, b' ') {
        Some(idx) => idx,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let session_id = match parse_usize_raw(after_dollar, space2) {
        Some(v) => v,
        None => return CONTROL_NO_NOTIFICATION,
    };

    let name_ptr = unsafe { after_dollar.add(space2 + 1) };
    let name_len = after_dollar_len - space2 - 1;

    if !out.is_null() {
        unsafe {
            *out = Notification::empty();
            (*out).tag = NotificationTag::ClientSessionChanged;
            (*out).client_ptr = client_ptr;
            (*out).client_len = client_len;
            (*out).session_id = session_id;
            (*out).name_ptr = name_ptr;
            (*out).name_len = name_len;
        }
    }
    CONTROL_OK
}

fn parse_block_terminator_raw(line_ptr: *const u8, line_len: usize) -> Option<bool> {
    let mut actual_len = line_len;
    if actual_len > 0 && unsafe { *line_ptr.add(actual_len - 1) } == b'\r' {
        actual_len -= 1;
    }

    let mut pos: usize = 0;

    let cmd_end = {
        let mut i = pos;
        while i < actual_len && unsafe { *line_ptr.add(i) } != b' ' {
            i += 1;
        }
        i
    };

    let is_err: bool;
    if bytes_eq_raw(line_ptr, cmd_end, b"%end") {
        is_err = false;
    } else if bytes_eq_raw(line_ptr, cmd_end, b"%error") {
        is_err = true;
    } else {
        return None;
    }

    if cmd_end >= actual_len {
        return None;
    }
    pos = cmd_end + 1;

    let time_end = {
        let mut i = pos;
        while i < actual_len && unsafe { *line_ptr.add(i) } != b' ' {
            i += 1;
        }
        i
    };
    if parse_usize_raw(unsafe { line_ptr.add(pos) }, time_end - pos).is_none() {
        return None;
    }
    if time_end >= actual_len {
        return None;
    }
    pos = time_end + 1;

    let cmd_id_end = {
        let mut i = pos;
        while i < actual_len && unsafe { *line_ptr.add(i) } != b' ' {
            i += 1;
        }
        i
    };
    if parse_usize_raw(unsafe { line_ptr.add(pos) }, cmd_id_end - pos).is_none() {
        return None;
    }
    if cmd_id_end >= actual_len {
        return None;
    }
    pos = cmd_id_end + 1;

    let flags_end = {
        let mut i = pos;
        while i < actual_len && unsafe { *line_ptr.add(i) } != b' ' {
            i += 1;
        }
        i
    };
    if parse_usize_raw(unsafe { line_ptr.add(pos) }, flags_end - pos).is_none() {
        return None;
    }

    if flags_end < actual_len {
        return None;
    }

    Some(is_err)
}

fn bytes_eq_raw(ptr: *const u8, len: usize, expected: &[u8]) -> bool {
    if len != expected.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < len {
        if unsafe { *ptr.add(i) } != unsafe { *expected.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

fn index_of_byte_raw(ptr: *const u8, len: usize, byte: u8) -> Option<usize> {
    let mut i: usize = 0;
    while i < len {
        if unsafe { *ptr.add(i) } == byte {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn last_index_of_byte_raw(ptr: *const u8, len: usize, byte: u8) -> Option<usize> {
    let mut i = len;
    while i > 0 {
        i -= 1;
        if unsafe { *ptr.add(i) } == byte {
            return Some(i);
        }
    }
    None
}

fn trim_right_cr_lf_raw(ptr: *const u8, len: usize) -> (*const u8, usize) {
    let mut end = len;
    while end > 0 {
        let c = unsafe { *ptr.add(end - 1) };
        if c != b'\r' && c != b'\n' {
            break;
        }
        end -= 1;
    }
    (ptr, end)
}

fn parse_usize_raw(ptr: *const u8, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let mut result: usize = 0;
    let mut i: usize = 0;
    while i < len {
        let c = unsafe { *ptr.add(i) };
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
