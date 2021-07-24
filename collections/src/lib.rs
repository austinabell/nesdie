mod utils;

/// A growable Vector with elements stored on the trie.
pub mod vec;

pub use vec::Vector;

#[cfg(feature = "legacy")]
pub mod legacy_unordered_map;
