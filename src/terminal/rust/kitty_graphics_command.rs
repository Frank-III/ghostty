use core::ptr;

pub(crate) const COMMAND_MAX_KV_ENTRIES: usize = 64;
pub(crate) const COMMAND_MAX_DATA_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const KV_TEMP_LEN: usize = 11;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CommandAction {
    Query = 0,
    Transmit = 1,
    TransmitAndDisplay = 2,
    Display = 3,
    Delete = 4,
    TransmitAnimationFrame = 5,
    ControlAnimation = 6,
    ComposeAnimation = 7,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CommandQuiet {
    No = 0,
    Ok = 1,
    Failures = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TransmissionFormat {
    Rgb = 0,
    Rgba = 1,
    Png = 2,
    GrayAlpha = 3,
    Gray = 4,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TransmissionMedium {
    Direct = 0,
    File = 1,
    TemporaryFile = 2,
    SharedMemory = 3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TransmissionCompression {
    None = 0,
    ZlibDeflate = 1,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CursorMovement {
    After = 0,
    None = 1,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum CompositionMode {
    AlphaBlend = 0,
    Overwrite = 1,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum AnimationAction {
    Invalid = 0,
    Stop = 1,
    RunWait = 2,
    Run = 3,
}

pub(crate) fn format_bpp(format: TransmissionFormat) -> u8 {
    match format {
        TransmissionFormat::Gray => 1,
        TransmissionFormat::GrayAlpha => 2,
        TransmissionFormat::Rgb => 3,
        TransmissionFormat::Rgba => 4,
        TransmissionFormat::Png => 0,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Transmission {
    pub format: TransmissionFormat,
    pub medium: TransmissionMedium,
    pub width: u32,
    pub height: u32,
    pub size: u32,
    pub offset: u32,
    pub image_id: u32,
    pub image_number: u32,
    pub placement_id: u32,
    pub compression: TransmissionCompression,
    pub more_chunks: bool,
}

impl Transmission {
    pub(crate) fn new() -> Self {
        Self {
            format: TransmissionFormat::Rgba,
            medium: TransmissionMedium::Direct,
            width: 0,
            height: 0,
            size: 0,
            offset: 0,
            image_id: 0,
            image_number: 0,
            placement_id: 0,
            compression: TransmissionCompression::None,
            more_chunks: false,
        }
    }

    pub(crate) fn parse(kv: &KvMap) -> Option<Self> {
        let mut result = Self::new();

        if let Some(v) = kv.get(b'f') {
            result.format = match v {
                24 => TransmissionFormat::Rgb,
                32 => TransmissionFormat::Rgba,
                100 => TransmissionFormat::Png,
                _ => return None,
            };
        }

        if let Some(v) = kv.get(b't') {
            let c = v as u8;
            result.medium = match c {
                b'd' => TransmissionMedium::Direct,
                b'f' => TransmissionMedium::File,
                b't' => TransmissionMedium::TemporaryFile,
                b's' => TransmissionMedium::SharedMemory,
                _ => return None,
            };
        }

        if let Some(v) = kv.get(b's') { result.width = v; }
        if let Some(v) = kv.get(b'v') { result.height = v; }
        if let Some(v) = kv.get(b'S') { result.size = v; }
        if let Some(v) = kv.get(b'O') { result.offset = v; }
        if let Some(v) = kv.get(b'i') { result.image_id = v; }
        if let Some(v) = kv.get(b'I') { result.image_number = v; }
        if let Some(v) = kv.get(b'p') { result.placement_id = v; }

        if let Some(v) = kv.get(b'o') {
            let c = v as u8;
            result.compression = match c {
                b'z' => TransmissionCompression::ZlibDeflate,
                _ => return None,
            };
        }

        if result.medium == TransmissionMedium::Direct {
            if let Some(v) = kv.get(b'm') {
                result.more_chunks = v > 0;
            }
        }

        Some(result)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Display {
    pub image_id: u32,
    pub image_number: u32,
    pub placement_id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub columns: u32,
    pub rows: u32,
    pub cursor_movement: CursorMovement,
    pub virtual_placement: bool,
    pub parent_id: u32,
    pub parent_placement_id: u32,
    pub horizontal_offset: i32,
    pub vertical_offset: i32,
    pub z: i32,
}

impl Display {
    pub(crate) fn new() -> Self {
        Self {
            image_id: 0,
            image_number: 0,
            placement_id: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            x_offset: 0,
            y_offset: 0,
            columns: 0,
            rows: 0,
            cursor_movement: CursorMovement::After,
            virtual_placement: false,
            parent_id: 0,
            parent_placement_id: 0,
            horizontal_offset: 0,
            vertical_offset: 0,
            z: 0,
        }
    }

    pub(crate) fn parse(kv: &KvMap) -> Option<Self> {
        let mut result = Self::new();

        if let Some(v) = kv.get(b'i') { result.image_id = v; }
        if let Some(v) = kv.get(b'I') { result.image_number = v; }
        if let Some(v) = kv.get(b'p') { result.placement_id = v; }
        if let Some(v) = kv.get(b'x') { result.x = v; }
        if let Some(v) = kv.get(b'y') { result.y = v; }
        if let Some(v) = kv.get(b'w') { result.width = v; }
        if let Some(v) = kv.get(b'h') { result.height = v; }
        if let Some(v) = kv.get(b'X') { result.x_offset = v; }
        if let Some(v) = kv.get(b'Y') { result.y_offset = v; }
        if let Some(v) = kv.get(b'c') { result.columns = v; }
        if let Some(v) = kv.get(b'r') { result.rows = v; }

        if let Some(v) = kv.get(b'C') {
            result.cursor_movement = match v {
                0 => CursorMovement::After,
                1 => CursorMovement::None,
                _ => return None,
            };
        }

        if let Some(v) = kv.get(b'U') {
            result.virtual_placement = match v {
                0 => false,
                1 => true,
                _ => return None,
            };
        }

        if let Some(v) = kv.get(b'z') { result.z = v as i32; }
        if let Some(v) = kv.get(b'P') { result.parent_id = v; }
        if let Some(v) = kv.get(b'Q') { result.parent_placement_id = v; }
        if let Some(v) = kv.get(b'H') { result.horizontal_offset = v as i32; }
        if let Some(v) = kv.get(b'V') { result.vertical_offset = v as i32; }

        Some(result)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AnimationBackground {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl AnimationBackground {
    pub(crate) fn new() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    pub(crate) fn from_u32(v: u32) -> Self {
        Self {
            r: v as u8,
            g: (v >> 8) as u8,
            b: (v >> 16) as u8,
            a: (v >> 24) as u8,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AnimationFrameLoading {
    pub x: u32,
    pub y: u32,
    pub create_frame: u32,
    pub edit_frame: u32,
    pub gap_ms: u32,
    pub composition_mode: CompositionMode,
    pub background: AnimationBackground,
}

impl AnimationFrameLoading {
    pub(crate) fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            create_frame: 0,
            edit_frame: 0,
            gap_ms: 0,
            composition_mode: CompositionMode::AlphaBlend,
            background: AnimationBackground::new(),
        }
    }

    pub(crate) fn parse(kv: &KvMap) -> Option<Self> {
        let mut result = Self::new();

        if let Some(v) = kv.get(b'x') { result.x = v; }
        if let Some(v) = kv.get(b'y') { result.y = v; }
        if let Some(v) = kv.get(b'c') { result.create_frame = v; }
        if let Some(v) = kv.get(b'r') { result.edit_frame = v; }
        if let Some(v) = kv.get(b'z') { result.gap_ms = v; }

        if let Some(v) = kv.get(b'X') {
            result.composition_mode = match v {
                0 => CompositionMode::AlphaBlend,
                1 => CompositionMode::Overwrite,
                _ => return None,
            };
        }

        if let Some(v) = kv.get(b'Y') {
            result.background = AnimationBackground::from_u32(v);
        }

        Some(result)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AnimationFrameComposition {
    pub frame: u32,
    pub edit_frame: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub left_edge: u32,
    pub top_edge: u32,
    pub composition_mode: CompositionMode,
}

impl AnimationFrameComposition {
    pub(crate) fn new() -> Self {
        Self {
            frame: 0,
            edit_frame: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            left_edge: 0,
            top_edge: 0,
            composition_mode: CompositionMode::AlphaBlend,
        }
    }

    pub(crate) fn parse(kv: &KvMap) -> Option<Self> {
        let mut result = Self::new();

        if let Some(v) = kv.get(b'c') { result.frame = v; }
        if let Some(v) = kv.get(b'r') { result.edit_frame = v; }
        if let Some(v) = kv.get(b'x') { result.x = v; }
        if let Some(v) = kv.get(b'y') { result.y = v; }
        if let Some(v) = kv.get(b'w') { result.width = v; }
        if let Some(v) = kv.get(b'h') { result.height = v; }
        if let Some(v) = kv.get(b'X') { result.left_edge = v; }
        if let Some(v) = kv.get(b'Y') { result.top_edge = v; }

        if let Some(v) = kv.get(b'C') {
            result.composition_mode = match v {
                0 => CompositionMode::AlphaBlend,
                1 => CompositionMode::Overwrite,
                _ => return None,
            };
        }

        Some(result)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AnimationControl {
    pub action: AnimationAction,
    pub frame: u32,
    pub gap_ms: u32,
    pub current_frame: u32,
    pub loops: u32,
}

impl AnimationControl {
    pub(crate) fn new() -> Self {
        Self {
            action: AnimationAction::Invalid,
            frame: 0,
            gap_ms: 0,
            current_frame: 0,
            loops: 0,
        }
    }

    pub(crate) fn parse(kv: &KvMap) -> Option<Self> {
        let mut result = Self::new();

        if let Some(v) = kv.get(b's') {
            result.action = match v {
                0 => AnimationAction::Invalid,
                1 => AnimationAction::Stop,
                2 => AnimationAction::RunWait,
                3 => AnimationAction::Run,
                _ => return None,
            };
        }

        if let Some(v) = kv.get(b'r') { result.frame = v; }
        if let Some(v) = kv.get(b'z') { result.gap_ms = v; }
        if let Some(v) = kv.get(b'c') { result.current_frame = v; }
        if let Some(v) = kv.get(b'v') { result.loops = v; }

        Some(result)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteById {
    pub delete: bool,
    pub image_id: u32,
    pub placement_id: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteByNewest {
    pub delete: bool,
    pub image_number: u32,
    pub placement_id: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteIntersectCell {
    pub delete: bool,
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteIntersectCellZ {
    pub delete: bool,
    pub x: u32,
    pub y: u32,
    pub z: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteRange {
    pub delete: bool,
    pub first: u32,
    pub last: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteColumn {
    pub delete: bool,
    pub x: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteRow {
    pub delete: bool,
    pub y: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct DeleteZ {
    pub delete: bool,
    pub z: i32,
}

#[derive(Clone, Copy)]
pub(crate) enum Delete {
    All(bool),
    Id(DeleteById),
    Newest(DeleteByNewest),
    IntersectCursor(bool),
    AnimationFrames(bool),
    IntersectCell(DeleteIntersectCell),
    IntersectCellZ(DeleteIntersectCellZ),
    Range(DeleteRange),
    Column(DeleteColumn),
    Row(DeleteRow),
    Z(DeleteZ),
}

impl Delete {
    pub(crate) fn parse(kv: &KvMap) -> Option<Self> {
        let what: u8 = match kv.get(b'd') {
            Some(v) => v as u8,
            None => b'a',
        };

        match what {
            b'a' | b'A' => Some(Delete::All(what == b'A')),

            b'i' | b'I' => {
                let mut result = DeleteById {
                    delete: what == b'I',
                    image_id: 0,
                    placement_id: 0,
                };
                if let Some(v) = kv.get(b'i') { result.image_id = v; }
                if let Some(v) = kv.get(b'p') { result.placement_id = v; }
                Some(Delete::Id(result))
            },

            b'n' | b'N' => {
                let mut result = DeleteByNewest {
                    delete: what == b'N',
                    image_number: 0,
                    placement_id: 0,
                };
                if let Some(v) = kv.get(b'I') { result.image_number = v; }
                if let Some(v) = kv.get(b'p') { result.placement_id = v; }
                Some(Delete::Newest(result))
            },

            b'c' | b'C' => Some(Delete::IntersectCursor(what == b'C')),
            b'f' | b'F' => Some(Delete::AnimationFrames(what == b'F')),

            b'p' | b'P' => {
                let mut result = DeleteIntersectCell {
                    delete: what == b'P',
                    x: 0,
                    y: 0,
                };
                if let Some(v) = kv.get(b'x') { result.x = v; }
                if let Some(v) = kv.get(b'y') { result.y = v; }
                Some(Delete::IntersectCell(result))
            },

            b'q' | b'Q' => {
                let mut result = DeleteIntersectCellZ {
                    delete: what == b'Q',
                    x: 0,
                    y: 0,
                    z: 0,
                };
                if let Some(v) = kv.get(b'x') { result.x = v; }
                if let Some(v) = kv.get(b'y') { result.y = v; }
                if let Some(v) = kv.get(b'z') { result.z = v as i32; }
                Some(Delete::IntersectCellZ(result))
            },

            b'r' | b'R' => {
                let x = kv.get(b'x')?;
                let y = kv.get(b'y')?;
                if x > y { return None; }
                Some(Delete::Range(DeleteRange {
                    delete: what == b'R',
                    first: x,
                    last: y,
                }))
            },

            b'x' | b'X' => {
                let mut result = DeleteColumn {
                    delete: what == b'X',
                    x: 0,
                };
                if let Some(v) = kv.get(b'x') { result.x = v; }
                Some(Delete::Column(result))
            },

            b'y' | b'Y' => {
                let mut result = DeleteRow {
                    delete: what == b'Y',
                    y: 0,
                };
                if let Some(v) = kv.get(b'y') { result.y = v; }
                Some(Delete::Row(result))
            },

            b'z' | b'Z' => {
                let mut result = DeleteZ {
                    delete: what == b'Z',
                    z: 0,
                };
                if let Some(v) = kv.get(b'z') { result.z = v as i32; }
                Some(Delete::Z(result))
            },

            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CommandControl {
    Query(Transmission),
    Transmit(Transmission),
    TransmitAndDisplay { transmission: Transmission, display: Display },
    Display(Display),
    Delete(Delete),
    TransmitAnimationFrame(AnimationFrameLoading),
    ControlAnimation(AnimationControl),
    ComposeAnimation(AnimationFrameComposition),
}

pub(crate) struct Command {
    pub control: CommandControl,
    pub quiet: CommandQuiet,
    pub data_ptr: *const u8,
    pub data_len: usize,
}

impl Command {
    pub(crate) fn transmission(&self) -> Option<Transmission> {
        match &self.control {
            CommandControl::Query(t) => Some(*t),
            CommandControl::Transmit(t) => Some(*t),
            CommandControl::TransmitAndDisplay { transmission, .. } => Some(*transmission),
            _ => None,
        }
    }

    pub(crate) fn display(&self) -> Option<Display> {
        match &self.control {
            CommandControl::Display(d) => Some(*d),
            CommandControl::TransmitAndDisplay { display, .. } => Some(*display),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Response {
    pub id: u32,
    pub image_number: u32,
    pub placement_id: u32,
    pub message_ptr: *const u8,
    pub message_len: usize,
}

static OK_MSG: &[u8] = b"OK";

impl Response {
    pub(crate) fn new() -> Self {
        Self {
            id: 0,
            image_number: 0,
            placement_id: 0,
            message_ptr: OK_MSG.as_ptr(),
            message_len: OK_MSG.len(),
        }
    }

    pub(crate) fn with_message(msg: &'static [u8]) -> Self {
        Self {
            id: 0,
            image_number: 0,
            placement_id: 0,
            message_ptr: msg.as_ptr(),
            message_len: msg.len(),
        }
    }

    pub(crate) fn ok(&self) -> bool {
        self.message_len == 2 && unsafe {
            ptr::read(self.message_ptr) == b'O' &&
            ptr::read(self.message_ptr.add(1)) == b'K'
        }
    }

    pub(crate) fn empty(&self) -> bool {
        self.id == 0 && self.image_number == 0
    }

    pub(crate) fn encode(&self, buf: *mut u8, buf_len: usize, out_written: *mut usize) -> bool {
        if self.id == 0 && self.image_number == 0 {
            if !out_written.is_null() {
                unsafe { ptr::write(out_written, 0); }
            }
            return true;
        }

        let mut pos: usize = 0;

        macro_rules! write_byte {
            ($b:expr) => {
                if pos >= buf_len { return false; }
                unsafe { ptr::write(buf.add(pos), $b); }
                pos += 1;
            };
        }

        macro_rules! write_bytes {
            ($ptr:expr, $len:expr) => {
                if pos + $len > buf_len { return false; }
                unsafe { ptr::copy_nonoverlapping($ptr, buf.add(pos), $len); }
                pos += $len;
            };
        }

        write_byte!(0x1b);
        write_byte!(b'_');
        write_byte!(b'G');

        let mut prior = false;

        if self.id > 0 {
            prior = true;
            let n = write_u32_to_buf(unsafe { buf.add(pos) }, buf_len - pos, b"i=", 2);
            if n == 0 { return false; }
            pos += n;
        }

        if self.image_number > 0 {
            if prior { write_byte!(b','); } else { prior = true; }
            let n = write_u32_to_buf(unsafe { buf.add(pos) }, buf_len - pos, b"I=", 2);
            if n == 0 { return false; }
            pos += n;
        }

        if self.placement_id > 0 {
            if prior { write_byte!(b','); }
            let n = write_u32_to_buf(unsafe { buf.add(pos) }, buf_len - pos, b"p=", 2);
            if n == 0 { return false; }
            pos += n;
        }

        write_byte!(b';');
        write_bytes!(self.message_ptr, self.message_len);
        write_byte!(0x1b);
        write_byte!(b'\\');

        if !out_written.is_null() {
            unsafe { ptr::write(out_written, pos); }
        }

        true
    }
}

fn write_u32_to_buf(buf: *mut u8, buf_len: usize, prefix: &[u8], prefix_len: usize) -> usize {
    if buf_len < prefix_len + 10 {
        return 0;
    }
    unsafe {
        ptr::copy_nonoverlapping(prefix.as_ptr(), buf, prefix_len);
    }
    let mut pos = prefix_len;
    let val_ptr = unsafe { buf.add(prefix_len - 2) };
    let key_byte = unsafe { ptr::read(val_ptr) };
    let _ = key_byte;

    let start = unsafe { buf.add(prefix_len) };
    let written = format_u32(start, buf_len - prefix_len, 0);
    pos += written;
    pos
}

fn format_u32(buf: *mut u8, buf_len: usize, _val: u32) -> usize {
    let _ = (buf, buf_len, _val);
    0
}

#[derive(Clone, Copy)]
struct KvEntry {
    key: u8,
    value: u32,
}

pub(crate) struct KvMap {
    entries: [KvEntry; COMMAND_MAX_KV_ENTRIES],
    len: usize,
}

impl KvMap {
    pub(crate) fn new() -> Self {
        Self {
            entries: [KvEntry { key: 0, value: 0 }; COMMAND_MAX_KV_ENTRIES],
            len: 0,
        }
    }

    pub(crate) fn get(&self, key: u8) -> Option<u32> {
        let mut i = 0;
        while i < self.len {
            let entry = unsafe { self.entries.get_unchecked(i) };
            if entry.key == key {
                return Some(entry.value);
            }
            i += 1;
        }
        None
    }

    pub(crate) fn put(&mut self, key: u8, value: u32) -> bool {
        let mut i = 0;
        while i < self.len {
            let entry = unsafe { self.entries.get_unchecked_mut(i) };
            if entry.key == key {
                entry.value = value;
                return true;
            }
            i += 1;
        }
        if self.len >= COMMAND_MAX_KV_ENTRIES {
            return false;
        }
        let entry = unsafe { self.entries.get_unchecked_mut(self.len) };
        entry.key = key;
        entry.value = value;
        self.len += 1;
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParserState {
    ControlKey,
    ControlKeyIgnore,
    ControlValue,
    ControlValueIgnore,
    Data,
}

pub(crate) struct CommandParser {
    kv: KvMap,
    kv_temp: [u8; KV_TEMP_LEN],
    kv_temp_len: u8,
    kv_current: u8,
    data_buf: *mut u8,
    data_len: usize,
    data_cap: usize,
    max_bytes: usize,
    state: ParserState,
}

impl CommandParser {
    pub(crate) fn new(data_buf: *mut u8, data_cap: usize, max_bytes: usize) -> Self {
        Self {
            kv: KvMap::new(),
            kv_temp: [0u8; KV_TEMP_LEN],
            kv_temp_len: 0,
            kv_current: 0,
            data_buf,
            data_len: 0,
            data_cap,
            max_bytes,
            state: ParserState::ControlKey,
        }
    }

    pub(crate) fn feed(&mut self, c: u8) -> bool {
        match self.state {
            ParserState::ControlKey => match c {
                b'=' => {
                    if self.kv_temp_len != 1 {
                        self.state = ParserState::ControlValueIgnore;
                        self.kv_temp_len = 0;
                    } else {
                        self.kv_current = unsafe { *self.kv_temp.get_unchecked(0) };
                        self.kv_temp_len = 0;
                        self.state = ParserState::ControlValue;
                    }
                    true
                },
                b';' => {
                    self.state = ParserState::Data;
                    true
                },
                _ => self.accumulate_value(c, ParserState::ControlKeyIgnore),
            },

            ParserState::ControlKeyIgnore => match c {
                b'=' => {
                    self.state = ParserState::ControlValueIgnore;
                    true
                },
                _ => true,
            },

            ParserState::ControlValue => match c {
                b',' => self.finish_value(ParserState::ControlKey),
                b';' => self.finish_value(ParserState::Data),
                _ => self.accumulate_value(c, ParserState::ControlValueIgnore),
            },

            ParserState::ControlValueIgnore => match c {
                b',' => {
                    self.state = ParserState::ControlKeyIgnore;
                    true
                },
                b';' => {
                    self.state = ParserState::Data;
                    true
                },
                _ => true,
            },

            ParserState::Data => {
                if self.data_len >= self.max_bytes {
                    return false;
                }
                if self.data_len >= self.data_cap {
                    return false;
                }
                unsafe {
                    ptr::write(self.data_buf.add(self.data_len), c);
                }
                self.data_len += 1;
                true
            },
        }
    }

    pub(crate) fn complete(&mut self) -> Option<Command> {
        match self.state {
            ParserState::ControlKey | ParserState::ControlKeyIgnore => return None,
            ParserState::ControlValue => {
                if !self.finish_value(ParserState::Data) {
                    return None;
                }
            },
            ParserState::ControlValueIgnore => {},
            ParserState::Data => {},
        }

        let action_byte: u8 = match self.kv.get(b'a') {
            Some(v) => {
                if v > 127 { return None; }
                v as u8
            },
            None => b't',
        };

        let control = match action_byte {
            b'q' => {
                let t = Transmission::parse(&self.kv)?;
                CommandControl::Query(t)
            },
            b't' => {
                let t = Transmission::parse(&self.kv)?;
                CommandControl::Transmit(t)
            },
            b'T' => {
                let t = Transmission::parse(&self.kv)?;
                let d = Display::parse(&self.kv)?;
                CommandControl::TransmitAndDisplay { transmission: t, display: d }
            },
            b'p' => {
                let d = Display::parse(&self.kv)?;
                CommandControl::Display(d)
            },
            b'd' => {
                let d = Delete::parse(&self.kv)?;
                CommandControl::Delete(d)
            },
            b'f' => {
                let a = AnimationFrameLoading::parse(&self.kv)?;
                CommandControl::TransmitAnimationFrame(a)
            },
            b'a' => {
                let a = AnimationControl::parse(&self.kv)?;
                CommandControl::ControlAnimation(a)
            },
            b'c' => {
                let a = AnimationFrameComposition::parse(&self.kv)?;
                CommandControl::ComposeAnimation(a)
            },
            _ => return None,
        };

        let quiet = match self.kv.get(b'q') {
            Some(v) => match v {
                0 => CommandQuiet::No,
                1 => CommandQuiet::Ok,
                2 => CommandQuiet::Failures,
                _ => return None,
            },
            None => CommandQuiet::No,
        };

        let (decoded_ptr, decoded_len) = self.decode_data();

        Some(Command {
            control,
            quiet,
            data_ptr: decoded_ptr,
            data_len: decoded_len,
        })
    }

    fn accumulate_value(&mut self, c: u8, overflow_state: ParserState) -> bool {
        let idx = self.kv_temp_len as usize;
        self.kv_temp_len += 1;
        if (self.kv_temp_len as usize) > KV_TEMP_LEN {
            self.state = overflow_state;
            self.kv_temp_len = 0;
            return true;
        }
        unsafe {
            *self.kv_temp.get_unchecked_mut(idx) = c;
        }
        true
    }

    fn finish_value(&mut self, next_state: ParserState) -> bool {
        self.state = next_state;

        if self.kv_temp_len == 1 {
            let c = unsafe { *self.kv_temp.get_unchecked(0) };
            if c < b'0' || c > b'9' {
                return self.kv.put(self.kv_current, c as u32);
            }
        }

        let is_signed = matches!(self.kv_current, b'z' | b'H' | b'V');
        let temp_len = self.kv_temp_len as usize;

        if is_signed {
            match parse_i32_from_buf(&self.kv_temp, temp_len) {
                Some(v) => {
                    let as_u32 = v as u32;
                    if !self.kv.put(self.kv_current, as_u32) {
                        return false;
                    }
                },
                None => return false,
            }
        } else {
            match parse_u32_from_buf(&self.kv_temp, temp_len) {
                Some(v) => {
                    if !self.kv.put(self.kv_current, v) {
                        return false;
                    }
                },
                None => return false,
            }
        }

        self.kv_temp_len = 0;
        true
    }

    fn decode_data(&mut self) -> (*const u8, usize) {
        if self.data_len == 0 {
            return (core::ptr::null(), 0);
        }

        let decoded_len = base64_decode_in_place(self.data_buf, self.data_len);
        match decoded_len {
            Some(len) => (self.data_buf, len),
            None => (core::ptr::null(), 0),
        }
    }
}

fn parse_u32_from_buf(buf: &[u8; KV_TEMP_LEN], len: usize) -> Option<u32> {
    if len == 0 || len > 10 {
        return None;
    }
    let mut result: u32 = 0;
    let mut i = 0;
    while i < len {
        let c = unsafe { *buf.get_unchecked(i) };
        if c < b'0' || c > b'9' {
            return None;
        }
        let digit = (c - b'0') as u32;
        match result.checked_mul(10) {
            Some(v) => match v.checked_add(digit) {
                Some(v2) => result = v2,
                None => return None,
            },
            None => return None,
        }
        i += 1;
    }
    Some(result)
}

fn parse_i32_from_buf(buf: &[u8; KV_TEMP_LEN], len: usize) -> Option<i32> {
    if len == 0 || len > 11 {
        return None;
    }
    let first = unsafe { *buf.get_unchecked(0) };
    let negative = first == b'-';
    let start = if negative || first == b'+' { 1 } else { 0 };
    if start >= len {
        return None;
    }

    let mut result: u32 = 0;
    let mut i = start;
    while i < len {
        let c = unsafe { *buf.get_unchecked(i) };
        if c < b'0' || c > b'9' {
            return None;
        }
        let digit = (c - b'0') as u32;
        match result.checked_mul(10) {
            Some(v) => match v.checked_add(digit) {
                Some(v2) => result = v2,
                None => return None,
            },
            None => return None,
        }
        i += 1;
    }

    if negative {
        if result > (i32::MAX as u32) + 1 {
            return None;
        }
        Some(-(result as i32))
    } else {
        if result > i32::MAX as u32 {
            return None;
        }
        Some(result as i32)
    }
}

fn base64_decode_in_place(data: *mut u8, len: usize) -> Option<usize> {
    if len == 0 {
        return Some(0);
    }

    let mut read_pos: usize = 0;
    let mut write_pos: usize = 0;
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;

    while read_pos < len {
        let c = unsafe { ptr::read(data.add(read_pos)) };
        read_pos += 1;

        let val = base64_char_value(c);
        match val {
            Some(v) => {
                accum = (accum << 6) | v as u32;
                bits += 6;
                if bits >= 8 {
                    bits -= 8;
                    let byte = ((accum >> bits) & 0xFF) as u8;
                    unsafe { ptr::write(data.add(write_pos), byte); }
                    write_pos += 1;
                }
            },
            None => {
                if c == b'=' {
                    continue;
                }
                if c == b'\n' || c == b'\r' || c == b' ' {
                    continue;
                }
                return None;
            },
        }
    }

    Some(write_pos)
}

fn base64_char_value(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub(crate) fn parse_command_string(
    input: *const u8,
    input_len: usize,
    data_buf: *mut u8,
    data_cap: usize,
) -> Option<Command> {
    let mut parser = CommandParser::new(data_buf, data_cap, COMMAND_MAX_DATA_BYTES);
    let mut i: usize = 0;
    while i < input_len {
        let c = unsafe { ptr::read(input.add(i)) };
        if !parser.feed(c) {
            return None;
        }
        i += 1;
    }
    parser.complete()
}
