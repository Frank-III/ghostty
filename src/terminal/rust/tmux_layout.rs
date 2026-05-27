use core::ptr;
use crate::allocator::*;

pub const LAYOUT_PARSE_ERROR: i32 = -1;
pub const LAYOUT_CHECKSUM_MISMATCH: i32 = -2;

#[repr(C)]
pub struct Layout {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub content: LayoutContent,
}

#[repr(C)]
pub enum LayoutContent {
    Pane(usize),
    Horizontal { ptr: *const Layout, len: usize },
    Vertical { ptr: *const Layout, len: usize },
}

impl Layout {
    pub fn children(&self) -> (*const Layout, usize) {
        match self.content {
            LayoutContent::Pane(_) => (ptr::null(), 0),
            LayoutContent::Horizontal { ptr, len } => (ptr, len),
            LayoutContent::Vertical { ptr, len } => (ptr, len),
        }
    }

    pub fn is_pane(&self) -> bool {
        matches!(self.content, LayoutContent::Pane(_))
    }

    pub fn pane_id(&self) -> Option<usize> {
        match self.content {
            LayoutContent::Pane(id) => Some(id),
            _ => None,
        }
    }

    pub fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        match self.content {
            LayoutContent::Pane(_) => {}
            LayoutContent::Horizontal { ptr, len } |
            LayoutContent::Vertical { ptr, len } => {
                if !ptr.is_null() && len > 0 {
                    let layout_size = core::mem::size_of::<Layout>();
                    let total = layout_size * len;
                    unsafe { alloc_free_impl(alloc, ptr as *mut u8, total); }
                }
            }
        }
    }
}

pub struct Checksum(pub u16);

impl Checksum {
    pub fn calculate(s: &[u8]) -> Checksum {
        let mut result: u16 = 0;
        let mut i: usize = 0;
        while i < s.len() {
            let c = unsafe { *s.get_unchecked(i) };
            result = (result >> 1) | ((result & 1) << 15);
            result = result.wrapping_add(c as u16);
            i += 1;
        }
        Checksum(result)
    }

    pub fn as_string(&self) -> [u8; 4] {
        let v = self.0;
        let charset = b"0123456789abcdef";
        [
            unsafe { *charset.get_unchecked(((v >> 12) & 0xf) as usize) },
            unsafe { *charset.get_unchecked(((v >> 8) & 0xf) as usize) },
            unsafe { *charset.get_unchecked(((v >> 4) & 0xf) as usize) },
            unsafe { *charset.get_unchecked((v & 0xf) as usize) },
        ]
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

fn parse_usize(s: &[u8]) -> Option<usize> {
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

fn index_of_byte(s: &[u8], byte: u8) -> Option<usize> {
    let mut i: usize = 0;
    while i < s.len() {
        if unsafe { *s.get_unchecked(i) } == byte {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn index_of_any(s: &[u8], bytes: &[u8]) -> Option<usize> {
    let mut i: usize = 0;
    while i < s.len() {
        let c = unsafe { *s.get_unchecked(i) };
        let mut j: usize = 0;
        while j < bytes.len() {
            if c == unsafe { *bytes.get_unchecked(j) } {
                return Some(i);
            }
            j += 1;
        }
        i += 1;
    }
    None
}

fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    if s.len() < prefix.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < prefix.len() {
        if unsafe { *s.get_unchecked(i) } != unsafe { *prefix.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

fn alloc_layout_array(
    alloc: *const GhosttyAllocator,
    count: usize,
) -> *mut Layout {
    if count == 0 {
        return ptr::null_mut();
    }
    let layout_size = core::mem::size_of::<Layout>();
    let total = layout_size * count;
    let raw = unsafe { alloc_alloc_impl(alloc, total) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    unsafe { ptr::write_bytes(raw, 0, total); }
    raw as *mut Layout
}

pub fn layout_parse(
    alloc: *const GhosttyAllocator,
    s: &[u8],
    out: *mut Layout,
) -> i32 {
    if out.is_null() {
        return LAYOUT_PARSE_ERROR;
    }
    let mut offset: usize = 0;
    let result = parse_next(alloc, s, &mut offset, out);
    if result < 0 {
        return result;
    }
    if offset != s.len() {
        return LAYOUT_PARSE_ERROR;
    }
    0
}

pub fn layout_parse_with_checksum(
    alloc: *const GhosttyAllocator,
    s: &[u8],
    out: *mut Layout,
) -> i32 {
    if s.len() < 5 {
        return LAYOUT_PARSE_ERROR;
    }
    if unsafe { *s.get_unchecked(4) } != b',' {
        return LAYOUT_PARSE_ERROR;
    }
    let layout_str = unsafe { s.get_unchecked(5..) };
    let checksum = Checksum::calculate(layout_str);
    let checksum_str = checksum.as_string();
    if !starts_with(s, &checksum_str) {
        return LAYOUT_CHECKSUM_MISMATCH;
    }
    layout_parse(alloc, layout_str, out)
}

fn parse_next(
    alloc: *const GhosttyAllocator,
    s: &[u8],
    offset: *mut usize,
    out: *mut Layout,
) -> i32 {
    let off = unsafe { *offset };
    let remaining = unsafe { s.get_unchecked(off..) };

    let x_idx = match index_of_byte(remaining, b'x') {
        Some(idx) => idx,
        None => return LAYOUT_PARSE_ERROR,
    };
    let width = match parse_usize(unsafe { remaining.get_unchecked(..x_idx) }) {
        Some(v) => v,
        None => return LAYOUT_PARSE_ERROR,
    };
    let after_x = x_idx + 1;
    let remaining2 = unsafe { remaining.get_unchecked(after_x..) };

    let comma_idx = match index_of_byte(remaining2, b',') {
        Some(idx) => idx,
        None => return LAYOUT_PARSE_ERROR,
    };
    let height = match parse_usize(unsafe { remaining2.get_unchecked(..comma_idx) }) {
        Some(v) => v,
        None => return LAYOUT_PARSE_ERROR,
    };
    let after_comma = comma_idx + 1;
    let remaining3 = unsafe { remaining2.get_unchecked(after_comma..) };

    let comma_idx2 = match index_of_byte(remaining3, b',') {
        Some(idx) => idx,
        None => return LAYOUT_PARSE_ERROR,
    };
    let x_val = match parse_usize(unsafe { remaining3.get_unchecked(..comma_idx2) }) {
        Some(v) => v,
        None => return LAYOUT_PARSE_ERROR,
    };
    let after_comma2 = comma_idx2 + 1;
    let remaining4 = unsafe { remaining3.get_unchecked(after_comma2..) };

    let delim_idx = match index_of_any(remaining4, b",{[") {
        Some(idx) => idx,
        None => return LAYOUT_PARSE_ERROR,
    };
    let y_val = match parse_usize(unsafe { remaining4.get_unchecked(..delim_idx) }) {
        Some(v) => v,
        None => return LAYOUT_PARSE_ERROR,
    };

    let current_off = off + after_x + after_comma + after_comma2 + delim_idx;
    let delim = unsafe { *s.get_unchecked(current_off) };

    unsafe { *offset = current_off; }

    let content: LayoutContent;

    if delim == b',' {
        unsafe { *offset = current_off + 1; }
        let remaining5 = unsafe { s.get_unchecked(current_off + 1..) };
        let end_idx = index_of_any(remaining5, b",}]").unwrap_or(remaining5.len());
        let pane_id = match parse_usize(unsafe { remaining5.get_unchecked(..end_idx) }) {
            Some(v) => v,
            None => return LAYOUT_PARSE_ERROR,
        };
        unsafe { *offset = current_off + 1 + end_idx; }
        content = LayoutContent::Pane(pane_id);
    } else if delim == b'{' || delim == b'[' {
        let closing = if delim == b'{' { b'}' } else { b']' };
        let mut nodes_count: usize = 0;
        let mut nodes_cap: usize = 4;
        let mut nodes = alloc_layout_array(alloc, nodes_cap);
        if nodes.is_null() {
            return LAYOUT_PARSE_ERROR;
        }

        unsafe { *offset = current_off + 1; }

        loop {
            if nodes_count >= nodes_cap {
                let new_cap = nodes_cap * 2;
                let new_nodes = alloc_layout_array(alloc, new_cap);
                if new_nodes.is_null() {
                    let old_total = core::mem::size_of::<Layout>() * nodes_cap;
                    unsafe { alloc_free_impl(alloc, nodes as *mut u8, old_total); }
                    return LAYOUT_PARSE_ERROR;
                }
                let old_total = core::mem::size_of::<Layout>() * nodes_count;
                unsafe {
                    ptr::copy_nonoverlapping(
                        nodes as *const u8,
                        new_nodes as *mut u8,
                        old_total,
                    );
                    alloc_free_impl(
                        alloc,
                        nodes as *mut u8,
                        core::mem::size_of::<Layout>() * nodes_cap,
                    );
                }
                nodes = new_nodes;
                nodes_cap = new_cap;
            }

            let node_ptr = unsafe { nodes.add(nodes_count) };
            let result = parse_next(alloc, s, offset, node_ptr);
            if result < 0 {
                let total = core::mem::size_of::<Layout>() * nodes_cap;
                unsafe { alloc_free_impl(alloc, nodes as *mut u8, total); }
                return result;
            }
            nodes_count += 1;

            let cur_off = unsafe { *offset };
            if cur_off >= s.len() {
                let total = core::mem::size_of::<Layout>() * nodes_cap;
                unsafe { alloc_free_impl(alloc, nodes as *mut u8, total); }
                return LAYOUT_PARSE_ERROR;
            }

            let next_byte = unsafe { *s.get_unchecked(cur_off) };
            if next_byte == b',' {
                unsafe { *offset = cur_off + 1; }
                continue;
            }

            if next_byte != closing {
                let total = core::mem::size_of::<Layout>() * nodes_cap;
                unsafe { alloc_free_impl(alloc, nodes as *mut u8, total); }
                return LAYOUT_PARSE_ERROR;
            }

            unsafe { *offset = cur_off + 1; }

            if nodes_count < nodes_cap {
                let final_nodes = alloc_layout_array(alloc, nodes_count);
                if final_nodes.is_null() && nodes_count > 0 {
                    let total = core::mem::size_of::<Layout>() * nodes_cap;
                    unsafe { alloc_free_impl(alloc, nodes as *mut u8, total); }
                    return LAYOUT_PARSE_ERROR;
                }
                if !final_nodes.is_null() {
                    let copy_size = core::mem::size_of::<Layout>() * nodes_count;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            nodes as *const u8,
                            final_nodes as *mut u8,
                            copy_size,
                        );
                        alloc_free_impl(
                            alloc,
                            nodes as *mut u8,
                            core::mem::size_of::<Layout>() * nodes_cap,
                        );
                    }
                    nodes = final_nodes;
                }
            }

            content = if delim == b'{' {
                LayoutContent::Horizontal { ptr: nodes, len: nodes_count }
            } else {
                LayoutContent::Vertical { ptr: nodes, len: nodes_count }
            };
            break;
        }
    } else {
        return LAYOUT_PARSE_ERROR;
    }

    unsafe {
        (*out).width = width;
        (*out).height = height;
        (*out).x = x_val;
        (*out).y = y_val;
        (*out).content = content;
    }

    0
}

pub fn layout_free(layout: *mut Layout, alloc: *const GhosttyAllocator) {
    if layout.is_null() {
        return;
    }
    layout_deinit(layout, alloc);
}

fn layout_deinit(layout: *mut Layout, alloc: *const GhosttyAllocator) {
    if layout.is_null() {
        return;
    }
    unsafe {
        match (*layout).content {
            LayoutContent::Pane(_) => {}
            LayoutContent::Horizontal { ptr, len } |
            LayoutContent::Vertical { ptr, len } => {
                if !ptr.is_null() && len > 0 {
                    let mut i: usize = 0;
                    while i < len {
                        let child = (ptr as *mut Layout).add(i);
                        layout_deinit(child, alloc);
                        i += 1;
                    }
                    let total = core::mem::size_of::<Layout>() * len;
                    alloc_free_impl(alloc, ptr as *mut u8, total);
                }
            }
        }
    }
}

pub fn layout_walk_panes(
    layout: *const Layout,
    ctx: *mut core::ffi::c_void,
    callback: unsafe extern "C" fn(usize, *mut core::ffi::c_void),
) {
    if layout.is_null() {
        return;
    }
    unsafe {
        match (*layout).content {
            LayoutContent::Pane(id) => {
                callback(id, ctx);
            }
            LayoutContent::Horizontal { ptr, len } |
            LayoutContent::Vertical { ptr, len } => {
                let mut i: usize = 0;
                while i < len {
                    let child = ptr.add(i);
                    layout_walk_panes(child, ctx, callback);
                    i += 1;
                }
            }
        }
    }
}
