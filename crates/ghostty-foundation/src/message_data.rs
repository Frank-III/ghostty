//! Small/allocated message payload union ported from `src/datastruct/message_data.zig`.

use std::borrow::Cow;

/// Union-style message payload for thread messaging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageData<'a, T: Clone> {
    Small(SmallBuffer<T>),
    Stable(Cow<'a, [T]>),
    Alloc(Vec<T>),
}

/// Inline storage for small payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmallBuffer<T: Clone> {
    data: Vec<T>,
}

impl<'a, T: Clone> MessageData<'a, T> {
    pub fn init(data: &[T], small_capacity: usize) -> Self {
        if data.len() <= small_capacity {
            return Self::Small(SmallBuffer {
                data: data.to_vec(),
            });
        }
        Self::Alloc(data.to_vec())
    }

    pub fn from_stable(data: Cow<'a, [T]>, small_capacity: usize) -> Self {
        if data.len() <= small_capacity {
            return Self::Small(SmallBuffer {
                data: data.into_owned(),
            });
        }
        Self::Stable(data)
    }

    pub fn slice(&self) -> &[T] {
        match self {
            Self::Small(v) => &v.data,
            Self::Stable(v) => v,
            Self::Alloc(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_small() {
        let data = MessageData::<u8>::init(b"hello!", 10);
        assert!(matches!(data, MessageData::Small(_)));
        assert_eq!(data.slice(), b"hello!");
    }

    #[test]
    fn init_alloc() {
        let input = "hello! ".repeat(100);
        let data = MessageData::<u8>::init(input.as_bytes(), 10);
        assert!(matches!(data, MessageData::Alloc(_)));
    }

    #[test]
    fn small_fits_large_capacity() {
        let input = "X".repeat(500);
        let data = MessageData::<u8>::init(input.as_bytes(), 500);
        assert!(matches!(data, MessageData::Small(_)));
    }
}
