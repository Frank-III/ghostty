use core::ffi::c_void;
use core::ptr;
use crate::allocator::*;
use crate::csi::EraseDisplay;
use crate::cursor_style::CursorVisualStyle;
use crate::mode_def::ModeTag;
use crate::screen_set::ScreenKey;
use crate::size_types::CellCountInt;
use crate::terminal_types::{Terminal, TABSTOP_INTERVAL};
use super::control::*;
use super::layout::*;
use super::output::*;

const COMMAND_QUEUE_INITIAL: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewerState {
    StartupBlock = 0,
    StartupSession = 1,
    Defunct = 2,
    CommandQueue = 3,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionTag {
    None = 0,
    Exit = 1,
    Command = 2,
    Windows = 3,
}

#[repr(C)]
pub struct Action {
    pub tag: ActionTag,
    pub command_ptr: *const u8,
    pub command_len: usize,
    pub windows_ptr: *const ViewerWindow,
    pub windows_len: usize,
}

impl Action {
    fn empty() -> Action {
        Action {
            tag: ActionTag::None,
            command_ptr: ptr::null(),
            command_len: 0,
            windows_ptr: ptr::null(),
            windows_len: 0,
        }
    }

    fn exit_action() -> Action {
        Action {
            tag: ActionTag::Exit,
            command_ptr: ptr::null(),
            command_len: 0,
            windows_ptr: ptr::null(),
            windows_len: 0,
        }
    }
}

#[repr(C)]
pub struct ViewerWindow {
    pub id: usize,
    pub width: usize,
    pub height: usize,
    pub layout: Layout,
}

#[repr(C)]
pub struct ViewerPane {
    pub terminal: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommandTag {
    ListWindows = 0,
    PaneHistory = 1,
    PaneVisible = 2,
    PaneState = 3,
    TmuxVersion = 4,
    User = 5,
}

#[repr(C)]
pub struct Command {
    pub tag: CommandTag,
    pub pane_id: usize,
    pub screen_is_alternate: bool,
    pub user_cmd_ptr: *mut u8,
    pub user_cmd_len: usize,
}

impl Command {
    fn list_windows() -> Command {
        Command {
            tag: CommandTag::ListWindows,
            pane_id: 0,
            screen_is_alternate: false,
            user_cmd_ptr: ptr::null_mut(),
            user_cmd_len: 0,
        }
    }

    fn pane_history(id: usize, is_alternate: bool) -> Command {
        Command {
            tag: CommandTag::PaneHistory,
            pane_id: id,
            screen_is_alternate: is_alternate,
            user_cmd_ptr: ptr::null_mut(),
            user_cmd_len: 0,
        }
    }

    fn pane_visible(id: usize, is_alternate: bool) -> Command {
        Command {
            tag: CommandTag::PaneVisible,
            pane_id: id,
            screen_is_alternate: is_alternate,
            user_cmd_ptr: ptr::null_mut(),
            user_cmd_len: 0,
        }
    }

    fn pane_state() -> Command {
        Command {
            tag: CommandTag::PaneState,
            pane_id: 0,
            screen_is_alternate: false,
            user_cmd_ptr: ptr::null_mut(),
            user_cmd_len: 0,
        }
    }

    fn tmux_version() -> Command {
        Command {
            tag: CommandTag::TmuxVersion,
            pane_id: 0,
            screen_is_alternate: false,
            user_cmd_ptr: ptr::null_mut(),
            user_cmd_len: 0,
        }
    }

    fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if matches!(self.tag, CommandTag::User) {
            if !self.user_cmd_ptr.is_null() && self.user_cmd_len > 0 {
                unsafe { alloc_free_impl(alloc, self.user_cmd_ptr, self.user_cmd_len); }
            }
        }
    }
}

struct CommandQueue {
    ptr: *mut Command,
    len: usize,
    cap: usize,
    head: usize,
}

impl CommandQueue {
    fn new(alloc: *const GhosttyAllocator, initial: usize) -> CommandQueue {
        let cmd_size = core::mem::size_of::<Command>();
        let total = cmd_size * initial;
        let raw = if total > 0 {
            unsafe { alloc_alloc_impl(alloc, total) }
        } else {
            ptr::null_mut()
        };
        if !raw.is_null() {
            unsafe { ptr::write_bytes(raw, 0, total); }
        }
        CommandQueue {
            ptr: raw as *mut Command,
            len: 0,
            cap: if raw.is_null() { 0 } else { initial },
            head: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn queue_len(&self) -> usize {
        self.len
    }

    fn first(&self) -> Option<*const Command> {
        if self.len == 0 || self.ptr.is_null() {
            return None;
        }
        let idx = self.head % self.cap;
        Some(unsafe { self.ptr.add(idx) })
    }

    fn delete_oldest(&mut self, count: usize) {
        let actual = if count > self.len { self.len } else { count };
        self.head = (self.head + actual) % self.cap;
        self.len -= actual;
    }

    fn push(&mut self, cmd: Command, alloc: *const GhosttyAllocator) -> bool {
        if self.len >= self.cap {
            if !self.grow(alloc) {
                return false;
            }
        }
        let idx = (self.head + self.len) % self.cap;
        unsafe { *self.ptr.add(idx) = cmd; }
        self.len += 1;
        true
    }

    fn grow(&mut self, alloc: *const GhosttyAllocator) -> bool {
        let new_cap = if self.cap == 0 { COMMAND_QUEUE_INITIAL } else { self.cap * 2 };
        let cmd_size = core::mem::size_of::<Command>();
        let total = cmd_size * new_cap;
        let new_raw = unsafe { alloc_alloc_impl(alloc, total) };
        if new_raw.is_null() {
            return false;
        }
        unsafe { ptr::write_bytes(new_raw, 0, total); }
        let new_ptr = new_raw as *mut Command;

        let mut i: usize = 0;
        while i < self.len {
            let old_idx = (self.head + i) % self.cap;
            unsafe {
                let src = self.ptr.add(old_idx);
                let dst = new_ptr.add(i);
                ptr::write(dst, ptr::read(src));
            }
            i += 1;
        }

        if !self.ptr.is_null() && self.cap > 0 {
            let old_total = cmd_size * self.cap;
            unsafe { alloc_free_impl(alloc, self.ptr as *mut u8, old_total); }
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
        self.head = 0;
        true
    }

    fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.ptr.is_null() && self.cap > 0 {
            let mut i: usize = 0;
            while i < self.len {
                let idx = (self.head + i) % self.cap;
                unsafe { (*self.ptr.add(idx)).deinit(alloc); }
                i += 1;
            }
            let total = core::mem::size_of::<Command>() * self.cap;
            unsafe { alloc_free_impl(alloc, self.ptr as *mut u8, total); }
        }
        self.ptr = ptr::null_mut();
        self.len = 0;
        self.cap = 0;
        self.head = 0;
    }
}

struct WindowList {
    ptr: *mut ViewerWindow,
    len: usize,
    cap: usize,
}

impl WindowList {
    fn new() -> WindowList {
        WindowList {
            ptr: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    fn items(&self) -> &[ViewerWindow] {
        if self.ptr.is_null() || self.len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    fn push(&mut self, window: ViewerWindow, alloc: *const GhosttyAllocator) -> bool {
        if self.len >= self.cap {
            let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
            let win_size = core::mem::size_of::<ViewerWindow>();
            let total = win_size * new_cap;
            let new_raw = unsafe { alloc_alloc_impl(alloc, total) };
            if new_raw.is_null() {
                return false;
            }
            unsafe { ptr::write_bytes(new_raw, 0, total); }
            if !self.ptr.is_null() && self.len > 0 {
                let old_total = win_size * self.len;
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.ptr as *const u8,
                        new_raw,
                        old_total,
                    );
                    alloc_free_impl(alloc, self.ptr as *mut u8, win_size * self.cap);
                }
            }
            self.ptr = new_raw as *mut ViewerWindow;
            self.cap = new_cap;
        }
        unsafe { *self.ptr.add(self.len) = window; }
        self.len += 1;
        true
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.ptr.is_null() && self.cap > 0 {
            let total = core::mem::size_of::<ViewerWindow>() * self.cap;
            unsafe { alloc_free_impl(alloc, self.ptr as *mut u8, total); }
        }
        self.ptr = ptr::null_mut();
        self.len = 0;
        self.cap = 0;
    }
}

struct PaneMap {
    keys: *mut usize,
    values: *mut ViewerPane,
    len: usize,
    cap: usize,
}

impl PaneMap {
    fn new() -> PaneMap {
        PaneMap {
            keys: ptr::null_mut(),
            values: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    fn count(&self) -> usize {
        self.len
    }

    fn get(&self, key: usize) -> Option<*mut ViewerPane> {
        let mut i: usize = 0;
        while i < self.len {
            if unsafe { *self.keys.add(i) } == key {
                return Some(unsafe { self.values.add(i) });
            }
            i += 1;
        }
        None
    }

    fn contains(&self, key: usize) -> bool {
        self.get(key).is_some()
    }

    fn insert(
        &mut self,
        key: usize,
        value: ViewerPane,
        alloc: *const GhosttyAllocator,
    ) -> bool {
        if self.len >= self.cap {
            let new_cap = if self.cap == 0 { 8 } else { self.cap * 2 };
            let key_size = core::mem::size_of::<usize>();
            let val_size = core::mem::size_of::<ViewerPane>();
            let new_keys = unsafe { alloc_alloc_impl(alloc, key_size * new_cap) };
            let new_values = unsafe { alloc_alloc_impl(alloc, val_size * new_cap) };
            if new_keys.is_null() || new_values.is_null() {
                if !new_keys.is_null() {
                    unsafe { alloc_free_impl(alloc, new_keys, key_size * new_cap); }
                }
                if !new_values.is_null() {
                    unsafe { alloc_free_impl(alloc, new_values, val_size * new_cap); }
                }
                return false;
            }
            unsafe {
                ptr::write_bytes(new_keys, 0, key_size * new_cap);
                ptr::write_bytes(new_values, 0, val_size * new_cap);
            }
            if !self.keys.is_null() && self.len > 0 {
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.keys,
                        new_keys as *mut usize,
                        self.len,
                    );
                    ptr::copy_nonoverlapping(
                        self.values,
                        new_values as *mut ViewerPane,
                        self.len,
                    );
                    alloc_free_impl(alloc, self.keys as *mut u8, key_size * self.cap);
                    alloc_free_impl(alloc, self.values as *mut u8, val_size * self.cap);
                }
            }
            self.keys = new_keys as *mut usize;
            self.values = new_values as *mut ViewerPane;
            self.cap = new_cap;
        }
        unsafe {
            *self.keys.add(self.len) = key;
            *self.values.add(self.len) = value;
        }
        self.len += 1;
        true
    }

    fn remove(&mut self, key: usize) -> bool {
        let mut i: usize = 0;
        while i < self.len {
            if unsafe { *self.keys.add(i) } == key {
                if i + 1 < self.len {
                    unsafe {
                        *self.keys.add(i) = *self.keys.add(self.len - 1);
                        ptr::write(self.values.add(i), ptr::read(self.values.add(self.len - 1)));
                    }
                }
                self.len -= 1;
                return true;
            }
            i += 1;
        }
        false
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.keys.is_null() && self.cap > 0 {
            let key_size = core::mem::size_of::<usize>();
            unsafe { alloc_free_impl(alloc, self.keys as *mut u8, key_size * self.cap); }
        }
        if !self.values.is_null() && self.cap > 0 {
            let val_size = core::mem::size_of::<ViewerPane>();
            unsafe { alloc_free_impl(alloc, self.values as *mut u8, val_size * self.cap); }
        }
        self.keys = ptr::null_mut();
        self.values = ptr::null_mut();
        self.len = 0;
        self.cap = 0;
    }
}

#[repr(C)]
pub struct Viewer {
    pub alloc: *const GhosttyAllocator,
    pub state: ViewerState,
    pub session_id: usize,
    pub version_ptr: *const u8,
    pub version_len: usize,
    command_queue: CommandQueue,
    windows: WindowList,
    panes: PaneMap,
    action_single: [Action; 1],
    actions_ptr: *mut Action,
    actions_len: usize,
    actions_cap: usize,
}

impl Viewer {
    pub fn init(alloc: *const GhosttyAllocator) -> Viewer {
        Viewer {
            alloc,
            state: ViewerState::StartupBlock,
            session_id: 0,
            version_ptr: ptr::null(),
            version_len: 0,
            command_queue: CommandQueue::new(alloc, COMMAND_QUEUE_INITIAL),
            windows: WindowList::new(),
            panes: PaneMap::new(),
            action_single: [Action::empty()],
            actions_ptr: ptr::null_mut(),
            actions_len: 0,
            actions_cap: 0,
        }
    }

    pub fn deinit(&mut self) {
        self.command_queue.deinit(self.alloc);
        self.windows.deinit(self.alloc);
        self.deinit_panes();
        self.panes.deinit(self.alloc);
        if !self.version_ptr.is_null() && self.version_len > 0 {
            unsafe {
                alloc_free_impl(
                    self.alloc,
                    self.version_ptr as *mut u8,
                    self.version_len,
                );
            }
        }
        if !self.actions_ptr.is_null() && self.actions_cap > 0 {
            let total = core::mem::size_of::<Action>() * self.actions_cap;
            unsafe { alloc_free_impl(self.alloc, self.actions_ptr as *mut u8, total); }
        }
    }

    fn single_action(&mut self, action: Action) -> *const Action {
        unsafe { *self.action_single.get_unchecked_mut(0) = action; }
        self.action_single.as_ptr()
    }

    fn defunct(&mut self) -> (*const Action, usize) {
        self.state = ViewerState::Defunct;
        (self.single_action(Action::exit_action()), 1)
    }

    fn reset_actions(&mut self) {
        self.actions_len = 0;
    }

    fn push_action(&mut self, action: Action) -> bool {
        if self.actions_len >= self.actions_cap {
            let new_cap = if self.actions_cap == 0 { 4 } else { self.actions_cap * 2 };
            let action_size = core::mem::size_of::<Action>();
            let total = action_size * new_cap;
            let new_raw = unsafe { alloc_alloc_impl(self.alloc, total) };
            if new_raw.is_null() {
                return false;
            }
            unsafe { ptr::write_bytes(new_raw, 0, total); }
            if !self.actions_ptr.is_null() && self.actions_len > 0 {
                let old_total = action_size * self.actions_len;
                unsafe {
                    ptr::copy_nonoverlapping(
                        self.actions_ptr as *const u8,
                        new_raw,
                        old_total,
                    );
                    alloc_free_impl(
                        self.alloc,
                        self.actions_ptr as *mut u8,
                        action_size * self.actions_cap,
                    );
                }
            }
            self.actions_ptr = new_raw as *mut Action;
            self.actions_cap = new_cap;
        }
        unsafe { *self.actions_ptr.add(self.actions_len) = action; }
        self.actions_len += 1;
        true
    }

    fn actions_slice(&self) -> (*const Action, usize) {
        if self.actions_ptr.is_null() || self.actions_len == 0 {
            return (ptr::null(), 0);
        }
        (self.actions_ptr, self.actions_len)
    }

    pub fn next(
        &mut self,
        notification: *const Notification,
    ) -> (*const Action, usize) {
        if notification.is_null() {
            return (ptr::null(), 0);
        }
        let n = unsafe { &*notification };
        match self.state {
            ViewerState::Defunct => (ptr::null(), 0),
            ViewerState::StartupBlock => self.next_startup_block(n),
            ViewerState::StartupSession => self.next_startup_session(n),
            ViewerState::CommandQueue => self.next_command(n),
        }
    }

    fn next_startup_block(&mut self, n: &Notification) -> (*const Action, usize) {
        match n.tag {
            NotificationTag::Exit => return self.defunct(),
            NotificationTag::BlockEnd | NotificationTag::BlockErr => {
                self.state = ViewerState::StartupSession;
                return (ptr::null(), 0);
            }
            _ => return (ptr::null(), 0),
        }
    }

    fn next_startup_session(&mut self, n: &Notification) -> (*const Action, usize) {
        match n.tag {
            NotificationTag::Exit => return self.defunct(),
            NotificationTag::SessionChanged => {
                self.session_id = n.id;
                let result = self.enter_command_queue_with_commands(
                    &[Command::tmux_version(), Command::list_windows()],
                );
                if !result {
                    return self.defunct();
                }
                return self.actions_slice();
            }
            _ => return (ptr::null(), 0),
        }
    }

    fn next_command(&mut self, n: &Notification) -> (*const Action, usize) {
        self.reset_actions();

        let command_consumed_initial = self.command_queue.is_empty();
        let mut command_consumed = command_consumed_initial;

        match n.tag {
            NotificationTag::Exit => return self.defunct(),

            NotificationTag::BlockEnd | NotificationTag::BlockErr => {
                let is_err = matches!(n.tag, NotificationTag::BlockErr);
                let content_ptr = n.data_ptr;
                let content_len = n.data_len;
                self.received_command_output(content_ptr, content_len, is_err);
                command_consumed = true;
            }

            NotificationTag::Output => {
                self.received_output(n.id, n.data_ptr, n.data_len);
            }

            NotificationTag::SessionChanged => {
                if !self.session_changed(n.id) {
                    return self.defunct();
                }
                command_consumed = true;
            }

            NotificationTag::LayoutChange => {
                if !self.layout_changed(n.window_id, n.layout_ptr, n.layout_len) {
                    return self.defunct();
                }
            }

            NotificationTag::WindowAdd => {
                if !self.window_add(n.id) {
                    return self.defunct();
                }
            }

            NotificationTag::WindowPaneChanged
            | NotificationTag::SessionsChanged
            | NotificationTag::WindowRenamed
            | NotificationTag::ClientDetached
            | NotificationTag::ClientSessionChanged => {}

            _ => {}
        }

        if matches!(self.state, ViewerState::CommandQueue) && command_consumed {
            if let Some(cmd_ptr) = self.command_queue.first() {
                let cmd = unsafe { &*cmd_ptr };
                let mut buf: [u8; 512] = [0; 512];
                let mut written: usize = 0;
                format_command_string(cmd, &mut buf, &mut written);
                if written > 0 {
                    let cmd_copy = self.alloc_command_string(&buf[..written]);
                    if !cmd_copy.is_null() {
                        let action = Action {
                            tag: ActionTag::Command,
                            command_ptr: cmd_copy,
                            command_len: written,
                            windows_ptr: ptr::null(),
                            windows_len: 0,
                        };
                        let _ = self.push_action(action);
                    }
                }
            }
        }

        self.actions_slice()
    }

    fn alloc_command_string(&self, data: &[u8]) -> *const u8 {
        if data.is_empty() {
            return ptr::null();
        }
        let raw = unsafe { alloc_alloc_impl(self.alloc, data.len()) };
        if raw.is_null() {
            return ptr::null();
        }
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), raw, data.len());
        }
        raw as *const u8
    }

    fn received_command_output(
        &mut self,
        content_ptr: *const u8,
        content_len: usize,
        _is_err: bool,
    ) {
        let cmd: Command = if let Some(cmd_ptr) = self.command_queue.first() {
            unsafe { ptr::read(cmd_ptr) }
        } else {
            return;
        };
        self.command_queue.delete_oldest(1);

        let content = if content_ptr.is_null() || content_len == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(content_ptr, content_len) }
        };

        match cmd.tag {
            CommandTag::TmuxVersion => {
                self.received_tmux_version(content);
            }
            CommandTag::ListWindows => {
                self.received_list_windows(content);
            }
            CommandTag::PaneState => {
                self.received_pane_state(content);
            }
            CommandTag::PaneHistory | CommandTag::PaneVisible => {
                self.received_pane_capture(
                    cmd.pane_id,
                    cmd.screen_is_alternate,
                    matches!(cmd.tag, CommandTag::PaneHistory),
                    content,
                );
            }
            CommandTag::User => {}
        }
    }

    fn received_tmux_version(&mut self, content: &[u8]) {
        let trimmed = trim_ws(content);
        if trimmed.is_empty() {
            return;
        }
        if !self.version_ptr.is_null() && self.version_len > 0 {
            unsafe {
                alloc_free_impl(
                    self.alloc,
                    self.version_ptr as *mut u8,
                    self.version_len,
                );
            }
        }
        let copy = unsafe { alloc_alloc_impl(self.alloc, trimmed.len()) };
        if copy.is_null() {
            return;
        }
        unsafe { ptr::copy_nonoverlapping(trimmed.as_ptr(), copy, trimmed.len()); }
        self.version_ptr = copy as *const u8;
        self.version_len = trimmed.len();
    }

    fn received_list_windows(&mut self, content: &[u8]) {
        let mut new_windows = WindowList::new();
        let mut line_start: usize = 0;
        let mut i: usize = 0;

        while i <= content.len() {
            let is_end = i == content.len();
            let is_newline = !is_end && unsafe { *content.get_unchecked(i) } == b'\n';

            if is_newline || is_end {
                if i > line_start {
                    let line_raw = unsafe { content.get_unchecked(line_start..i) };
                    let line = trim_ws(line_raw);
                    if !line.is_empty() {
                        let mut info = ParsedWindowInfo {
                            session_id: 0,
                            window_id: 0,
                            window_width: 0,
                            window_height: 0,
                            window_layout_ptr: ptr::null(),
                            window_layout_len: 0,
                        };
                        let result = parse_window_info_line(line, &mut info);
                        if result == 0 {
                            let layout_slice = if !info.window_layout_ptr.is_null()
                                && info.window_layout_len > 0
                            {
                                unsafe {
                                    core::slice::from_raw_parts(
                                        info.window_layout_ptr,
                                        info.window_layout_len,
                                    )
                                }
                            } else {
                                &[]
                            };

                            let mut layout = Layout {
                                width: 0,
                                height: 0,
                                x: 0,
                                y: 0,
                                content: LayoutContent::Pane(0),
                            };
                            let _ = layout_parse_with_checksum(
                                self.alloc,
                                layout_slice,
                                &mut layout,
                            );

                            let window = ViewerWindow {
                                id: info.window_id,
                                width: info.window_width,
                                height: info.window_height,
                                layout,
                            };
                            let _ = new_windows.push(window, self.alloc);
                        }
                    }
                }
                line_start = i + 1;
            }
            i += 1;
        }

        let windows_action = Action {
            tag: ActionTag::Windows,
            command_ptr: ptr::null(),
            command_len: 0,
            windows_ptr: new_windows.ptr,
            windows_len: new_windows.len,
        };
        let _ = self.push_action(windows_action);

        self.sync_layouts(new_windows.ptr, new_windows.len);

        // Transfer ownership of windows
        let old_ptr = self.windows.ptr;
        let old_cap = self.windows.cap;
        self.windows.ptr = new_windows.ptr;
        self.windows.len = new_windows.len;
        self.windows.cap = new_windows.cap;
        if !old_ptr.is_null() && old_cap > 0 {
            let total = core::mem::size_of::<ViewerWindow>() * old_cap;
            unsafe { alloc_free_impl(self.alloc, old_ptr as *mut u8, total); }
        }
    }

    fn received_pane_state(&mut self, content: &[u8]) {
        let mut line_start: usize = 0;
        let mut i: usize = 0;

        while i <= content.len() {
            let is_end = i == content.len();
            let is_newline = !is_end && unsafe { *content.get_unchecked(i) } == b'\n';

            if is_newline || is_end {
                if i > line_start {
                    let line_raw = unsafe { content.get_unchecked(line_start..i) };
                    let line = trim_ws(line_raw);
                    if !line.is_empty() {
                        let mut state = ParsedPaneState {
                            pane_id: 0,
                            cursor_x: 0,
                            cursor_y: 0,
                            cursor_flag: false,
                            cursor_shape_ptr: ptr::null(),
                            cursor_shape_len: 0,
                            cursor_colour_ptr: ptr::null(),
                            cursor_colour_len: 0,
                            cursor_blinking: false,
                            alternate_on: false,
                            alternate_saved_x: 0,
                            alternate_saved_y: 0,
                            insert_flag: false,
                            wrap_flag: false,
                            keypad_flag: false,
                            keypad_cursor_flag: false,
                            origin_flag: false,
                            mouse_all_flag: false,
                            mouse_any_flag: false,
                            mouse_button_flag: false,
                            mouse_standard_flag: false,
                            mouse_utf8_flag: false,
                            mouse_sgr_flag: false,
                            focus_flag: false,
                            bracketed_paste: false,
                            scroll_region_upper: 0,
                            scroll_region_lower: 0,
                            pane_tabs_ptr: ptr::null(),
                            pane_tabs_len: 0,
                        };
                        let _ = parse_pane_state_line(line, &mut state);
                        self.apply_pane_state(&state);
                    }
                }
                line_start = i + 1;
            }
            i += 1;
        }
    }

    fn pane_terminal_mut(&mut self, pane_id: usize) -> Option<&mut Terminal> {
        let pane = self.panes.get(pane_id)?;
        let terminal = unsafe { (*pane).terminal };
        if terminal.is_null() {
            return None;
        }
        Some(unsafe { &mut *(terminal as *mut Terminal) })
    }

    fn switch_terminal_screen(t: &mut Terminal, screen_key: ScreenKey) -> bool {
        if t.screens.get(screen_key).is_null() {
            return false;
        }
        t.screens.switch_to(screen_key);
        true
    }

    fn received_output(
        &mut self,
        pane_id: usize,
        data_ptr: *const u8,
        data_len: usize,
    ) {
        if data_ptr.is_null() || data_len == 0 {
            return;
        }
        let Some(t) = self.pane_terminal_mut(pane_id) else {
            return;
        };
        let data = unsafe { core::slice::from_raw_parts(data_ptr, data_len) };
        t.write(data);
    }

    fn received_pane_capture(
        &mut self,
        pane_id: usize,
        screen_is_alternate: bool,
        is_history: bool,
        content: &[u8],
    ) {
        let Some(t) = self.pane_terminal_mut(pane_id) else {
            return;
        };
        let screen_key = if screen_is_alternate {
            ScreenKey::Alternate
        } else {
            ScreenKey::Primary
        };
        if !Self::switch_terminal_screen(t, screen_key) {
            return;
        }

        if is_history {
            t.write(content);
            t.carriage_return();
            let mut row = 0;
            while row < t.rows {
                t.index();
                row += 1;
            }
            t.cursor_set_cell(1, 1);
        } else {
            t.erase_display(EraseDisplay::Complete, false);
            t.cursor_set_cell(1, 1);
            t.write(content);
        }
    }

    fn apply_pane_state(&mut self, state: &ParsedPaneState) {
        let Some(t) = self.pane_terminal_mut(state.pane_id) else {
            return;
        };

        let screen_key = if state.alternate_on {
            ScreenKey::Alternate
        } else {
            ScreenKey::Primary
        };
        if Self::switch_terminal_screen(t, screen_key) {
            let cursor_x = state.cursor_x as usize + 1;
            let cursor_y = state.cursor_y as usize + 1;
            if cursor_x <= t.cols as usize && cursor_y <= t.rows as usize {
                t.cursor_set_cell(cursor_y, cursor_x);
            }

            let screen = t.active();
            if !screen.is_null() && !state.cursor_shape_ptr.is_null() && state.cursor_shape_len > 0 {
                let shape = unsafe {
                    core::slice::from_raw_parts(
                        state.cursor_shape_ptr,
                        state.cursor_shape_len,
                    )
                };
                unsafe {
                    if bytes_eq(shape, b"block") {
                        (*screen).cursor.cursor_style = CursorVisualStyle::Block;
                    } else if bytes_eq(shape, b"underline") {
                        (*screen).cursor.cursor_style = CursorVisualStyle::Underline;
                    } else if bytes_eq(shape, b"bar") {
                        (*screen).cursor.cursor_style = CursorVisualStyle::Bar;
                    }
                }
            }
        }

        if Self::switch_terminal_screen(t, ScreenKey::Alternate) {
            let alt_x = state.alternate_saved_x as usize + 1;
            let alt_y = state.alternate_saved_y as usize + 1;
            if alt_x <= t.cols as usize && alt_y <= t.rows as usize {
                t.cursor_set_cell(alt_y, alt_x);
            }
        }
        let _ = Self::switch_terminal_screen(t, screen_key);

        t.mode_set(ModeTag { value: 25, ansi: false }, state.cursor_flag);
        t.mode_set(ModeTag { value: 12, ansi: false }, state.cursor_blinking);
        t.mode_set(ModeTag { value: 4, ansi: true }, state.insert_flag);
        t.mode_set(ModeTag { value: 7, ansi: false }, state.wrap_flag);
        t.mode_set(ModeTag { value: 66, ansi: false }, state.keypad_flag);
        t.mode_set(ModeTag { value: 1, ansi: false }, state.keypad_cursor_flag);
        t.mode_set(ModeTag { value: 6, ansi: false }, state.origin_flag);
        t.mode_set(ModeTag { value: 1003, ansi: false }, state.mouse_all_flag);
        t.mode_set(ModeTag { value: 1002, ansi: false }, state.mouse_any_flag);
        t.mode_set(ModeTag { value: 1000, ansi: false }, state.mouse_button_flag);
        t.mode_set(ModeTag { value: 9, ansi: false }, state.mouse_standard_flag);
        t.mode_set(ModeTag { value: 1005, ansi: false }, state.mouse_utf8_flag);
        t.mode_set(ModeTag { value: 1006, ansi: false }, state.mouse_sgr_flag);
        t.mode_set(ModeTag { value: 1004, ansi: false }, state.focus_flag);
        t.mode_set(ModeTag { value: 2004, ansi: false }, state.bracketed_paste);

        t.scrolling_region.top = state.scroll_region_upper as CellCountInt;
        t.scrolling_region.bottom = state.scroll_region_lower as CellCountInt;

        t.tabstops.reset(0);
        if !state.pane_tabs_ptr.is_null() && state.pane_tabs_len > 0 {
            let tabs = unsafe {
                core::slice::from_raw_parts(state.pane_tabs_ptr, state.pane_tabs_len)
            };
            for_each_usize_field(tabs, b',', |col| {
                if col < t.cols as usize {
                    t.tabstops.set(col);
                }
            });
        } else {
            t.tabstops.reset(TABSTOP_INTERVAL as usize);
        }
    }

    fn sync_layouts(&mut self, windows_ptr: *const ViewerWindow, windows_len: usize) {
        let mut new_panes = PaneMap::new();

        let mut wi: usize = 0;
        while wi < windows_len {
            let window = unsafe { &*windows_ptr.add(wi) };
            self.collect_panes_from_layout(&window.layout, &mut new_panes);
            wi += 1;
        }

        let mut added_any = false;
        let mut i: usize = 0;
        while i < new_panes.len {
            let pane_id = unsafe { *new_panes.keys.add(i) };
            if !self.panes.contains(pane_id) {
                added_any = true;
                let _ = self.command_queue.push(
                    Command::pane_history(pane_id, false),
                    self.alloc,
                );
                let _ = self.command_queue.push(
                    Command::pane_visible(pane_id, false),
                    self.alloc,
                );
                let _ = self.command_queue.push(
                    Command::pane_history(pane_id, true),
                    self.alloc,
                );
                let _ = self.command_queue.push(
                    Command::pane_visible(pane_id, true),
                    self.alloc,
                );
            }
            i += 1;
        }

        if added_any {
            let _ = self.command_queue.push(Command::pane_state(), self.alloc);
        }

        let mut i: usize = 0;
        while i < self.panes.len {
            let pane_id = unsafe { *self.panes.keys.add(i) };
            if !new_panes.contains(pane_id) {
                if let Some(pane_ptr) = self.panes.get(pane_id) {
                    let pane = unsafe { ptr::read(pane_ptr) };
                    Self::deinit_pane(self.alloc, pane);
                }
                let _ = self.panes.remove(pane_id);
            } else {
                i += 1;
            }
        }
        let mut j: usize = 0;
        while j < new_panes.len {
            let pane_id = unsafe { *new_panes.keys.add(j) };
            let pane = unsafe { ptr::read(new_panes.values.add(j)) };
            let _ = self.panes.insert(pane_id, pane, self.alloc);
            j += 1;
        }
        new_panes.deinit(self.alloc);
    }

    fn alloc_pane_terminal(
        alloc: *const GhosttyAllocator,
        layout: &Layout,
    ) -> *mut c_void {
        let cols = layout.width as CellCountInt;
        let rows = layout.height as CellCountInt;
        if cols == 0 || rows == 0 {
            return ptr::null_mut();
        }
        #[cfg(ghostty_vt_terminal_owned)]
        unsafe {
            let Some(term) = Terminal::init_full(alloc, cols, rows, 10_000) else {
                return ptr::null_mut();
            };
            let size = core::mem::size_of::<Terminal>();
            let raw = alloc_alloc_impl(alloc, size);
            if raw.is_null() {
                return ptr::null_mut();
            }
            ptr::write(raw as *mut Terminal, term);
            raw as *mut c_void
        }
        #[cfg(not(ghostty_vt_terminal_owned))]
        {
            let _ = (alloc, cols, rows);
            ptr::null_mut()
        }
    }

    fn deinit_pane(alloc: *const GhosttyAllocator, pane: ViewerPane) {
        if pane.terminal.is_null() {
            return;
        }
        unsafe {
            let term = pane.terminal as *mut Terminal;
            let mut term_val = ptr::read(term);
            #[cfg(ghostty_vt_terminal_owned)]
            term_val.deinit_full(alloc);
            #[cfg(not(ghostty_vt_terminal_owned))]
            term_val.deinit(alloc);
            alloc_free_impl(alloc, term as *mut u8, core::mem::size_of::<Terminal>());
        }
    }

    fn deinit_panes(&mut self) {
        while self.panes.len > 0 {
            let pane_id = unsafe { *self.panes.keys.add(0) };
            if let Some(pane_ptr) = self.panes.get(pane_id) {
                let pane = unsafe { ptr::read(pane_ptr) };
                Self::deinit_pane(self.alloc, pane);
            }
            let _ = self.panes.remove(pane_id);
        }
    }

    fn collect_panes_from_layout(
        &self,
        layout: *const Layout,
        panes: *mut PaneMap,
    ) {
        if layout.is_null() {
            return;
        }
        unsafe {
            match (*layout).content {
                LayoutContent::Pane(id) => {
                    if !(*panes).contains(id) {
                        let pane = if let Some(existing) = self.panes.get(id) {
                            ptr::read(existing)
                        } else {
                            ViewerPane {
                                terminal: Self::alloc_pane_terminal(self.alloc, &*layout),
                            }
                        };
                        let _ = (*panes).insert(id, pane, self.alloc);
                    }
                }
                LayoutContent::Horizontal { ptr, len } |
                LayoutContent::Vertical { ptr, len } => {
                    let mut i: usize = 0;
                    while i < len {
                        let child = ptr.add(i);
                        self.collect_panes_from_layout(child, panes);
                        i += 1;
                    }
                }
            }
        }
    }

    fn layout_changed(
        &mut self,
        window_id: usize,
        layout_ptr: *const u8,
        layout_len: usize,
    ) -> bool {
        let mut found = false;
        let mut wi: usize = 0;
        while wi < self.windows.len {
            let w = unsafe { &mut *self.windows.ptr.add(wi) };
            if w.id == window_id {
                found = true;
                if !layout_ptr.is_null() && layout_len > 0 {
                    let layout_slice = unsafe {
                        core::slice::from_raw_parts(layout_ptr, layout_len)
                    };
                    let mut new_layout = Layout {
                        width: 0,
                        height: 0,
                        x: 0,
                        y: 0,
                        content: LayoutContent::Pane(0),
                    };
                    let _ = layout_parse_with_checksum(
                        self.alloc,
                        layout_slice,
                        &mut new_layout,
                    );
                    w.layout = new_layout;
                }
                break;
            }
            wi += 1;
        }

        if !found {
            return true;
        }

        let windows_action = Action {
            tag: ActionTag::Windows,
            command_ptr: ptr::null(),
            command_len: 0,
            windows_ptr: self.windows.ptr,
            windows_len: self.windows.len,
        };
        let _ = self.push_action(windows_action);

        self.sync_layouts(self.windows.ptr, self.windows.len);
        true
    }

    fn window_add(&mut self, _window_id: usize) -> bool {
        self.command_queue.push(Command::list_windows(), self.alloc)
    }

    fn session_changed(&mut self, session_id: usize) -> bool {
        let empty_action = Action {
            tag: ActionTag::Windows,
            command_ptr: ptr::null(),
            command_len: 0,
            windows_ptr: ptr::null(),
            windows_len: 0,
        };
        let _ = self.push_action(empty_action);

        self.windows.clear();
        self.deinit_panes();
        self.command_queue.deinit(self.alloc);
        self.command_queue = CommandQueue::new(self.alloc, COMMAND_QUEUE_INITIAL);

        self.session_id = session_id;
        self.state = ViewerState::CommandQueue;

        let _ = self.command_queue.push(Command::list_windows(), self.alloc);
        true
    }

    fn enter_command_queue_with_commands(&mut self, commands: &[Command]) -> bool {
        if commands.is_empty() {
            return false;
        }

        self.reset_actions();

        let first_cmd = unsafe { commands.get_unchecked(0) };
        let mut buf: [u8; 512] = [0; 512];
        let mut written: usize = 0;
        format_command_string(first_cmd, &mut buf, &mut written);

        if written > 0 {
            let cmd_copy = self.alloc_command_string(&buf[..written]);
            if !cmd_copy.is_null() {
                let action = Action {
                    tag: ActionTag::Command,
                    command_ptr: cmd_copy,
                    command_len: written,
                    windows_ptr: ptr::null(),
                    windows_len: 0,
                };
                let _ = self.push_action(action);
            }
        }

        let mut i: usize = 0;
        while i < commands.len() {
            let cmd = unsafe { ptr::read(commands.as_ptr().add(i)) };
            if !self.command_queue.push(cmd, self.alloc) {
                return false;
            }
            i += 1;
        }

        self.state = ViewerState::CommandQueue;
        true
    }

    /// Test and integration helpers (std builds only).
    #[cfg(feature = "std")]
    pub fn window_count(&self) -> usize {
        self.windows.len
    }

    #[cfg(feature = "std")]
    pub fn pane_count(&self) -> usize {
        self.panes.count()
    }

    #[cfg(feature = "std")]
    pub fn command_queue_len(&self) -> usize {
        self.command_queue.queue_len()
    }

    #[cfg(feature = "std")]
    pub fn tmux_version_bytes(&self) -> &[u8] {
        if self.version_ptr.is_null() || self.version_len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.version_ptr, self.version_len) }
    }

    #[cfg(feature = "std")]
    pub fn pane_has_terminal(&self, pane_id: usize) -> bool {
        self.panes
            .get(pane_id)
            .map(|p| unsafe { !(*p).terminal.is_null() })
            .unwrap_or(false)
    }
}

fn format_command_string(
    cmd: &Command,
    buf: &mut [u8; 512],
    written: *mut usize,
) {
    let mut pos: usize = 0;

    macro_rules! write_bytes {
        ($bytes:expr) => {{
            let b = $bytes;
            let mut bi: usize = 0;
            while bi < b.len() && pos < 512 {
                unsafe { *buf.get_unchecked_mut(pos) = *b.get_unchecked(bi); }
                pos += 1;
                bi += 1;
            }
        }};
    }

    macro_rules! write_usize {
        ($val:expr) => {{
            let mut v = $val;
            let mut digits: [u8; 20] = [0; 20];
            let mut dcount: usize = 0;
            if v == 0 {
                unsafe { *digits.get_unchecked_mut(0) = b'0'; }
                dcount = 1;
            } else {
                while v > 0 {
                    let d = (v % 10) as u8;
                    unsafe { *digits.get_unchecked_mut(dcount) = b'0' + d; }
                    dcount += 1;
                    v /= 10;
                }
            }
            let mut di = dcount;
            while di > 0 && pos < 512 {
                di -= 1;
                unsafe { *buf.get_unchecked_mut(pos) = *digits.get_unchecked(di); }
                pos += 1;
            }
        }};
    }

    match cmd.tag {
        CommandTag::ListWindows => {
            write_bytes!(b"list-windows -F '#{session_id} #{window_id} #{window_width} #{window_height} #{window_layout}'\n");
        }
        CommandTag::PaneHistory => {
            write_bytes!(b"capture-pane -p -e -q ");
            if cmd.screen_is_alternate {
                write_bytes!(b"-a ");
            }
            write_bytes!(b"-S - -E -1 -t %");
            write_usize!(cmd.pane_id);
            write_bytes!(b"\n");
        }
        CommandTag::PaneVisible => {
            write_bytes!(b"capture-pane -p -e -q ");
            if cmd.screen_is_alternate {
                write_bytes!(b"-a ");
            }
            write_bytes!(b"-t %");
            write_usize!(cmd.pane_id);
            write_bytes!(b"\n");
        }
        CommandTag::PaneState => {
            write_bytes!(b"list-panes -F '#{pane_id};#{cursor_x};#{cursor_y};#{cursor_flag};#{cursor_shape};#{cursor_colour};#{cursor_blinking};#{alternate_on};#{alternate_saved_x};#{alternate_saved_y};#{insert_flag};#{wrap_flag};#{keypad_flag};#{keypad_cursor_flag};#{origin_flag};#{mouse_all_flag};#{mouse_any_flag};#{mouse_button_flag};#{mouse_standard_flag};#{mouse_utf8_flag};#{mouse_sgr_flag};#{focus_flag};#{bracketed_paste};#{scroll_region_upper};#{scroll_region_lower};#{pane_tabs};#{version}'\n");
        }
        CommandTag::TmuxVersion => {
            write_bytes!(b"display-message -p '#{version}'\n");
        }
        CommandTag::User => {
            if !cmd.user_cmd_ptr.is_null() && cmd.user_cmd_len > 0 {
                let mut ui: usize = 0;
                while ui < cmd.user_cmd_len && pos < 512 {
                    unsafe {
                        *buf.get_unchecked_mut(pos) = *cmd.user_cmd_ptr.add(ui);
                    }
                    pos += 1;
                    ui += 1;
                }
            }
        }
    }

    unsafe { *written = pos; }
}

fn trim_ws(s: &[u8]) -> &[u8] {
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

fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0usize;
    while i < a.len() {
        if unsafe { *a.get_unchecked(i) } != unsafe { *b.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

fn for_each_usize_field<F: FnMut(usize)>(s: &[u8], delim: u8, mut f: F) {
    let mut start = 0usize;
    let mut i = 0usize;
    while i <= s.len() {
        if i == s.len() || unsafe { *s.get_unchecked(i) } == delim {
            if i > start {
                if let Some(value) = parse_usize(unsafe { s.get_unchecked(start..i) }) {
                    f(value);
                }
            }
            start = i + 1;
        }
        i += 1;
    }
}

fn parse_usize(s: &[u8]) -> Option<usize> {
    if s.is_empty() {
        return None;
    }
    let mut result = 0usize;
    let mut i = 0usize;
    while i < s.len() {
        let c = unsafe { *s.get_unchecked(i) };
        if c < b'0' || c > b'9' {
            return None;
        }
        result = result.checked_mul(10)?;
        result = result.checked_add((c - b'0') as usize)?;
        i += 1;
    }
    Some(result)
}

#[cfg(feature = "tmux-tests")]
impl Viewer {
    pub fn test_tmux_version_bytes(&self) -> &[u8] {
        if self.version_ptr.is_null() || self.version_len == 0 {
            return b"";
        }
        unsafe { core::slice::from_raw_parts(self.version_ptr, self.version_len) }
    }

    pub fn test_window_count(&self) -> usize {
        self.windows.len
    }

    pub fn test_pane_count(&self) -> usize {
        self.panes.count()
    }

    pub fn test_pane_terminal(&self, pane_id: usize) -> Option<*mut Terminal> {
        let pane = self.panes.get(pane_id)?;
        let terminal = unsafe { (*pane).terminal };
        if terminal.is_null() {
            None
        } else {
            Some(terminal as *mut Terminal)
        }
    }
}
