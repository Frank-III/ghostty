//! Collection of growable lists ported from `src/datastruct/array_list_collection.zig`.

/// A collection of [`Vec`]s with methods for bulk operations.
pub struct ArrayListCollection<T> {
    /// Lists belonging to this collection.
    pub lists: Vec<Vec<T>>,
}

impl<T> ArrayListCollection<T> {
    pub fn new(list_count: usize, initial_capacity: usize) -> Self {
        let lists = (0..list_count)
            .map(|_| Vec::with_capacity(initial_capacity))
            .collect();
        Self { lists }
    }

    /// Clear all lists in the collection, retaining capacity.
    pub fn reset(&mut self) {
        for list in &mut self.lists {
            list.clear();
        }
    }
}

impl<T> Default for ArrayListCollection<T> {
    fn default() -> Self {
        Self { lists: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_lists_with_capacity() {
        let coll = ArrayListCollection::<u32>::new(3, 8);
        assert_eq!(coll.lists.len(), 3);
        for list in &coll.lists {
            assert!(list.is_empty());
            assert!(list.capacity() >= 8);
        }
    }

    #[test]
    fn reset_clears_retaining_capacity() {
        let mut coll = ArrayListCollection::new(2, 4);
        coll.lists[0].extend([1, 2, 3]);
        coll.lists[1].push(9);
        let cap0 = coll.lists[0].capacity();
        let cap1 = coll.lists[1].capacity();

        coll.reset();

        assert!(coll.lists[0].is_empty());
        assert!(coll.lists[1].is_empty());
        assert_eq!(coll.lists[0].capacity(), cap0);
        assert_eq!(coll.lists[1].capacity(), cap1);
    }

    #[test]
    fn lists_are_independent() {
        let mut coll = ArrayListCollection::new(2, 2);
        coll.lists[0].push(1);
        assert_eq!(coll.lists[0].len(), 1);
        assert!(coll.lists[1].is_empty());
    }

    #[test]
    fn default_is_empty() {
        let coll = ArrayListCollection::<i32>::default();
        assert!(coll.lists.is_empty());
    }
}
