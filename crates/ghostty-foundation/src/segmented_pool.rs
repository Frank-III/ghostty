//! Segmented pool ported from `src/datastruct/segmented_pool.zig`.
//!
//! Returns stable indices into pool storage; callers map indices to values
//! through [`SegmentedPool::get_mut`].

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    OutOfValues,
}

/// Stable-index pool for ordered get/put cycles.
pub struct SegmentedPool<T, const PREALLOC: usize> {
    storage: Vec<T>,
    i: usize,
    available: usize,
}

impl<T, const PREALLOC: usize> SegmentedPool<T, PREALLOC> {
    pub fn new() -> Self
    where
        T: Default,
    {
        Self {
            storage: (0..PREALLOC).map(|_| T::default()).collect(),
            i: 0,
            available: PREALLOC,
        }
    }

    pub fn available(&self) -> usize {
        self.available
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn get_index(&mut self) -> Result<PoolIndex, PoolError> {
        if self.available == 0 {
            return Err(PoolError::OutOfValues);
        }
        let idx = self.i % self.storage.len();
        self.i = self.i.wrapping_add(1);
        self.available -= 1;
        Ok(PoolIndex(idx))
    }

    pub fn get_mut(&mut self, index: PoolIndex) -> &mut T {
        &mut self.storage[index.0]
    }

    pub fn get_index_grow(&mut self, grow: impl Fn() -> T) -> Result<PoolIndex, PoolError> {
        if self.available == 0 {
            self.grow(grow);
        }
        self.get_index()
    }

    pub fn put(&mut self) {
        debug_assert!(self.available < self.storage.len());
        self.available += 1;
    }

    fn grow(&mut self, grow: impl Fn() -> T) {
        let old_len = self.storage.len();
        let new_len = old_len * 2;
        self.storage.reserve(new_len - old_len);
        for _ in old_len..new_len {
            self.storage.push(grow());
        }
        self.i = old_len;
        self.available = old_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmented_pool_get_put_and_grow() {
        let mut pool = SegmentedPool::<u8, 2>::new();
        assert_eq!(pool.available(), 2);

        let i1 = pool.get_index().unwrap();
        let i2 = pool.get_index().unwrap();
        assert_ne!(i1, i2);
        assert_eq!(pool.get_index(), Err(PoolError::OutOfValues));

        *pool.get_mut(i1) = 42;
        pool.put();
        let temp = pool.get_index().unwrap();
        assert_eq!(i1, temp);
        assert_eq!(*pool.get_mut(temp), 42);
        assert_eq!(pool.get_index(), Err(PoolError::OutOfValues));

        let i3 = pool.get_index_grow(Default::default).unwrap();
        assert_ne!(i1, i3);
        assert_ne!(i2, i3);
        let _ = pool.get_index().unwrap();
        assert_eq!(pool.get_index(), Err(PoolError::OutOfValues));

        pool.put();
        assert_eq!(pool.get_index().unwrap(), i1);
        assert_eq!(pool.get_index(), Err(PoolError::OutOfValues));
    }
}
