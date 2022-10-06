#![cfg_attr(target_arch = "wasm32", no_std)]

mod utils;

/// Storage key hash function types and trait to override map hash functions.
pub mod key;
mod kvstore;
pub use kvstore::KvStore;

extern crate alloc;

mod lib {
    mod core {
        pub use core::*;
    }

    pub use self::core::cell::{Cell, RefCell};
    pub use self::core::clone::{self, Clone};
    pub use self::core::convert::{self, From, Into};
    pub use self::core::default::{self, Default};
    pub use self::core::fmt::{self, Debug, Display};
    pub use self::core::hash::{self, Hash};
    pub use self::core::iter::FusedIterator;
    pub use self::core::marker::{self, PhantomData};
    pub use self::core::ops::{Bound, RangeBounds};
    pub use self::core::result::{self, Result};
    pub use self::core::{borrow, char, cmp, iter, mem, num, ops, slice, str};

    pub use alloc::borrow::{Cow, ToOwned};

    pub use alloc::string::{String, ToString};

    pub use alloc::vec::{self, Vec};

    pub use alloc::boxed::Box;

    pub use alloc::collections::{btree_map, BTreeMap};
}
