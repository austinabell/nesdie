//! A wrapper around the runtime key-value storage.

use borsh::{BorshDeserialize, BorshSerialize};
use core::{borrow::Borrow, marker::PhantomData};
use nesdie::env;

use crate::{
    key::{Identity, ToKey},
    utils,
};

/// A wrapper around the NEAR contract key-value storage.
pub struct KvStore<K, V, H = Identity> {
    prefix: Box<[u8]>,
    _marker: PhantomData<fn() -> (K, V, H)>,
}

impl<K, V> KvStore<K, V> {
    pub fn new(prefix: Box<[u8]>) -> Self {
        Self {
            prefix,
            _marker: Default::default(),
        }
    }
}

impl<K, V, H> ::core::fmt::Debug for KvStore<K, V, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("KvStore")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl<K, V, H> KvStore<K, V, H>
where
    H: ToKey,
{
    /// Initialize a [`KvStore`] with a custom hash function.
    ///
    /// # Example
    /// ```
    /// use nesdie_store::{KvStore, key::Keccak256};
    ///
    /// let map = KvStore::<String, String, Keccak256>::with_hasher(b"m".to_vec().into_boxed_slice());
    /// ```
    pub fn with_hasher(prefix: Box<[u8]>) -> Self {
        Self {
            prefix: prefix,
            _marker: Default::default(),
        }
    }
}

impl<K, V, H> KvStore<K, V, H>
where
    H: ToKey,
{
    fn deserialize_element(bytes: &[u8]) -> V
    where
        V: BorshDeserialize,
    {
        V::try_from_slice(bytes).unwrap()
    }

    fn storage_key<Q: ?Sized>(&self, key: &Q) -> Vec<u8>
    where
        Q: BorshSerialize,
        K: Borrow<Q>,
    {
        let mut buffer = Vec::with_capacity(self.prefix.len());
        H::to_key(&self.prefix, key, &mut buffer);
        buffer
    }

    /// Inserts a key-value pair into storage.
    ///
    /// If storage did not have this key present, [`None`] is returned.
    ///
    /// If storage did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    ///
    /// # Example
    /// ```
    /// use nesdie_store::KvStore;
    ///
    /// let mut map: KvStore<u32, String> = KvStore::new(b"m".to_vec().into_boxed_slice());
    /// assert!(!map.insert(&37, "a"));
    /// assert_eq!(map.contains_key(&37), true);
    ///
    /// map.insert(&37, "b");
    /// assert!(map.insert(&37, "c"));
    /// assert_eq!(map.get(&37).unwrap(), "c".to_string());
    /// ```
    #[inline]
    pub fn insert<Q: ?Sized, R: ?Sized>(&mut self, key: &Q, value: &R) -> bool
    where
        K: Borrow<Q>,
        Q: BorshSerialize,
        V: Borrow<R>,
        R: BorshSerialize,
    {
        env::storage_write(&self.storage_key(&key), &value.try_to_vec().unwrap())
    }

    /// Returns the value corresponding to the key.
    ///
    /// The key may be any borrowed form of storage's key type, but
    /// [`BorshSerialize`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Example
    /// ```
    /// use nesdie_store::KvStore;
    ///
    /// let mut map: KvStore<u32, String> = KvStore::new(b"m".to_vec().into_boxed_slice());
    ///
    /// map.insert(&1, "a");
    /// assert_eq!(map.get(&1u32), Some("a".to_string()));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: BorshSerialize,
        V: BorshDeserialize,
    {
        utils::alloc_storage_read(&self.storage_key(&key))
            .as_deref()
            .map(Self::deserialize_element)
    }

    /// Returns `true` if storage contains a value for the specified key.
    ///
    /// The key may be any borrowed form of storage's key type, but
    /// [`BorshSerialize`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Example
    /// ```
    /// use nesdie_store::KvStore;
    ///
    /// let mut map: KvStore<u32, String> = KvStore::new(b"m".to_vec().into_boxed_slice());
    /// map.insert(&1, "a");
    /// assert_eq!(map.contains_key(&1u32), true);
    /// assert_eq!(map.contains_key(&2u32), false);
    /// ```
    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: BorshSerialize,
    {
        env::storage_has_key(&self.storage_key(&key))
    }

    /// Removes a key from storage, returning the value at the key if the key
    /// was previously in storage.
    ///
    /// The key may be any borrowed form of storage's key type, but
    /// [`BorshSerialize`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Example
    /// ```
    /// use nesdie_store::KvStore;
    ///
    /// let mut map: KvStore<u32, String> = KvStore::new(b"m".to_vec().into_boxed_slice());
    /// map.insert(&1, "a");
    /// assert_eq!(map.remove(&1), Some("a".to_string()));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    pub fn remove<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: BorshSerialize,
        V: BorshDeserialize,
    {
        utils::storage_remove_alloc(&self.storage_key(&key))
            .as_deref()
            .map(Self::deserialize_element)
    }
}
