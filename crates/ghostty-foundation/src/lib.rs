//! Foundation crate for the Ghostty Rust port (Phase 1).
//!
//! Ports from Zig:
//! - `src/datastruct/`
//! - `src/unicode/`
//! - `src/simd/`
//! - `src/lib/`
//! - `src/os/`

pub mod os;

pub mod error;

pub mod array_list_collection;
pub mod blocking_queue;
pub mod cache_table;
pub mod circ_buf;
pub mod comparison;
pub mod intrusive_list;
pub mod lru;
pub mod message_data;
pub mod segmented_pool;
pub mod split_tree;
pub mod unicode;

pub use array_list_collection::ArrayListCollection;
pub use blocking_queue::{BlockingQueue, QueueSize, Timeout};
pub use cache_table::{CacheContext, CacheEntry, CacheTable, IdentityU32Context};
pub use circ_buf::{CircBuf, Direction};
pub use comparison::{approx_eq_f32, approx_eq_f64, deep_equal, DeepEqual};
pub use error::{FoundationError, FoundationResult};
pub use intrusive_list::{IntrusiveList, IntrusiveNode};
pub use lru::{GetOrPutResult, LruMap};
pub use message_data::MessageData;
pub use os::{
    append_env, append_env_always, is_valid_mac_address, path_expand, prepend_env, printf_q_decode,
    url_percent_decode, url_percent_encode, DecodeError, PathExpandError,
};
pub use segmented_pool::{PoolError, PoolIndex, SegmentedPool};
pub use split_tree::{
    Goto, NodeHandle, Side, Spatial, SpatialDirection, SpatialSlot, SplitDirection, SplitLayout,
    SplitTree, SplitTreeIterator, ViewEntry,
};
pub use unicode::{
    grapheme_break, grapheme_break_no_control, BreakState, GraphemeBreak, Properties,
};
