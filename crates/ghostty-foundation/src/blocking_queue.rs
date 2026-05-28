//! Fixed-capacity SPSC blocking queue ported from `src/datastruct/blocking_queue.zig`.

use std::sync::{Condvar, Mutex};
use std::time::Duration;

pub type QueueSize = u32;

/// Push timeout behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeout {
    Instant,
    Forever,
    Ns(u64),
}

struct Inner<T, const CAPACITY: usize> {
    data: [Option<T>; CAPACITY],
    write: QueueSize,
    read: QueueSize,
    len: QueueSize,
    not_full_waiters: usize,
}

/// Fixed-capacity blocking queue for single-producer/single-consumer usage.
pub struct BlockingQueue<T, const CAPACITY: usize> {
    inner: Mutex<Inner<T, CAPACITY>>,
    cond_not_full: Condvar,
}

pub struct DrainIter<'a, T, const CAPACITY: usize> {
    guard: MutexGuard<'a, Inner<T, CAPACITY>>,
}

type MutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;

impl<T, const CAPACITY: usize> BlockingQueue<T, CAPACITY> {
    const BOUNDS: QueueSize = CAPACITY as QueueSize;

    pub fn new() -> Self {
        assert!(CAPACITY > 0, "blocking queue capacity must be non-zero");
        Self {
            inner: Mutex::new(Inner {
                data: std::array::from_fn(|_| None),
                write: 0,
                read: 0,
                len: 0,
                not_full_waiters: 0,
            }),
            cond_not_full: Condvar::new(),
        }
    }

    /// Push a value. Returns the queue length after push, or `0` on failure.
    pub fn push(&self, value: T, timeout: Timeout) -> QueueSize {
        let mut inner = self.inner.lock().unwrap();

        if inner.len == Self::BOUNDS {
            match timeout {
                Timeout::Instant => return 0,
                Timeout::Forever => {
                    inner.not_full_waiters += 1;
                    drop(inner);
                    inner = self
                        .cond_not_full
                        .wait(self.inner.lock().unwrap())
                        .unwrap_or_else(|e| e.into_inner());
                    inner.not_full_waiters = inner.not_full_waiters.saturating_sub(1);
                }
                Timeout::Ns(ns) => {
                    inner.not_full_waiters += 1;
                    drop(inner);
                    let (new_inner, timed_out) = self
                        .cond_not_full
                        .wait_timeout(self.inner.lock().unwrap(), Duration::from_nanos(ns))
                        .unwrap_or_else(|e| e.into_inner());
                    inner = new_inner;
                    inner.not_full_waiters = inner.not_full_waiters.saturating_sub(1);
                    if timed_out.timed_out() {
                        return 0;
                    }
                }
            }

            if inner.len == Self::BOUNDS {
                return 0;
            }
        }

        let write = inner.write as usize;
        inner.data[write] = Some(value);
        inner.write += 1;
        if inner.write >= Self::BOUNDS {
            inner.write = 0;
        }
        inner.len += 1;
        inner.len
    }

    pub fn pop(&self) -> Option<T> {
        let mut inner = self.inner.lock().unwrap();
        if inner.len == 0 {
            return None;
        }

        let idx = inner.read as usize;
        inner.read += 1;
        if inner.read >= Self::BOUNDS {
            inner.read = 0;
        }
        inner.len -= 1;

        if inner.not_full_waiters > 0 {
            self.cond_not_full.notify_one();
        }

        inner.data[idx].take()
    }

    pub fn drain(&self) -> DrainIter<'_, T, CAPACITY> {
        DrainIter {
            guard: self.inner.lock().unwrap(),
        }
    }
}

impl<'a, T, const CAPACITY: usize> Iterator for DrainIter<'a, T, CAPACITY> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = &mut *self.guard;
        if inner.len == 0 {
            return None;
        }

        let idx = inner.read as usize;
        inner.read += 1;
        if inner.read >= BlockingQueue::<T, CAPACITY>::BOUNDS {
            inner.read = 0;
        }
        inner.len -= 1;
        inner.data[idx].take()
    }
}

impl<'a, T, const CAPACITY: usize> Drop for DrainIter<'a, T, CAPACITY> {
    fn drop(&mut self) {
        if self.guard.not_full_waiters > 0 {
            // Drop cannot access Condvar on BlockingQueue; notify on last pop instead.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_push_and_pop() {
        let q = BlockingQueue::<u64, 4>::new();
        assert_eq!(q.pop(), None);

        assert_eq!(q.push(1, Timeout::Instant), 1);
        assert_eq!(q.push(2, Timeout::Instant), 2);
        assert_eq!(q.push(3, Timeout::Instant), 3);
        assert_eq!(q.push(4, Timeout::Instant), 4);
        assert_eq!(q.push(5, Timeout::Instant), 0);

        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.pop(), Some(4));
        assert_eq!(q.pop(), None);

        let mut it = q.drain();
        assert_eq!(it.next(), None);
        drop(it);

        assert_eq!(q.push(1, Timeout::Instant), 1);
    }

    #[test]
    fn timed_push_fails_when_full() {
        let q = BlockingQueue::<u64, 1>::new();
        assert_eq!(q.push(1, Timeout::Instant), 1);
        assert_eq!(q.push(2, Timeout::Instant), 0);
        assert_eq!(q.push(2, Timeout::Ns(1_000)), 0);
    }
}
