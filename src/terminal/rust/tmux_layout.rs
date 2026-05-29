use crate::allocator::*;
use core::ptr;

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
            LayoutContent::Horizontal { ptr, len } | LayoutContent::Vertical { ptr, len } => {
                if !ptr.is_null() && len > 0 {
                    let layout_size = core::mem::size_of::<Layout>();
                    let total = layout_size * len;
                    unsafe {
                        alloc_free_impl(alloc, ptr as *mut u8, total);
                    }
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

fn alloc_layout_array(alloc: *const GhosttyAllocator, count: usize) -> *mut Layout {
    if count == 0 {
        return ptr::null_mut();
    }
    let layout_size = core::mem::size_of::<Layout>();
    let total = layout_size * count;
    let raw = unsafe { alloc_alloc_impl(alloc, total) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        ptr::write_bytes(raw, 0, total);
    }
    raw as *mut Layout
}

pub fn layout_parse(alloc: *const GhosttyAllocator, s: &[u8], out: *mut Layout) -> i32 {
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

    unsafe {
        *offset = current_off;
    }

    let content: LayoutContent;

    if delim == b',' {
        unsafe {
            *offset = current_off + 1;
        }
        let remaining5 = unsafe { s.get_unchecked(current_off + 1..) };
        let end_idx = index_of_any(remaining5, b",}]").unwrap_or(remaining5.len());
        let pane_id = match parse_usize(unsafe { remaining5.get_unchecked(..end_idx) }) {
            Some(v) => v,
            None => return LAYOUT_PARSE_ERROR,
        };
        unsafe {
            *offset = current_off + 1 + end_idx;
        }
        content = LayoutContent::Pane(pane_id);
    } else if delim == b'{' || delim == b'[' {
        let closing = if delim == b'{' { b'}' } else { b']' };
        let mut nodes_count: usize = 0;
        let mut nodes_cap: usize = 4;
        let mut nodes = alloc_layout_array(alloc, nodes_cap);
        if nodes.is_null() {
            return LAYOUT_PARSE_ERROR;
        }

        unsafe {
            *offset = current_off + 1;
        }

        loop {
            if nodes_count >= nodes_cap {
                let new_cap = nodes_cap * 2;
                let new_nodes = alloc_layout_array(alloc, new_cap);
                if new_nodes.is_null() {
                    let old_total = core::mem::size_of::<Layout>() * nodes_cap;
                    unsafe {
                        alloc_free_impl(alloc, nodes as *mut u8, old_total);
                    }
                    return LAYOUT_PARSE_ERROR;
                }
                let old_total = core::mem::size_of::<Layout>() * nodes_count;
                unsafe {
                    ptr::copy_nonoverlapping(nodes as *const u8, new_nodes as *mut u8, old_total);
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
                unsafe {
                    alloc_free_impl(alloc, nodes as *mut u8, total);
                }
                return result;
            }
            nodes_count += 1;

            let cur_off = unsafe { *offset };
            if cur_off >= s.len() {
                let total = core::mem::size_of::<Layout>() * nodes_cap;
                unsafe {
                    alloc_free_impl(alloc, nodes as *mut u8, total);
                }
                return LAYOUT_PARSE_ERROR;
            }

            let next_byte = unsafe { *s.get_unchecked(cur_off) };
            if next_byte == b',' {
                unsafe {
                    *offset = cur_off + 1;
                }
                continue;
            }

            if next_byte != closing {
                let total = core::mem::size_of::<Layout>() * nodes_cap;
                unsafe {
                    alloc_free_impl(alloc, nodes as *mut u8, total);
                }
                return LAYOUT_PARSE_ERROR;
            }

            unsafe {
                *offset = cur_off + 1;
            }

            if nodes_count < nodes_cap {
                let final_nodes = alloc_layout_array(alloc, nodes_count);
                if final_nodes.is_null() && nodes_count > 0 {
                    let total = core::mem::size_of::<Layout>() * nodes_cap;
                    unsafe {
                        alloc_free_impl(alloc, nodes as *mut u8, total);
                    }
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
                LayoutContent::Horizontal {
                    ptr: nodes,
                    len: nodes_count,
                }
            } else {
                LayoutContent::Vertical {
                    ptr: nodes,
                    len: nodes_count,
                }
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
            LayoutContent::Horizontal { ptr, len } | LayoutContent::Vertical { ptr, len } => {
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
            LayoutContent::Horizontal { ptr, len } | LayoutContent::Vertical { ptr, len } => {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_layout_eq(layout: &Layout, width: usize, height: usize, x: usize, y: usize) {
        assert_eq!(layout.width, width);
        assert_eq!(layout.height, height);
        assert_eq!(layout.x, x);
        assert_eq!(layout.y, y);
    }

    // -- Checksum tests --

    #[test]
    fn checksum_empty_string() {
        let cs = Checksum::calculate(b"");
        assert_eq!(cs.value(), 0);
        assert_eq!(&cs.as_string(), b"0000");
    }

    #[test]
    fn checksum_single_character() {
        let cs = Checksum::calculate(b"A");
        assert_eq!(cs.value(), 65);
        assert_eq!(&cs.as_string(), b"0041");
    }

    #[test]
    fn checksum_two_characters() {
        let cs = Checksum::calculate(b"AB");
        assert_eq!(cs.value(), 32866);
        assert_eq!(&cs.as_string(), b"8062");
    }

    #[test]
    fn checksum_simple_layout() {
        let cs = Checksum::calculate(b"80x24,0,0,42");
        assert_eq!(&cs.as_string(), b"d962");
    }

    #[test]
    fn checksum_horizontal_split_layout() {
        let cs = Checksum::calculate(b"80x24,0,0{40x24,0,0,1,40x24,40,0,2}");
        assert_eq!(&cs.as_string(), b"f8f9");
    }

    #[test]
    fn checksum_as_string_zero_padding() {
        let cs = Checksum(0x000f);
        assert_eq!(&cs.as_string(), b"000f");
    }

    #[test]
    fn checksum_as_string_all_digits() {
        let cs = Checksum(0x1234);
        assert_eq!(&cs.as_string(), b"1234");
    }

    #[test]
    fn checksum_as_string_with_letters() {
        let cs = Checksum(0xabcd);
        assert_eq!(&cs.as_string(), b"abcd");
    }

    #[test]
    fn checksum_as_string_max_value() {
        let cs = Checksum(0xffff);
        assert_eq!(&cs.as_string(), b"ffff");
    }

    #[test]
    fn checksum_wraparound() {
        let cs = Checksum::calculate(b"\xff\xff\xff\xff\xff\xff\xff\xff");
        assert_eq!(&cs.as_string(), b"03fc");
    }

    #[test]
    fn checksum_deterministic() {
        let s = b"159x48,0,0{79x48,0,0,79x48,80,0}";
        let cs1 = Checksum::calculate(s);
        let cs2 = Checksum::calculate(s);
        assert_eq!(cs1.value(), cs2.value());
    }

    #[test]
    fn checksum_different_inputs_different_outputs() {
        let cs1 = Checksum::calculate(b"80x24,0,0,1");
        let cs2 = Checksum::calculate(b"80x24,0,0,2");
        assert_ne!(cs1.value(), cs2.value());
    }

    #[test]
    fn checksum_known_tmux_layout_bb62() {
        let cs = Checksum::calculate(b"159x48,0,0{79x48,0,0,79x48,80,0}");
        assert_eq!(&cs.as_string(), b"bb62");
    }

    // -- Layout parse tests (pure computation, no allocator needed for syntax error checks) --

    #[test]
    fn syntax_error_empty_string() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_missing_width() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"x24,0,0,1", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_missing_height() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x,0,0,1", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_missing_x() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x24,,0,1", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_missing_y() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x24,0,,1", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_missing_pane_id() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x24,0,0,", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_non_numeric_width() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"abcx24,0,0,1", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_non_numeric_pane_id() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x24,0,0,abc", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_trailing_data() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x24,0,0,1extra", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_no_x_separator() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"8024,0,0,1", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn syntax_error_no_content_delimiter() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse(ptr::null(), b"80x24,0,0", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn layout_parse_with_checksum_too_short() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse_with_checksum(ptr::null(), b"bb62", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
        let r2 = layout_parse_with_checksum(ptr::null(), b"", out.as_mut_ptr());
        assert_eq!(r2, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn layout_parse_with_checksum_missing_comma() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse_with_checksum(ptr::null(), b"bb62x159x48,0,0", out.as_mut_ptr());
        assert_eq!(r, LAYOUT_PARSE_ERROR);
    }

    #[test]
    fn layout_parse_with_checksum_mismatch() {
        let mut out = core::mem::MaybeUninit::<Layout>::uninit();
        let r = layout_parse_with_checksum(
            ptr::null(),
            b"0000,80x24,0,0{40x24,0,0,1,40x24,40,0,2}",
            out.as_mut_ptr(),
        );
        assert_eq!(r, LAYOUT_CHECKSUM_MISMATCH);
    }

    #[test]
    fn layout_is_pane_and_pane_id() {
        let layout = Layout {
            width: 80,
            height: 24,
            x: 0,
            y: 0,
            content: LayoutContent::Pane(42),
        };
        assert!(layout.is_pane());
        assert_eq!(layout.pane_id(), Some(42));
        let (children, len) = layout.children();
        assert!(children.is_null());
        assert_eq!(len, 0);
    }

    #[test]
    fn layout_children_returns_ptr_and_len() {
        let children_arr: [Layout; 2] = [
            Layout {
                width: 40,
                height: 24,
                x: 0,
                y: 0,
                content: LayoutContent::Pane(1),
            },
            Layout {
                width: 40,
                height: 24,
                x: 40,
                y: 0,
                content: LayoutContent::Pane(2),
            },
        ];
        let layout = Layout {
            width: 80,
            height: 24,
            x: 0,
            y: 0,
            content: LayoutContent::Horizontal {
                ptr: children_arr.as_ptr(),
                len: 2,
            },
        };
        assert!(!layout.is_pane());
        assert_eq!(layout.pane_id(), None);
        let (ptr, len) = layout.children();
        assert!(!ptr.is_null());
        assert_eq!(len, 2);
    }

    #[cfg(all(test, feature = "std"))]
    mod alloc_tests {
        use super::*;

        unsafe extern "C" fn test_alloc(
            _ctx: *mut core::ffi::c_void,
            len: usize,
            _align: u8,
            _ra: usize,
        ) -> *mut u8 {
            if len == 0 {
                return core::ptr::NonNull::<u8>::dangling().as_ptr();
            }
            let layout = std::alloc::Layout::from_size_align(len, 1).unwrap();
            std::alloc::alloc(layout)
        }

        unsafe extern "C" fn test_resize(
            _ctx: *mut core::ffi::c_void,
            _ptr: *mut u8,
            _old_len: usize,
            _align: u8,
            _new_len: usize,
            _ra: usize,
        ) -> bool {
            false
        }

        unsafe extern "C" fn test_remap(
            _ctx: *mut core::ffi::c_void,
            ptr: *mut u8,
            old_len: usize,
            _align: u8,
            new_len: usize,
            _ra: usize,
        ) -> *mut u8 {
            if ptr.is_null() || old_len == 0 {
                let layout = std::alloc::Layout::from_size_align(new_len, 1).unwrap();
                return std::alloc::alloc(layout);
            }
            if new_len == 0 {
                let layout = std::alloc::Layout::from_size_align(old_len, 1).unwrap();
                std::alloc::dealloc(ptr, layout);
                return core::ptr::NonNull::<u8>::dangling().as_ptr();
            }
            let layout = std::alloc::Layout::from_size_align(old_len, 1).unwrap();
            std::alloc::realloc(ptr, layout, new_len)
        }

        unsafe extern "C" fn test_free(
            _ctx: *mut core::ffi::c_void,
            ptr: *mut u8,
            len: usize,
            _align: u8,
            _ra: usize,
        ) {
            if !ptr.is_null() && len > 0 {
                let layout = std::alloc::Layout::from_size_align(len, 1).unwrap();
                std::alloc::dealloc(ptr, layout);
            }
        }

        static TEST_VTABLE: GhosttyAllocatorVtable = GhosttyAllocatorVtable {
            alloc: test_alloc,
            resize: test_resize,
            remap: test_remap,
            free: test_free,
        };

        fn test_allocator() -> GhosttyAllocator {
            GhosttyAllocator {
                ctx: ptr::null_mut(),
                vtable: &TEST_VTABLE,
            }
        }

        #[test]
        fn single_pane() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(&alloc, b"80x24,0,0,42", out.as_mut_ptr());
            assert_eq!(r, 0);
            let layout = unsafe { out.assume_init() };
            assert_layout_eq(&layout, 80, 24, 0, 0);
            assert_eq!(layout.pane_id(), Some(42));
        }

        #[test]
        fn horizontal_split() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(
                &alloc,
                b"80x24,0,0{40x24,0,0,1,40x24,40,0,2}",
                out.as_mut_ptr(),
            );
            assert_eq!(r, 0);
            let mut layout = unsafe { out.assume_init() };
            assert_layout_eq(&layout, 80, 24, 0, 0);
            let (ptr, len) = layout.children();
            assert_eq!(len, 2);
            unsafe {
                assert_eq!((*ptr.add(0)).pane_id(), Some(1));
                assert_eq!((*ptr.add(1)).pane_id(), Some(2));
            }
            layout_deinit(&mut layout as *mut _, &alloc);
        }

        #[test]
        fn vertical_split() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(
                &alloc,
                b"80x24,0,0[80x12,0,0,1,80x12,0,12,2]",
                out.as_mut_ptr(),
            );
            assert_eq!(r, 0);
            let mut layout = unsafe { out.assume_init() };
            assert_layout_eq(&layout, 80, 24, 0, 0);
            let (ptr, len) = layout.children();
            assert_eq!(len, 2);
            unsafe {
                assert_eq!((*ptr.add(0)).pane_id(), Some(1));
                assert_eq!((*ptr.add(1)).pane_id(), Some(2));
            }
            layout_deinit(&mut layout as *mut _, &alloc);
        }

        #[test]
        fn mixed_horizontal_and_vertical() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(
                &alloc,
                b"80x24,0,0{40x24,0,0[40x12,0,0,1,40x12,0,12,2],40x24,40,0,3}",
                out.as_mut_ptr(),
            );
            assert_eq!(r, 0);
            let mut layout = unsafe { out.assume_init() };
            assert_layout_eq(&layout, 80, 24, 0, 0);
            let (hptr, hlen) = layout.children();
            assert_eq!(hlen, 2);
            unsafe {
                let first = &*hptr.add(0);
                let (vptr, vlen) = first.children();
                assert_eq!(vlen, 2);
                assert_eq!((*vptr.add(0)).pane_id(), Some(1));
                assert_eq!((*vptr.add(1)).pane_id(), Some(2));
                assert_eq!((*hptr.add(1)).pane_id(), Some(3));
            }
            layout_deinit(&mut layout as *mut _, &alloc);
        }

        #[test]
        fn deeply_nested_layout() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(
                &alloc,
                b"80x24,0,0{40x24,0,0[40x12,0,0,1,40x12,0,12,2],40x24,40,0,3}",
                out.as_mut_ptr(),
            );
            assert_eq!(r, 0);
            let mut layout = unsafe { out.assume_init() };
            let (hptr, hlen) = layout.children();
            assert_eq!(hlen, 2);
            unsafe {
                let vert_child = &*hptr.add(0);
                let (vptr, vlen) = vert_child.children();
                assert_eq!(vlen, 2);
                assert_eq!((*vptr.add(0)).pane_id(), Some(1));
                assert_eq!((*vptr.add(1)).pane_id(), Some(2));
                assert_eq!((*hptr.add(1)).pane_id(), Some(3));
            }
            layout_deinit(&mut layout as *mut _, &alloc);
        }

        #[test]
        fn syntax_error_unclosed_horizontal_bracket() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(&alloc, b"80x24,0,0{40x24,0,0,1", out.as_mut_ptr());
            assert_eq!(r, LAYOUT_PARSE_ERROR);
        }

        #[test]
        fn syntax_error_unclosed_vertical_bracket() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(&alloc, b"80x24,0,0[40x24,0,0,1", out.as_mut_ptr());
            assert_eq!(r, LAYOUT_PARSE_ERROR);
        }

        #[test]
        fn syntax_error_mismatched_brackets() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(&alloc, b"80x24,0,0{40x24,0,0,1]", out.as_mut_ptr());
            assert_eq!(r, LAYOUT_PARSE_ERROR);
            let r2 = layout_parse(&alloc, b"80x24,0,0[40x24,0,0,1}", out.as_mut_ptr());
            assert_eq!(r2, LAYOUT_PARSE_ERROR);
        }

        #[test]
        fn parse_with_checksum_valid() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse_with_checksum(
                &alloc,
                b"f8f9,80x24,0,0{40x24,0,0,1,40x24,40,0,2}",
                out.as_mut_ptr(),
            );
            assert_eq!(r, 0);
            let mut layout = unsafe { out.assume_init() };
            assert_layout_eq(&layout, 80, 24, 0, 0);
            layout_deinit(&mut layout as *mut _, &alloc);
        }

        extern "C" fn walk_panes_cb(id: usize, ctx: *mut core::ffi::c_void) {
            unsafe {
                let ids = &mut *(ctx as *mut Vec<usize>);
                ids.push(id);
            }
        }

        #[test]
        fn layout_walk_panes_collects_ids() {
            let alloc = test_allocator();
            let mut out = core::mem::MaybeUninit::<Layout>::uninit();
            let r = layout_parse(
                &alloc,
                b"80x24,0,0{40x24,0,0,1,40x24,40,0,2}",
                out.as_mut_ptr(),
            );
            assert_eq!(r, 0);
            let mut layout = unsafe { out.assume_init() };
            let mut ids: Vec<usize> = Vec::new();
            let ids_ptr = &mut ids as *mut Vec<usize> as *mut core::ffi::c_void;
            unsafe {
                layout_walk_panes(&layout, ids_ptr, walk_panes_cb);
            }
            assert_eq!(ids, vec![1, 2]);
            layout_deinit(&mut layout as *mut _, &alloc);
        }
    }
}
