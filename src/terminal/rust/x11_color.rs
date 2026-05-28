use crate::style_types::RGB;

const RGB_DATA: &[u8] = include_bytes!("../res/rgb.txt");
const MAX_ENTRIES: usize = 800;

#[derive(Clone, Copy)]
struct Entry {
    name: [u8; 48],
    len: u8,
    r: u8,
    g: u8,
    b: u8,
}

impl Entry {
    const EMPTY: Self = Self {
        name: [0; 48],
        len: 0,
        r: 0,
        g: 0,
        b: 0,
    };
}

const PARSED: ([Entry; MAX_ENTRIES], usize) = parse_and_sort(RGB_DATA);
const ENTRIES: [Entry; MAX_ENTRIES] = PARSED.0;
const COUNT: usize = PARSED.1;

pub fn get(name: &str) -> Option<RGB> {
    let query = name.as_bytes();
    let mut lo = 0usize;
    let mut hi = COUNT;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let e = &ENTRIES[mid];
        let entry_name = crate::bytes_util::subslice_len(&e.name, e.len as usize);
        match cmp_bytes_icase(query, entry_name) {
            core::cmp::Ordering::Equal => return Some(RGB::new(e.r, e.g, e.b)),
            core::cmp::Ordering::Less => hi = mid,
            core::cmp::Ordering::Greater => lo = mid + 1,
        }
    }
    None
}

const fn to_lower(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

const fn cmp_entry_icase(a: &Entry, b: &Entry) -> i32 {
    let len = if a.len < b.len {
        a.len as usize
    } else {
        b.len as usize
    };
    let mut i = 0usize;
    while i < len {
        let ca = to_lower(a.name[i]);
        let cb = to_lower(b.name[i]);
        if ca < cb {
            return -1;
        }
        if ca > cb {
            return 1;
        }
        i += 1;
    }
    (a.len as i32) - (b.len as i32)
}

fn cmp_bytes_icase(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
    let len = core::cmp::min(a.len(), b.len());
    let mut i = 0;
    while i < len {
        let ca = to_lower(a[i]);
        let cb = to_lower(b[i]);
        match ca.cmp(&cb) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        i += 1;
    }
    a.len().cmp(&b.len())
}

const fn parse_u8_trimmed(data: &[u8], start: usize, end: usize) -> u8 {
    let mut i = start;
    while i < end && data[i] == b' ' {
        i += 1;
    }
    let mut e = end;
    while e > i && data[e - 1] == b' ' {
        e -= 1;
    }
    let mut result: u8 = 0;
    while i < e {
        result = result * 10 + (data[i] - b'0');
        i += 1;
    }
    result
}

const fn trim_range(data: &[u8], start: usize, end: usize) -> (usize, usize) {
    let mut s = start;
    while s < end && (data[s] == b' ' || data[s] == b'\t') {
        s += 1;
    }
    let mut e = end;
    while e > s && (data[e - 1] == b' ' || data[e - 1] == b'\t') {
        e -= 1;
    }
    (s, e)
}

const fn parse_and_sort(data: &[u8]) -> ([Entry; MAX_ENTRIES], usize) {
    let mut entries = [Entry::EMPTY; MAX_ENTRIES];
    let mut count = 0usize;
    let data_len = data.len();
    let mut i = 0usize;

    while i < data_len {
        if data[i] == b'\n' {
            i += 1;
            continue;
        }

        let line_start = i;
        while i < data_len && data[i] != b'\n' {
            i += 1;
        }
        let mut line_end = i;
        if i < data_len {
            i += 1;
        }

        if line_end > line_start && data[line_end - 1] == b'\r' {
            line_end -= 1;
        }
        if line_end <= line_start + 12 {
            continue;
        }

        let r = parse_u8_trimmed(data, line_start, line_start + 3);
        let g = parse_u8_trimmed(data, line_start + 4, line_start + 7);
        let b = parse_u8_trimmed(data, line_start + 8, line_start + 11);
        let (ns, ne) = trim_range(data, line_start + 12, line_end);
        let name_len = ne - ns;
        if name_len == 0 || name_len > 48 {
            continue;
        }

        let mut entry = Entry::EMPTY;
        entry.r = r;
        entry.g = g;
        entry.b = b;
        entry.len = name_len as u8;
        let mut j = 0usize;
        while j < name_len {
            entry.name[j] = data[ns + j];
            j += 1;
        }

        entries[count] = entry;
        count += 1;
    }

    let mut si = 1usize;
    while si < count {
        let key = entries[si];
        let mut sj = si;
        while sj > 0 && cmp_entry_icase(&entries[sj - 1], &key) > 0 {
            entries[sj] = entries[sj - 1];
            sj -= 1;
        }
        entries[sj] = key;
        si += 1;
    }

    (entries, count)
}
