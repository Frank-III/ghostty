//! Circular buffer ported from `src/datastruct/circ_buf.zig`.

use std::vec::Vec;

/// Two contiguous slices returned by [`CircBuf::get_ptr_slice`].
pub type PtrSlicePair<'a, T> = (&'a mut [T], &'a mut [T]);

/// Fixed-capacity circular buffer with a configurable default fill value.
pub struct CircBuf<T: Copy> {
    storage: Vec<T>,
    head: usize,
    tail: usize,
    full: bool,
    default: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reverse,
}

pub struct CircBufIter<'a, T: Copy> {
    buf: &'a CircBuf<T>,
    idx: usize,
    direction: Direction,
}

impl<T: Copy> CircBuf<T> {
    pub fn new(capacity: usize, default: T) -> Self {
        let mut storage = Vec::with_capacity(capacity);
        storage.resize(capacity, default);
        Self {
            storage,
            head: 0,
            tail: 0,
            full: capacity == 0,
            default,
        }
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.full = false;
    }

    pub fn append(&mut self, value: T) -> Result<(), ()> {
        if self.full {
            return Err(());
        }
        self.storage[self.head] = value;
        self.head += 1;
        if self.head >= self.storage.len() {
            self.head = 0;
        }
        self.full = self.head == self.tail;
        Ok(())
    }

    pub fn append_assume_capacity(&mut self, value: T) {
        debug_assert!(!self.full);
        self.storage[self.head] = value;
        self.head += 1;
        if self.head >= self.storage.len() {
            self.head = 0;
        }
        self.full = self.head == self.tail;
    }

    pub fn append_slice_assume_capacity(&mut self, slice: &[T]) {
        let (first, second) = self.get_ptr_slice(self.len(), slice.len());
        let split = first.len();
        first.copy_from_slice(&slice[..split]);
        second.copy_from_slice(&slice[split..]);
    }

    pub fn delete_oldest(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        debug_assert!(n <= self.storage.len());

        let default = self.default;
        let (first, second) = self.get_ptr_slice(0, n);
        for item in first.iter_mut().chain(second.iter_mut()) {
            *item = default;
        }

        let delete = n.min(self.len());
        self.tail += delete;
        if self.tail >= self.storage.len() {
            self.tail -= self.storage.len();
        }
        self.full = false;
    }

    pub fn empty(&self) -> bool {
        !self.full && self.head == self.tail
    }

    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    pub fn len(&self) -> usize {
        if self.full {
            return self.storage.len();
        }
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            self.storage.len() - (self.tail - self.head)
        }
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn resize(&mut self, size: usize) {
        self.rotate_to_zero();
        let prev_len = self.len();
        let prev_cap = self.storage.len();
        self.storage.resize(size, self.default);
        if size > prev_cap {
            if self.full {
                self.head = prev_len;
                self.full = false;
            }
        }
    }

    pub fn iterator(&self, direction: Direction) -> CircBufIter<'_, T> {
        CircBufIter {
            buf: self,
            idx: 0,
            direction,
        }
    }

    pub fn first(&self) -> Option<T> {
        self.iterator(Direction::Forward).next()
    }

    pub fn last(&self) -> Option<T> {
        self.iterator(Direction::Reverse).next()
    }

    pub fn get_ptr_slice(&mut self, offset: usize, slice_len: usize) -> PtrSlicePair<'_, T> {
        if slice_len == 0 {
            return (&mut [], &mut []);
        }
        debug_assert!(offset + slice_len <= self.capacity());

        let end_offset = offset + slice_len;
        if end_offset > self.len() {
            self.advance(end_offset - self.len());
        }

        let start_idx = self.storage_offset(offset);
        let end_idx = self.storage_offset(end_offset - 1);

        if end_idx >= start_idx {
            let slice = &mut self.storage[start_idx..=end_idx];
            (slice, &mut [])
        } else {
            let ptr = self.storage.as_mut_ptr();
            let cap = self.storage.len();
            unsafe {
                let first = std::slice::from_raw_parts_mut(ptr.add(start_idx), cap - start_idx);
                let second = std::slice::from_raw_parts_mut(ptr, end_idx + 1);
                (first, second)
            }
        }
    }

    fn advance(&mut self, amount: usize) {
        debug_assert!(amount <= self.storage.len() - self.len());
        self.head += amount;
        if self.head >= self.storage.len() {
            self.head -= self.storage.len();
        }
        if self.full {
            self.tail = self.head;
        }
        self.full = self.head == self.tail;
    }

    fn storage_offset(&self, offset: usize) -> usize {
        debug_assert!(offset < self.storage.len());
        let fits_offset = self.tail + offset;
        if fits_offset < self.storage.len() {
            fits_offset
        } else {
            fits_offset - self.storage.len()
        }
    }

    fn rotate_to_zero(&mut self) {
        if self.tail == 0 {
            return;
        }
        self.storage.rotate_left(self.tail);
        self.head = self.len() % self.storage.len();
        self.tail = 0;
    }
}

impl<'a, T: Copy> Iterator for CircBufIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.buf.len() {
            return None;
        }
        let tail_idx = match self.direction {
            Direction::Forward => self.idx,
            Direction::Reverse => self.buf.len() - self.idx - 1,
        };
        let storage_idx = (self.buf.tail + tail_idx) % self.buf.capacity();
        self.idx += 1;
        Some(self.buf.storage[storage_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_iterate() {
        let mut buf = CircBuf::new(3, 0u8);
        assert!(buf.empty());
        buf.append(1).unwrap();
        buf.append(2).unwrap();
        buf.append(3).unwrap();
        assert!(buf.append(4).is_err());

        buf.delete_oldest(1);
        buf.append(4).unwrap();

        let values: Vec<_> = buf.iterator(Direction::Forward).collect();
        assert_eq!(values, vec![2, 3, 4]);
    }

    #[test]
    fn append_slice_with_wrap() {
        let mut buf = CircBuf::new(4, 0u8);
        {
            let _ = buf.get_ptr_slice(0, buf.capacity());
            assert!(buf.is_full());
        }
        buf.delete_oldest(2);
        buf.append_slice_assume_capacity(b"AB");
        let values: Vec<_> = buf.iterator(Direction::Forward).collect();
        assert_eq!(values, vec![0, 0, b'A', b'B']);
    }

    #[test]
    fn get_ptr_slice_wraps() {
        let mut buf = CircBuf::new(4, 0u8);
        {
            let _ = buf.get_ptr_slice(0, 3);
            assert_eq!(buf.len(), 3);
        }
        buf.delete_oldest(2);
        {
            let (first, second) = buf.get_ptr_slice(0, 4);
            assert_eq!(first.len(), 2);
            assert_eq!(second.len(), 2);
            first[0] = 1;
            first[1] = 2;
            second[0] = 3;
            second[1] = 4;
        }
        let values: Vec<_> = buf.iterator(Direction::Forward).collect();
        assert_eq!(values, vec![1, 2, 3, 4]);
    }
}
