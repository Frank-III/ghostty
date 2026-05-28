//! Fixed-bucket LRU cache ported from `src/datastruct/cache_table.zig`.

/// Cache key lookup and comparison.
pub trait CacheContext<K, V> {
    fn hash(&self, key: &K) -> u64;
    fn eql(&self, a: &K, b: &K) -> bool;
    fn evicted(&mut self, _key: K, _value: V) {}
}

/// Evicted key/value pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry<K, V> {
    pub key: K,
    pub value: V,
}

/// Fixed-bucket associative cache with LRU eviction within each bucket.
pub struct CacheTable<K, V, C, const BUCKET_COUNT: usize, const BUCKET_SIZE: usize> {
    buckets: [[Option<(K, V)>; BUCKET_SIZE]; BUCKET_COUNT],
    lengths: [u8; BUCKET_COUNT],
    context: C,
}

impl<K, V, C, const BUCKET_COUNT: usize, const BUCKET_SIZE: usize> CacheTable<K, V, C, BUCKET_COUNT, BUCKET_SIZE>
where
    K: Clone,
    V: Clone,
    C: CacheContext<K, V>,
{
    pub fn new(context: C) -> Self {
        assert!(BUCKET_COUNT.is_power_of_two(), "bucket_count must be a power of two");
        assert!(BUCKET_SIZE > 0, "bucket_size must be non-zero");
        Self {
            buckets: core::array::from_fn(|_| core::array::from_fn(|_| None)),
            lengths: [0; BUCKET_COUNT],
            context,
        }
    }

    pub fn put(&mut self, key: K, value: V) -> Option<CacheEntry<K, V>> {
        let idx = (self.context.hash(&key) as usize) & (BUCKET_COUNT - 1);
        let len = self.lengths[idx] as usize;

        if len < BUCKET_SIZE {
            self.buckets[idx][len] = Some((key, value));
            self.lengths[idx] += 1;
            return None;
        }

        debug_assert_eq!(len, BUCKET_SIZE);
        let mut entries: Vec<(K, V)> = self.buckets[idx]
            .iter()
            .map(|entry| entry.clone().expect("full bucket slot"))
            .collect();
        let evicted = rotate_in(&mut entries, (key, value));
        for (slot, entry) in self.buckets[idx].iter_mut().zip(entries) {
            *slot = Some(entry);
        }
        self.context.evicted(evicted.key.clone(), evicted.value.clone());
        Some(evicted)
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let idx = (self.context.hash(key) as usize) & (BUCKET_COUNT - 1);
        let len = self.lengths[idx] as usize;
        let mut i = len;
        while i > 0 {
            i -= 1;
            let Some((ref slot_key, ref value)) = self.buckets[idx][i] else {
                continue;
            };
            if self.context.eql(key, slot_key) {
                let value = value.clone();
                let mut slice: Vec<Option<(K, V)>> = self.buckets[idx][i..len]
                    .iter()
                    .cloned()
                    .collect();
                rotate_once(&mut slice);
                for (slot, entry) in self.buckets[idx][i..len].iter_mut().zip(slice) {
                    *slot = entry;
                }
                return Some(value);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        for (bucket, len) in self.buckets.iter_mut().zip(self.lengths.iter()) {
            for entry in bucket.iter_mut().take(*len as usize) {
                if let Some((key, value)) = entry.take() {
                    self.context.evicted(key, value);
                }
            }
        }
        self.lengths = [0; BUCKET_COUNT];
    }
}

fn rotate_once<T>(items: &mut [Option<T>]) {
    if items.len() <= 1 {
        return;
    }
    let tmp = items[0].take();
    for i in 0..items.len() - 1 {
        items[i] = items[i + 1].take();
    }
    items[items.len() - 1] = tmp;
}

fn rotate_in<K: Clone, V: Clone>(items: &mut [(K, V)], item: (K, V)) -> CacheEntry<K, V> {
    let (key, value) = items[0].clone();
    for i in 0..items.len() - 1 {
        items[i] = items[i + 1].clone();
    }
    items[items.len() - 1] = item;
    CacheEntry { key, value }
}

/// Identity hash/equality context for integer keys.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityU32Context;

impl CacheContext<u32, u32> for IdentityU32Context {
    fn hash(&self, key: &u32) -> u64 {
        *key as u64
    }

    fn eql(&self, a: &u32, b: &u32) -> bool {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestTable = CacheTable<u32, u32, IdentityU32Context, 2, 2>;

    #[test]
    fn put_and_get_with_eviction() {
        let mut table = TestTable::new(IdentityU32Context);

        assert!(table.put(0, 0).is_none());
        assert!(table.put(1, 0).is_none());
        assert!(table.put(2, 0).is_none());
        assert!(table.put(3, 0).is_none());

        let evicted = table.put(4, 0).expect("expected eviction");
        assert_eq!(evicted.key, 0);
        assert_eq!(evicted.value, 0);

        assert_eq!(table.get(&0), None);
    }
}
