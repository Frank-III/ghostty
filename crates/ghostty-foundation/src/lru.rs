//! LRU hash map ported from `src/datastruct/lru.zig`.

use core::hash::Hash;
use std::collections::{HashMap, VecDeque};

/// Result of [`LruMap::get_or_put`].
pub struct GetOrPutResult<'a, K, V> {
    pub value: &'a mut V,
    pub found_existing: bool,
    pub evicted: Option<(K, V)>,
}

/// Hash map with least-recently-used eviction.
pub struct LruMap<K, V>
where
    K: Eq + Hash + Clone,
{
    map: HashMap<K, V>,
    order: VecDeque<K>,
    capacity: usize,
}

impl<K, V> LruMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Default,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_or_put(&mut self, key: K) -> GetOrPutResult<'_, K, V> {
        if self.map.contains_key(&key) {
            self.touch(&key);
            let value = self.map.get_mut(&key).expect("key exists");
            return GetOrPutResult {
                value,
                found_existing: true,
                evicted: None,
            };
        }

        let evicted = if self.map.len() >= self.capacity {
            self.evict_lru()
        } else {
            None
        };

        self.order.push_back(key.clone());
        let value = self.map.entry(key).or_default();

        GetOrPutResult {
            value,
            found_existing: false,
            evicted,
        }
    }

    pub fn resize(&mut self, capacity: usize) -> Option<Vec<V>> {
        if capacity >= self.capacity {
            self.capacity = capacity;
            return None;
        }

        if self.map.len() <= capacity {
            self.capacity = capacity;
            return None;
        }

        let delta = self.map.len() - capacity;
        let mut evicted = Vec::with_capacity(delta);
        for _ in 0..delta {
            if let Some((_, value)) = self.evict_lru() {
                evicted.push(value);
            }
        }
        self.capacity = capacity;
        debug_assert_eq!(self.map.len(), capacity);
        Some(evicted)
    }

    fn touch(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key.clone());
    }

    fn evict_lru(&mut self) -> Option<(K, V)> {
        let key = self.order.pop_front()?;
        self.map.remove_entry(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_put_evicts_lru() {
        let mut map = LruMap::new(2);
        {
            let gop = map.get_or_put(1);
            assert!(!gop.found_existing);
            assert!(gop.evicted.is_none());
            *gop.value = 1;
        }
        {
            let gop = map.get_or_put(2);
            assert!(!gop.found_existing);
            *gop.value = 2;
        }

        assert!(map.get_or_put(1).found_existing);
        assert!(map.get_or_put(2).found_existing);

        {
            let gop = map.get_or_put(3);
            assert!(!gop.found_existing);
            let evicted = gop.evicted.expect("expected eviction");
            assert_eq!(evicted.0, 1);
            assert_eq!(evicted.1, 1);
            *gop.value = 3;
        }

        assert!(map.get_or_put(2).found_existing);
        {
            let gop = map.get_or_put(4);
            assert!(!gop.found_existing);
            let evicted = gop.evicted.expect("expected eviction");
            assert_eq!(evicted.0, 3);
            assert_eq!(evicted.1, 3);
            *gop.value = 4;
        }
    }

    #[test]
    fn resize_shrink_removes_lru() {
        let mut map = LruMap::new(2);
        {
            let gop = map.get_or_put(1);
            *gop.value = 1;
        }
        {
            let gop = map.get_or_put(2);
            *gop.value = 2;
        }

        let evicted = map.resize(1).expect("expected evictions");
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0], 1);

        let gop = map.get_or_put(1);
        assert!(!gop.found_existing);
        assert_eq!(gop.evicted.as_ref().map(|(_, v)| *v), Some(2));
    }
}
