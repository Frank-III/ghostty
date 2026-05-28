//! Intrusive doubly-linked list ported from `src/datastruct/intrusive_linked_list.zig`.

use core::ptr::NonNull;

/// Node trait for intrusive list links.
pub unsafe trait IntrusiveNode {
    fn prev(&self) -> Option<NonNull<Self>>;
    fn next(&self) -> Option<NonNull<Self>>;
    fn set_prev(&mut self, prev: Option<NonNull<Self>>);
    fn set_next(&mut self, next: Option<NonNull<Self>>);
}

/// Intrusive doubly-linked list.
pub struct IntrusiveList<T: IntrusiveNode> {
    first: Option<NonNull<T>>,
    last: Option<NonNull<T>>,
}

impl<T: IntrusiveNode> Default for IntrusiveList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IntrusiveNode> IntrusiveList<T> {
    pub fn new() -> Self {
        Self {
            first: None,
            last: None,
        }
    }

    pub fn first(&self) -> Option<NonNull<T>> {
        self.first
    }

    pub fn last(&self) -> Option<NonNull<T>> {
        self.last
    }

    pub unsafe fn insert_after(&mut self, node: NonNull<T>, new_node: NonNull<T>) {
        unsafe {
            let node_ref = node.as_ptr();
            let new_ref = new_node.as_ptr();

            (*new_ref).set_prev(Some(node));
            if let Some(next) = (*node_ref).next() {
                (*new_ref).set_next(Some(next));
                next.as_ptr().as_mut().unwrap_unchecked().set_prev(Some(new_node));
            } else {
                (*new_ref).set_next(None);
                self.last = Some(new_node);
            }
            (*node_ref).set_next(Some(new_node));
        }
    }

    pub unsafe fn insert_before(&mut self, node: NonNull<T>, new_node: NonNull<T>) {
        unsafe {
            let node_ref = node.as_ptr();
            let new_ref = new_node.as_ptr();

            (*new_ref).set_next(Some(node));
            if let Some(prev) = (*node_ref).prev() {
                (*new_ref).set_prev(Some(prev));
                prev.as_ptr().as_mut().unwrap_unchecked().set_next(Some(new_node));
            } else {
                (*new_ref).set_prev(None);
                self.first = Some(new_node);
            }
            (*node_ref).set_prev(Some(new_node));
        }
    }

    pub unsafe fn append(&mut self, new_node: NonNull<T>) {
        if let Some(last) = self.last {
            self.insert_after(last, new_node);
        } else {
            self.prepend(new_node);
        }
    }

    pub unsafe fn prepend(&mut self, new_node: NonNull<T>) {
        if let Some(first) = self.first {
            self.insert_before(first, new_node);
        } else {
            unsafe {
                let new_ref = new_node.as_ptr();
                (*new_ref).set_prev(None);
                (*new_ref).set_next(None);
            }
            self.first = Some(new_node);
            self.last = Some(new_node);
        }
    }

    pub unsafe fn remove(&mut self, node: NonNull<T>) {
        unsafe {
            let node_ref = node.as_ptr();
            if let Some(prev) = (*node_ref).prev() {
                prev.as_ptr()
                    .as_mut()
                    .unwrap_unchecked()
                    .set_next((*node_ref).next());
            } else {
                self.first = (*node_ref).next();
            }

            if let Some(next) = (*node_ref).next() {
                next.as_ptr()
                    .as_mut()
                    .unwrap_unchecked()
                    .set_prev((*node_ref).prev());
            } else {
                self.last = (*node_ref).prev();
            }
        }
    }

    pub unsafe fn pop(&mut self) -> Option<NonNull<T>> {
        let last = self.last?;
        self.remove(last);
        Some(last)
    }

    pub unsafe fn pop_first(&mut self) -> Option<NonNull<T>> {
        let first = self.first?;
        self.remove(first);
        Some(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Node {
        data: u32,
        prev: Option<NonNull<Node>>,
        next: Option<NonNull<Node>>,
    }

    unsafe impl IntrusiveNode for Node {
        fn prev(&self) -> Option<NonNull<Self>> {
            self.prev
        }

        fn next(&self) -> Option<NonNull<Self>> {
            self.next
        }

        fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
            self.prev = prev;
        }

        fn set_next(&mut self, next: Option<NonNull<Self>>) {
            self.next = next;
        }
    }

    fn ptr(node: &mut Node) -> NonNull<Node> {
        NonNull::from(node)
    }

    #[test]
    fn basic_list_operations() {
        let mut list = IntrusiveList::<Node>::new();
        let mut one = Node {
            data: 1,
            prev: None,
            next: None,
        };
        let mut two = Node {
            data: 2,
            prev: None,
            next: None,
        };
        let mut three = Node {
            data: 3,
            prev: None,
            next: None,
        };
        let mut four = Node {
            data: 4,
            prev: None,
            next: None,
        };
        let mut five = Node {
            data: 5,
            prev: None,
            next: None,
        };

        unsafe {
            list.append(ptr(&mut two));
            list.append(ptr(&mut five));
            list.prepend(ptr(&mut one));
            list.insert_before(ptr(&mut five), ptr(&mut four));
            list.insert_after(ptr(&mut two), ptr(&mut three));
        }

        unsafe {
            let mut it = list.first;
            let mut index = 1;
            while let Some(node) = it {
                assert_eq!(node.as_ref().data, index);
                it = node.as_ref().next;
                index += 1;
            }
            assert_eq!(index, 6);
        }

        unsafe {
            let mut it = list.last;
            let mut index = 1;
            while let Some(node) = it {
                assert_eq!(node.as_ref().data, 6 - index);
                it = node.as_ref().prev;
                index += 1;
            }
            assert_eq!(index, 6);
        }

        unsafe {
            assert_eq!(list.pop_first().unwrap().as_ref().data, 1);
            assert_eq!(list.pop().unwrap().as_ref().data, 5);
            list.remove(ptr(&mut three));
            assert_eq!(list.first().unwrap().as_ref().data, 2);
            assert_eq!(list.last().unwrap().as_ref().data, 4);
        }
    }
}
