//! Selection derivation helpers (`selectWord`, `selectLine`, etc.).
//!
//! Port of `src/terminal/Screen.zig` selection derivation and
//! `PageList.highlightSemanticContent` (output path for `selectOutput`).

use crate::highlight::{HighlightUntracked, Pin};
use crate::page_list_methods::{
    cell_iterator_at_pin, PromptIterator, RowIterator,
};
use crate::page_list_types::{PageList, PageListDirection};
use crate::page_types::SemanticContent;
use crate::point::PointTag;
use crate::screen_types::Screen;
use crate::selection_codepoints::{DEFAULT_LINE_WHITESPACE, DEFAULT_WORD_BOUNDARIES};
use crate::selection_types::Selection;

pub struct SelectLineOpts<'a> {
    pub pin: Pin,
    pub whitespace: Option<&'a [u32]>,
    pub semantic_prompt_boundary: bool,
}

fn pages_from_screen(screen: &Screen) -> Option<&PageList> {
    if screen.pages.is_null() {
        return None;
    }
    Some(unsafe { &*screen.pages })
}

fn codepoint_in_slice(cp: u32, slice: &[u32]) -> bool {
    slice.iter().any(|&c| c == cp)
}

pub(crate) fn default_word_boundaries_u32() -> &'static [u32] {
    const BOUNDARIES: [u32; 20] = [
        DEFAULT_WORD_BOUNDARIES[0] as u32,
        DEFAULT_WORD_BOUNDARIES[1] as u32,
        DEFAULT_WORD_BOUNDARIES[2] as u32,
        DEFAULT_WORD_BOUNDARIES[3] as u32,
        DEFAULT_WORD_BOUNDARIES[4] as u32,
        DEFAULT_WORD_BOUNDARIES[5] as u32,
        DEFAULT_WORD_BOUNDARIES[6] as u32,
        DEFAULT_WORD_BOUNDARIES[7] as u32,
        DEFAULT_WORD_BOUNDARIES[8] as u32,
        DEFAULT_WORD_BOUNDARIES[9] as u32,
        DEFAULT_WORD_BOUNDARIES[10] as u32,
        DEFAULT_WORD_BOUNDARIES[11] as u32,
        DEFAULT_WORD_BOUNDARIES[12] as u32,
        DEFAULT_WORD_BOUNDARIES[13] as u32,
        DEFAULT_WORD_BOUNDARIES[14] as u32,
        DEFAULT_WORD_BOUNDARIES[15] as u32,
        DEFAULT_WORD_BOUNDARIES[16] as u32,
        DEFAULT_WORD_BOUNDARIES[17] as u32,
        DEFAULT_WORD_BOUNDARIES[18] as u32,
        DEFAULT_WORD_BOUNDARIES[19] as u32,
    ];
    &BOUNDARIES
}

pub(crate) fn default_line_whitespace_u32() -> &'static [u32] {
    const WS: [u32; 3] = [
        DEFAULT_LINE_WHITESPACE[0] as u32,
        DEFAULT_LINE_WHITESPACE[1] as u32,
        DEFAULT_LINE_WHITESPACE[2] as u32,
    ];
    &WS
}

pub fn select_word(pin: Pin, boundary_codepoints: &[u32]) -> Option<Selection> {
    let (_row, start_cell) = pin.row_and_cell_ptr();
    if !unsafe { (*start_cell).has_text() } {
        return None;
    }

    let expect_boundary =
        codepoint_in_slice(unsafe { (*start_cell).codepoint() }, boundary_codepoints);

    let end = {
        let mut it = cell_iterator_at_pin(pin, PageListDirection::RightDown, None);
        let mut prev = match it.next() {
            Some(p) => p,
            None => return None,
        };
        loop {
            match it.next() {
                None => break prev,
                Some(p) => {
                    let (row, cell) = p.row_and_cell_ptr();
                    if !unsafe { (*cell).has_text() } {
                        break prev;
                    }
                    let this_boundary =
                        codepoint_in_slice(unsafe { (*cell).codepoint() }, boundary_codepoints);
                    if this_boundary != expect_boundary {
                        break prev;
                    }
                    let cols = unsafe { (*p.node).data.size.cols };
                    if p.x == cols - 1 && !unsafe { (*row).wrap() } {
                        break p;
                    }
                    prev = p;
                }
            }
        }
    };

    let start = {
        let mut it = cell_iterator_at_pin(pin, PageListDirection::LeftUp, None);
        let mut prev = match it.next() {
            Some(p) => p,
            None => return None,
        };
        loop {
            match it.next() {
                None => break prev,
                Some(p) => {
                    let (row, cell) = p.row_and_cell_ptr();
                    let cols = unsafe { (*p.node).data.size.cols };
                    if p.x == cols - 1 && !unsafe { (*row).wrap() } {
                        break prev;
                    }
                    if !unsafe { (*cell).has_text() } {
                        break prev;
                    }
                    let this_boundary =
                        codepoint_in_slice(unsafe { (*cell).codepoint() }, boundary_codepoints);
                    if this_boundary != expect_boundary {
                        break prev;
                    }
                    prev = p;
                }
            }
        }
    };

    Some(Selection::init(start, end, false))
}

pub fn select_word_between(
    start: Pin,
    end: Pin,
    boundary_codepoints: &[u32],
) -> Option<Selection> {
    let dir = if start.before(end) {
        PageListDirection::RightDown
    } else {
        PageListDirection::LeftUp
    };
    let mut it = cell_iterator_at_pin(start, dir, Some(end));
    while let Some(pin) = it.next() {
        match dir {
            PageListDirection::RightDown => {
                if end.before(pin) {
                    return None;
                }
            }
            PageListDirection::LeftUp => {
                if pin.before(end) {
                    return None;
                }
            }
        }
        if let Some(sel) = select_word(pin, boundary_codepoints) {
            return Some(sel);
        }
    }
    None
}

pub fn select_line(_screen: &Screen, opts: SelectLineOpts<'_>) -> Option<Selection> {
    let semantic_prompt_state: Option<SemanticContent> = if opts.semantic_prompt_boundary {
        let (_row, cell) = opts.pin.row_and_cell_ptr();
        Some(unsafe { (*cell).semantic_content() })
    } else {
        None
    };

    let start_pin = find_line_start_pin(opts.pin, semantic_prompt_state);
    let end_pin = find_line_end_pin(opts.pin, semantic_prompt_state)?;

    let start = trim_line_start(start_pin, end_pin, opts.whitespace)?;
    let end = trim_line_end(end_pin, start_pin, opts.whitespace)?;

    Some(Selection::init(start, end, false))
}

fn find_line_start_pin(
    pin: Pin,
    semantic_prompt_state: Option<SemanticContent>,
) -> Pin {
    let mut it = RowIterator::new_from_pin(pin, PageListDirection::LeftUp);
    let mut it_prev = match it.next() {
        Some(p) => p,
        None => return pin,
    };

    if let Some(v) = semantic_prompt_state {
        let row = it_prev.row_ptr();
        let cells = unsafe {
            let page = &(*it_prev.node).data;
            core::slice::from_raw_parts(
                page.row_cells_ptr(row),
                page.size.cols as usize,
            )
        };
        for i in 0..=pin.x as usize {
            let x_rev = pin.x as usize - i;
            if cells[x_rev].semantic_content() != v {
                let mut copy = it_prev;
                copy.x = (x_rev + 1) as u16;
                return copy;
            }
        }
    }

    while let Some(p) = it.next() {
        let row = p.row_ptr();

        if !unsafe { (*row).wrap() } {
            let mut copy = it_prev;
            copy.x = 0;
            return copy;
        }

        if let Some(v) = semantic_prompt_state {
            let cells = unsafe {
                let page = &(*p.node).data;
                core::slice::from_raw_parts(
                    page.row_cells_ptr(row),
                    page.size.cols as usize,
                )
            };
            for x in 0..cells.len() {
                let x_rev = cells.len() - 1 - x;
                if cells[x_rev].semantic_content() != v {
                    return it_prev;
                }
                it_prev = p;
                it_prev.x = x_rev as u16;
            }
            continue;
        }

        it_prev = p;
    }

    let mut copy = it_prev;
    copy.x = 0;
    copy
}

fn find_line_end_pin(
    pin: Pin,
    semantic_prompt_state: Option<SemanticContent>,
) -> Option<Pin> {
    let mut it = RowIterator::new_from_pin(pin, PageListDirection::RightDown);
    while let Some(p) = it.next() {
        let row = p.row_ptr();

        if let Some(v) = semantic_prompt_state {
            let cells = unsafe {
                let page = &(*p.node).data;
                core::slice::from_raw_parts(
                    page.row_cells_ptr(row),
                    page.size.cols as usize,
                )
            };

            let start_offset = if p.node == pin.node && p.y == pin.y {
                pin.x as usize
            } else {
                0
            };

            if start_offset == 0 && cells[0].semantic_content() != v {
                let mut prev = p.up(1)?;
                prev.x = unsafe { (*prev.node).data.size.cols } - 1;
                return Some(prev);
            }

            for (x, cell) in cells.iter().enumerate().skip(start_offset) {
                if cell.semantic_content() != v {
                    let mut copy = p;
                    copy.x = (x.saturating_sub(1)) as u16;
                    return Some(copy);
                }
            }
        }

        if !unsafe { (*row).wrap() } {
            let mut copy = p;
            copy.x = unsafe { (*p.node).data.size.cols } - 1;
            return Some(copy);
        }
    }

    None
}

fn trim_line_start(
    start_pin: Pin,
    end_pin: Pin,
    whitespace: Option<&[u32]>,
) -> Option<Pin> {
    let Some(whitespace) = whitespace else {
        return Some(start_pin);
    };
    let mut it = cell_iterator_at_pin(start_pin, PageListDirection::RightDown, Some(end_pin));
    while let Some(p) = it.next() {
        let (_row, cell) = p.row_and_cell_ptr();
        if !unsafe { (*cell).has_text() } {
            continue;
        }
        if codepoint_in_slice(unsafe { (*cell).codepoint() }, whitespace) {
            continue;
        }
        return Some(p);
    }
    None
}

fn trim_line_end(end_pin: Pin, start_pin: Pin, whitespace: Option<&[u32]>) -> Option<Pin> {
    let Some(whitespace) = whitespace else {
        return Some(end_pin);
    };
    let mut it = cell_iterator_at_pin(end_pin, PageListDirection::LeftUp, Some(start_pin));
    while let Some(p) = it.next() {
        let (_row, cell) = p.row_and_cell_ptr();
        if !unsafe { (*cell).has_text() } {
            continue;
        }
        if codepoint_in_slice(unsafe { (*cell).codepoint() }, whitespace) {
            continue;
        }
        return Some(p);
    }
    None
}

pub fn select_all(screen: &Screen) -> Option<Selection> {
    let pages = pages_from_screen(screen)?;
    const WHITESPACE: [u32; 3] = [0, b' ' as u32, b'\t' as u32];

    let tl = pages.get_top_left(PointTag::SCREEN);
    let br = pages.get_bottom_right(PointTag::SCREEN)?;

    let mut start: Option<Pin> = None;
    {
        let mut it = cell_iterator_at_pin(tl, PageListDirection::RightDown, Some(br));
        while let Some(p) = it.next() {
            let (_row, cell) = p.row_and_cell_ptr();
            if !unsafe { (*cell).has_text() } {
                continue;
            }
            if codepoint_in_slice(unsafe { (*cell).codepoint() }, &WHITESPACE) {
                continue;
            }
            start = Some(p);
            break;
        }
    }
    let start = start?;

    let mut end: Option<Pin> = None;
    {
        let mut it = cell_iterator_at_pin(br, PageListDirection::LeftUp, Some(tl));
        while let Some(p) = it.next() {
            let (_row, cell) = p.row_and_cell_ptr();
            if !unsafe { (*cell).has_text() } {
                continue;
            }
            if codepoint_in_slice(unsafe { (*cell).codepoint() }, &WHITESPACE) {
                continue;
            }
            end = Some(p);
            break;
        }
    }
    let end = end?;

    Some(Selection::init(start, end, false))
}

pub fn select_output(screen: &Screen, pin: Pin) -> Option<Selection> {
    let (_row, cell) = pin.row_and_cell_ptr();
    if unsafe { (*cell).semantic_content() } != SemanticContent::Output {
        return None;
    }

    let pages = pages_from_screen(screen)?;

    let prompt_pin = {
        let mut it = PromptIterator::new(Some(pin), None, PageListDirection::LeftUp);
        if let Some(p) = it.next() {
            p
        } else {
            let mut it = PromptIterator::new(Some(pin), None, PageListDirection::RightDown);
            let next = it.next()?;

            let start_pin = pages.get_top_left(PointTag::SCREEN);
            let mut end_pin = next.up(1)?;
            end_pin.x = unsafe { (*end_pin.node).data.size.cols } - 1;

            let mut cell_it =
                cell_iterator_at_pin(end_pin, PageListDirection::LeftUp, Some(start_pin));
            while let Some(p) = cell_it.next() {
                end_pin = p;
                let (_row, c) = p.row_and_cell_ptr();
                if unsafe { (*c).has_text() } {
                    break;
                }
            }

            return Some(Selection::init(start_pin, end_pin, false));
        }
    };

    let mut hl = highlight_semantic_content_output(pages, prompt_pin)?;

    let mut cell_it = cell_iterator_at_pin(hl.end, PageListDirection::LeftUp, Some(hl.start));
    while let Some(p) = cell_it.next() {
        let (_row, cell) = p.row_and_cell_ptr();
        hl.end = p;
        if unsafe { (*cell).has_text() } {
            break;
        }
    }

    Some(Selection::init(hl.start, hl.end, false))
}

fn highlight_semantic_content_output(pages: &PageList, at: Pin) -> Option<HighlightUntracked> {
    let end = {
        let mut it = PromptIterator::new(Some(at), None, PageListDirection::RightDown);
        let _first = it.next()?;

        if let Some(next) = it.next() {
            let mut prev = next.up(1)?;
            prev.x = unsafe { (*prev.node).data.size.cols } - 1;
            prev
        } else {
            pages.get_bottom_right(PointTag::SCREEN)?
        }
    };

    let mut result: Option<HighlightUntracked> = None;
    let mut it = cell_iterator_at_pin(at, PageListDirection::RightDown, Some(end));
    while let Some(p) = it.next() {
        let (_row, cell) = p.row_and_cell_ptr();
        match unsafe { (*cell).semantic_content() } {
            SemanticContent::Prompt | SemanticContent::Input => {}
            SemanticContent::Output => {
                if !unsafe { (*cell).has_text() } {
                    continue;
                }
                result = Some(HighlightUntracked {
                    start: p,
                    end: p,
                });
                break;
            }
        }
    }
    let mut result = result?;

    while let Some(p) = it.next() {
        let (_row, cell) = p.row_and_cell_ptr();
        match unsafe { (*cell).semantic_content() } {
            SemanticContent::Prompt | SemanticContent::Input => break,
            SemanticContent::Output => {
                if unsafe { (*cell).has_text() } {
                    result.end = p;
                }
            }
        }
    }

    Some(result)
}

impl Screen {
    pub fn select_word(&self, pin: Pin, boundary_codepoints: &[u32]) -> Option<Selection> {
        select_word(pin, boundary_codepoints)
    }

    pub fn select_word_between(
        &self,
        start: Pin,
        end: Pin,
        boundary_codepoints: &[u32],
    ) -> Option<Selection> {
        select_word_between(start, end, boundary_codepoints)
    }

    pub fn select_line(&self, opts: SelectLineOpts<'_>) -> Option<Selection> {
        select_line(self, opts)
    }

    pub fn select_all(&self) -> Option<Selection> {
        select_all(self)
    }

    pub fn select_output(&self, pin: Pin) -> Option<Selection> {
        select_output(self, pin)
    }
}

#[cfg(ghostty_vt_terminal_owned)]
mod ffi {
    use core::ffi::c_int;
    use core::ptr;

    use crate::early::*;
    use crate::selection::GhosttyGridRef;
    use crate::selection_copy::{grid_ref_to_pin, selection_to_ghostty};
    use crate::selection::{selection_write_impl, GhosttySelection};
    use crate::screen_selection::{
        default_line_whitespace_u32, default_word_boundaries_u32, select_all, select_line,
        select_output, select_word, select_word_between, SelectLineOpts,
    };
    use crate::screen_types::Screen;

    unsafe fn codepoint_slice<'a>(
        ptr: *const u32,
        len: usize,
    ) -> Result<Option<&'a [u32]>, c_int> {
        if len == 0 {
            if ptr.is_null() {
                return Ok(None);
            }
            return Ok(Some(&[]));
        }
        if ptr.is_null() {
            return Err(GHOSTTY_INVALID_VALUE);
        }
        Ok(Some(unsafe { core::slice::from_raw_parts(ptr, len) }))
    }

    unsafe fn boundaries_slice<'a>(
        ptr: *const u32,
        len: usize,
    ) -> Result<&'a [u32], c_int> {
        unsafe {
            match codepoint_slice(ptr, len) {
                Ok(None) => Ok(default_word_boundaries_u32()),
                Ok(Some(slice)) => Ok(slice),
                Err(e) => Err(e),
            }
        }
    }

    unsafe fn whitespace_slice<'a>(
        ptr: *const u32,
        len: usize,
    ) -> Result<Option<&'a [u32]>, c_int> {
        unsafe {
            match codepoint_slice(ptr, len) {
                Ok(None) => Ok(Some(default_line_whitespace_u32())),
                Ok(Some(slice)) => Ok(Some(slice)),
                Err(e) => Err(e),
            }
        }
    }

    unsafe fn write_selection(
        sel: Option<crate::selection_types::Selection>,
        out: *mut GhosttySelection,
    ) -> c_int {
        unsafe {
            if out.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            match sel {
                Some(s) => {
                    let ghostty = selection_to_ghostty(&s);
                    selection_write_impl(true, &ghostty, out)
                }
                None => selection_write_impl(false, ptr::null(), out),
            }
        }
    }

    pub(crate) unsafe fn terminal_owned_selection_word_impl(
        screen: *const Screen,
        grid: GhosttyGridRef,
        boundaries_ptr: *const u32,
        boundaries_len: usize,
        out: *mut GhosttySelection,
    ) -> c_int {
        unsafe {
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let pin = match grid_ref_to_pin(grid) {
                Some(p) => p,
                None => return GHOSTTY_INVALID_VALUE,
            };
            let boundaries = match boundaries_slice(boundaries_ptr, boundaries_len) {
                Ok(b) => b,
                Err(e) => return e,
            };
            let sel = select_word(pin, boundaries);
            write_selection(sel, out)
        }
    }

    pub(crate) unsafe fn terminal_owned_selection_word_between_impl(
        screen: *const Screen,
        start: GhosttyGridRef,
        end: GhosttyGridRef,
        boundaries_ptr: *const u32,
        boundaries_len: usize,
        out: *mut GhosttySelection,
    ) -> c_int {
        unsafe {
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let start_pin = match grid_ref_to_pin(start) {
                Some(p) => p,
                None => return GHOSTTY_INVALID_VALUE,
            };
            let end_pin = match grid_ref_to_pin(end) {
                Some(p) => p,
                None => return GHOSTTY_INVALID_VALUE,
            };
            let boundaries = match boundaries_slice(boundaries_ptr, boundaries_len) {
                Ok(b) => b,
                Err(e) => return e,
            };
            let sel = select_word_between(start_pin, end_pin, boundaries);
            write_selection(sel, out)
        }
    }

    pub(crate) unsafe fn terminal_owned_selection_line_impl(
        screen: *const Screen,
        grid: GhosttyGridRef,
        whitespace_ptr: *const u32,
        whitespace_len: usize,
        semantic_prompt_boundary: bool,
        out: *mut GhosttySelection,
    ) -> c_int {
        unsafe {
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let pin = match grid_ref_to_pin(grid) {
                Some(p) => p,
                None => return GHOSTTY_INVALID_VALUE,
            };
            let whitespace = match whitespace_slice(whitespace_ptr, whitespace_len) {
                Ok(w) => w,
                Err(e) => return e,
            };
            let sel = select_line(
                &*screen,
                SelectLineOpts {
                    pin,
                    whitespace,
                    semantic_prompt_boundary,
                },
            );
            write_selection(sel, out)
        }
    }

    pub(crate) unsafe fn terminal_owned_selection_all_impl(
        screen: *const Screen,
        out: *mut GhosttySelection,
    ) -> c_int {
        unsafe {
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let sel = select_all(&*screen);
            write_selection(sel, out)
        }
    }

    pub(crate) unsafe fn terminal_owned_selection_output_impl(
        screen: *const Screen,
        grid: GhosttyGridRef,
        out: *mut GhosttySelection,
    ) -> c_int {
        unsafe {
            if screen.is_null() {
                return GHOSTTY_INVALID_VALUE;
            }
            let pin = match grid_ref_to_pin(grid) {
                Some(p) => p,
                None => return GHOSTTY_INVALID_VALUE,
            };
            let sel = select_output(&*screen, pin);
            write_selection(sel, out)
        }
    }
}

#[cfg(ghostty_vt_terminal_owned)]
pub(crate) use ffi::{
    terminal_owned_selection_all_impl, terminal_owned_selection_line_impl,
    terminal_owned_selection_output_impl, terminal_owned_selection_word_between_impl,
    terminal_owned_selection_word_impl,
};
