use crate::early::*;
use crate::size_types::*;

pub type StyleId = u16;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentTag {
    #[default]
    Codepoint = 0,
    CodepointGrapheme = 1,
    BgColorPalette = 2,
    BgColorRgb = 3,
}

impl ContentTag {
    #[inline(always)]
    pub fn from_u2(v: u8) -> Self {
        match v & 0b11 {
            0 => Self::Codepoint,
            1 => Self::CodepointGrapheme,
            2 => Self::BgColorPalette,
            3 => Self::BgColorRgb,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CellRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl CellRgb {
    #[inline(always)]
    pub fn pack(self) -> u32 {
        (self.r as u32) | ((self.g as u32) << 8) | ((self.b as u32) << 16)
    }

    #[inline(always)]
    pub fn unpack(bits: u32) -> Self {
        Self {
            r: (bits & 0xFF) as u8,
            g: ((bits >> 8) & 0xFF) as u8,
            b: ((bits >> 16) & 0xFF) as u8,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Wide {
    #[default]
    Narrow = 0,
    Wide = 1,
    SpacerTail = 2,
    SpacerHead = 3,
}

impl Wide {
    #[inline(always)]
    pub fn from_u2(v: u8) -> Self {
        match v & 0b11 {
            0 => Self::Narrow,
            1 => Self::Wide,
            2 => Self::SpacerTail,
            3 => Self::SpacerHead,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SemanticContent {
    #[default]
    Output = 0,
    Input = 1,
    Prompt = 2,
}

impl SemanticContent {
    #[inline(always)]
    pub fn from_u2(v: u8) -> Self {
        match v & 0b11 {
            0 => Self::Output,
            1 => Self::Input,
            2 => Self::Prompt,
            _ => Self::Output,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SemanticPrompt {
    #[default]
    None = 0,
    Prompt = 1,
    PromptContinuation = 2,
}

impl SemanticPrompt {
    #[inline(always)]
    pub fn from_u2(v: u8) -> Self {
        match v & 0b11 {
            0 => Self::None,
            1 => Self::Prompt,
            2 => Self::PromptContinuation,
            _ => Self::None,
        }
    }
}

const CONTENT_TAG_MASK: u64 = 0b11;
const CONTENT_TAG_SHIFT: u32 = 0;

const CONTENT_MASK: u64 = (1u64 << 24) - 1;
const CONTENT_SHIFT: u32 = 2;

const STYLE_ID_MASK: u64 = (1u64 << 16) - 1;
const STYLE_ID_SHIFT: u32 = 26;

const WIDE_MASK: u64 = 0b11;
const WIDE_SHIFT: u32 = 42;

const PROTECTED_BIT: u32 = 44;
const PROTECTED_MASK: u64 = 1u64 << PROTECTED_BIT;

const HYPERLINK_BIT: u32 = 45;
const HYPERLINK_MASK: u64 = 1u64 << HYPERLINK_BIT;

const SEMANTIC_CONTENT_MASK: u64 = 0b11;
const SEMANTIC_CONTENT_SHIFT: u32 = 46;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Cell(pub u64);

impl Cell {
    #[inline(always)]
    pub fn bits(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn content_tag(self) -> ContentTag {
        ContentTag::from_u2(((self.0 >> CONTENT_TAG_SHIFT) & CONTENT_TAG_MASK) as u8)
    }

    #[inline(always)]
    pub fn set_content_tag(&mut self, tag: ContentTag) {
        self.0 = (self.0 & !(CONTENT_TAG_MASK << CONTENT_TAG_SHIFT))
            | ((tag as u64) << CONTENT_TAG_SHIFT);
    }

    #[inline(always)]
    pub fn content_codepoint(self) -> u32 {
        ((self.0 >> CONTENT_SHIFT) & CONTENT_MASK) as u32
    }

    #[inline(always)]
    pub fn set_content_codepoint(&mut self, cp: u32) {
        self.0 = (self.0 & !(CONTENT_MASK << CONTENT_SHIFT))
            | (((cp as u64) & CONTENT_MASK) << CONTENT_SHIFT);
    }

    #[inline(always)]
    pub fn content_color_palette(self) -> u8 {
        ((self.0 >> CONTENT_SHIFT) & 0xFF) as u8
    }

    #[inline(always)]
    pub fn set_content_color_palette(&mut self, idx: u8) {
        self.0 = (self.0 & !(CONTENT_MASK << CONTENT_SHIFT))
            | (((idx as u64) & CONTENT_MASK) << CONTENT_SHIFT);
    }

    #[inline(always)]
    pub fn content_color_rgb(self) -> CellRgb {
        CellRgb::unpack(((self.0 >> CONTENT_SHIFT) & CONTENT_MASK) as u32)
    }

    #[inline(always)]
    pub fn set_content_color_rgb(&mut self, rgb: CellRgb) {
        self.0 = (self.0 & !(CONTENT_MASK << CONTENT_SHIFT))
            | (((rgb.pack() as u64) & CONTENT_MASK) << CONTENT_SHIFT);
    }

    #[inline(always)]
    pub fn style_id(self) -> StyleId {
        ((self.0 >> STYLE_ID_SHIFT) & STYLE_ID_MASK) as StyleId
    }

    #[inline(always)]
    pub fn set_style_id(&mut self, id: StyleId) {
        self.0 = (self.0 & !(STYLE_ID_MASK << STYLE_ID_SHIFT))
            | (((id as u64) & STYLE_ID_MASK) << STYLE_ID_SHIFT);
    }

    #[inline(always)]
    pub fn wide(self) -> Wide {
        Wide::from_u2(((self.0 >> WIDE_SHIFT) & WIDE_MASK) as u8)
    }

    #[inline(always)]
    pub fn set_wide(&mut self, w: Wide) {
        self.0 = (self.0 & !(WIDE_MASK << WIDE_SHIFT)) | (((w as u64) & WIDE_MASK) << WIDE_SHIFT);
    }

    #[inline(always)]
    pub fn protected(self) -> bool {
        (self.0 & PROTECTED_MASK) != 0
    }

    #[inline(always)]
    pub fn set_protected(&mut self, v: bool) {
        if v {
            self.0 |= PROTECTED_MASK;
        } else {
            self.0 &= !PROTECTED_MASK;
        }
    }

    #[inline(always)]
    pub fn hyperlink(self) -> bool {
        (self.0 & HYPERLINK_MASK) != 0
    }

    #[inline(always)]
    pub fn set_hyperlink(&mut self, v: bool) {
        if v {
            self.0 |= HYPERLINK_MASK;
        } else {
            self.0 &= !HYPERLINK_MASK;
        }
    }

    #[inline(always)]
    pub fn semantic_content(self) -> SemanticContent {
        SemanticContent::from_u2(((self.0 >> SEMANTIC_CONTENT_SHIFT) & SEMANTIC_CONTENT_MASK) as u8)
    }

    #[inline(always)]
    pub fn set_semantic_content(&mut self, sc: SemanticContent) {
        self.0 = (self.0 & !(SEMANTIC_CONTENT_MASK << SEMANTIC_CONTENT_SHIFT))
            | (((sc as u64) & SEMANTIC_CONTENT_MASK) << SEMANTIC_CONTENT_SHIFT);
    }

    #[inline(always)]
    pub fn cval(self) -> u64 {
        self.0
    }

    pub fn init(cp: u32) -> Cell {
        let mut cell = Cell(0);
        cell.set_content_tag(ContentTag::Codepoint);
        cell.set_content_codepoint(cp);
        cell
    }

    #[inline(always)]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub fn has_text(self) -> bool {
        match self.content_tag() {
            ContentTag::Codepoint | ContentTag::CodepointGrapheme => self.content_codepoint() != 0,
            ContentTag::BgColorPalette | ContentTag::BgColorRgb => false,
        }
    }

    #[inline(always)]
    pub fn codepoint(&self) -> u32 {
        match self.content_tag() {
            ContentTag::Codepoint | ContentTag::CodepointGrapheme => self.content_codepoint(),
            ContentTag::BgColorPalette | ContentTag::BgColorRgb => 0,
        }
    }

    #[inline(always)]
    pub fn grid_width(self) -> u8 {
        match self.wide() {
            Wide::Narrow | Wide::SpacerHead | Wide::SpacerTail => 1,
            Wide::Wide => 2,
        }
    }

    #[inline(always)]
    pub fn has_styling(self) -> bool {
        self.style_id() != 0
    }

    pub fn is_empty(self) -> bool {
        match self.content_tag() {
            ContentTag::Codepoint | ContentTag::CodepointGrapheme => {
                !self.has_text() && self.wide() == Wide::Narrow
            }
            ContentTag::BgColorPalette | ContentTag::BgColorRgb => false,
        }
    }

    #[inline(always)]
    pub fn has_grapheme(self) -> bool {
        self.content_tag() == ContentTag::CodepointGrapheme
    }

    #[inline(always)]
    pub fn has_text_any(cells: &[Cell]) -> bool {
        cells.iter().any(|c| c.has_text())
    }
}

const ROW_CELLS_BITS: u32 = 32;
const ROW_CELLS_MASK: u64 = (1u64 << ROW_CELLS_BITS) - 1;

const ROW_WRAP_BIT: u32 = 32;
const ROW_WRAP_MASK: u64 = 1u64 << ROW_WRAP_BIT;

const ROW_WRAP_CONT_BIT: u32 = 33;
const ROW_WRAP_CONT_MASK: u64 = 1u64 << ROW_WRAP_CONT_BIT;

const ROW_GRAPHEME_BIT: u32 = 34;
const ROW_GRAPHEME_MASK: u64 = 1u64 << ROW_GRAPHEME_BIT;

const ROW_STYLED_BIT: u32 = 35;
const ROW_STYLED_MASK: u64 = 1u64 << ROW_STYLED_BIT;

const ROW_HYPERLINK_BIT: u32 = 36;
const ROW_HYPERLINK_MASK: u64 = 1u64 << ROW_HYPERLINK_BIT;

const ROW_SEMANTIC_PROMPT_SHIFT: u32 = 37;
const ROW_SEMANTIC_PROMPT_MASK: u64 = 0b11 << ROW_SEMANTIC_PROMPT_SHIFT;

const ROW_KITTY_BIT: u32 = 39;
const ROW_KITTY_MASK: u64 = 1u64 << ROW_KITTY_BIT;

const ROW_DIRTY_BIT: u32 = 40;
const ROW_DIRTY_MASK: u64 = 1u64 << ROW_DIRTY_BIT;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Row(pub u64);

impl Row {
    #[inline(always)]
    pub fn bits(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn cells(self) -> Offset {
        Offset {
            offset: (self.0 & ROW_CELLS_MASK) as OffsetInt,
        }
    }

    #[inline(always)]
    pub fn set_cells(&mut self, offset: Offset) {
        self.0 = (self.0 & !ROW_CELLS_MASK) | ((offset.offset as u64) & ROW_CELLS_MASK);
    }

    #[inline(always)]
    pub fn wrap(self) -> bool {
        (self.0 & ROW_WRAP_MASK) != 0
    }

    #[inline(always)]
    pub fn set_wrap(&mut self, v: bool) {
        if v {
            self.0 |= ROW_WRAP_MASK;
        } else {
            self.0 &= !ROW_WRAP_MASK;
        }
    }

    #[inline(always)]
    pub fn wrap_continuation(self) -> bool {
        (self.0 & ROW_WRAP_CONT_MASK) != 0
    }

    #[inline(always)]
    pub fn set_wrap_continuation(&mut self, v: bool) {
        if v {
            self.0 |= ROW_WRAP_CONT_MASK;
        } else {
            self.0 &= !ROW_WRAP_CONT_MASK;
        }
    }

    #[inline(always)]
    pub fn grapheme(self) -> bool {
        (self.0 & ROW_GRAPHEME_MASK) != 0
    }

    #[inline(always)]
    pub fn set_grapheme(&mut self, v: bool) {
        if v {
            self.0 |= ROW_GRAPHEME_MASK;
        } else {
            self.0 &= !ROW_GRAPHEME_MASK;
        }
    }

    #[inline(always)]
    pub fn styled(self) -> bool {
        (self.0 & ROW_STYLED_MASK) != 0
    }

    #[inline(always)]
    pub fn set_styled(&mut self, v: bool) {
        if v {
            self.0 |= ROW_STYLED_MASK;
        } else {
            self.0 &= !ROW_STYLED_MASK;
        }
    }

    #[inline(always)]
    pub fn hyperlink(self) -> bool {
        (self.0 & ROW_HYPERLINK_MASK) != 0
    }

    #[inline(always)]
    pub fn set_hyperlink(&mut self, v: bool) {
        if v {
            self.0 |= ROW_HYPERLINK_MASK;
        } else {
            self.0 &= !ROW_HYPERLINK_MASK;
        }
    }

    #[inline(always)]
    pub fn semantic_prompt(self) -> SemanticPrompt {
        SemanticPrompt::from_u2(
            ((self.0 & ROW_SEMANTIC_PROMPT_MASK) >> ROW_SEMANTIC_PROMPT_SHIFT) as u8,
        )
    }

    #[inline(always)]
    pub fn set_semantic_prompt(&mut self, sp: SemanticPrompt) {
        self.0 = (self.0 & !ROW_SEMANTIC_PROMPT_MASK)
            | (((sp as u64) & 0b11) << ROW_SEMANTIC_PROMPT_SHIFT);
    }

    #[inline(always)]
    pub fn kitty_virtual_placeholder(self) -> bool {
        (self.0 & ROW_KITTY_MASK) != 0
    }

    #[inline(always)]
    pub fn set_kitty_virtual_placeholder(&mut self, v: bool) {
        if v {
            self.0 |= ROW_KITTY_MASK;
        } else {
            self.0 &= !ROW_KITTY_MASK;
        }
    }

    #[inline(always)]
    pub fn dirty(self) -> bool {
        (self.0 & ROW_DIRTY_MASK) != 0
    }

    #[inline(always)]
    pub fn set_dirty(&mut self, v: bool) {
        if v {
            self.0 |= ROW_DIRTY_MASK;
        } else {
            self.0 &= !ROW_DIRTY_MASK;
        }
    }

    #[inline(always)]
    pub fn managed_memory(self) -> bool {
        self.styled() || self.hyperlink() || self.grapheme()
    }

    #[inline(always)]
    pub fn cval(self) -> u64 {
        self.0
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageSize {
    pub cols: CellCountInt,
    pub rows: CellCountInt,
}

impl Default for PageSize {
    fn default() -> Self {
        Self { cols: 0, rows: 0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CapacityAdjustment {
    pub cols: Option<CellCountInt>,
}

impl Default for CapacityAdjustment {
    fn default() -> Self {
        Self { cols: None }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageCapacity {
    pub cols: CellCountInt,
    pub rows: CellCountInt,
    pub styles: StyleCountInt,
    pub hyperlink_bytes: HyperlinkCountInt,
    pub grapheme_bytes: GraphemeBytesInt,
    pub string_bytes: StringBytesInt,
}

impl Default for PageCapacity {
    fn default() -> Self {
        Self {
            cols: 0,
            rows: 0,
            styles: 16,
            hyperlink_bytes: 0,
            grapheme_bytes: 0,
            string_bytes: 0,
        }
    }
}

impl PageCapacity {
    pub fn adjust(&self, req: &CapacityAdjustment) -> Result<Self, ()> {
        let mut adjusted = *self;
        if let Some(cols) = req.cols {
            let bits_per_row = Self::bits_per_row(cols);
            if bits_per_row == 0 {
                return Err(());
            }
            let available_bits = self.available_bits_for_grid();
            let new_rows = available_bits / bits_per_row;
            if new_rows == 0 {
                return Err(());
            }
            adjusted.cols = cols;
            adjusted.rows = new_rows.min(CellCountInt::MAX as usize) as CellCountInt;
        }
        Ok(adjusted)
    }

    fn bits_per_row(cols: CellCountInt) -> usize {
        core::mem::size_of::<Row>() * 8 + core::mem::size_of::<Cell>() * 8 * cols as usize
    }

    fn available_bits_for_grid(&self) -> usize {
        let rows_bytes = self.rows as usize * core::mem::size_of::<Row>();
        let cells_bytes = self.rows as usize * self.cols as usize * core::mem::size_of::<Cell>();
        (rows_bytes + cells_bytes) * 8
    }
}

#[cfg(not(target_os = "windows"))]
mod page_alloc_impl {
    use core::ffi::c_int;
    use core::ffi::c_void;

    const PROT_READ: c_int = 0x1;
    const PROT_WRITE: c_int = 0x2;
    const MAP_PRIVATE: c_int = 0x02;
    const MAP_ANONYMOUS: c_int = if cfg!(target_os = "macos") {
        0x1000
    } else {
        0x20
    };
    const MAP_FAILED: *mut c_void = !0 as *mut c_void;

    extern "C" {
        fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: c_int,
            flags: c_int,
            fd: c_int,
            offset: i64,
        ) -> *mut c_void;
        fn munmap(addr: *mut c_void, len: usize) -> c_int;
    }

    pub fn page_alloc(n: usize) -> Result<&'static mut [u8], ()> {
        unsafe {
            let ptr = mmap(
                core::ptr::null_mut(),
                n,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            );
            if ptr == MAP_FAILED || ptr.is_null() {
                return Err(());
            }
            Ok(core::slice::from_raw_parts_mut(ptr as *mut u8, n))
        }
    }

    pub fn page_free(mem: &mut [u8]) {
        unsafe {
            munmap(mem.as_mut_ptr() as *mut c_void, mem.len());
        }
    }
}

#[cfg(target_os = "windows")]
mod page_alloc_impl {
    use core::ffi::c_void;

    const MEM_COMMIT: u32 = 0x00001000;
    const MEM_RESERVE: u32 = 0x00002000;
    const MEM_RELEASE: u32 = 0x00008000;
    const PAGE_READWRITE: u32 = 0x04;

    extern "system" {
        fn VirtualAlloc(
            lpAddress: *mut c_void,
            dwSize: usize,
            flAllocationType: u32,
            flProtect: u32,
        ) -> *mut c_void;
        fn VirtualFree(lpAddress: *mut c_void, dwSize: usize, dwFreeType: u32) -> i32;
    }

    pub fn page_alloc(n: usize) -> Result<&'static mut [u8], ()> {
        unsafe {
            let ptr = VirtualAlloc(
                core::ptr::null_mut(),
                n,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );
            if ptr.is_null() {
                return Err(());
            }
            Ok(core::slice::from_raw_parts_mut(ptr as *mut u8, n))
        }
    }

    pub fn page_free(mem: &mut [u8]) {
        unsafe {
            VirtualFree(mem.as_mut_ptr() as *mut c_void, 0, MEM_RELEASE);
        }
    }
}

pub use page_alloc_impl::{page_alloc, page_free};
