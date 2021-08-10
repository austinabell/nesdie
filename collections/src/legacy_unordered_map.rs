//! A map implemented on a trie. Unlike `std::collections::HashMap` the keys in this map are not
//! hashed but are instead serialized.

use crate::utils::{alloc_storage_read, append_slice};
use crate::Vector;
use borsh::{BorshDeserialize, BorshSerialize};
use core::mem::size_of;
use nesdie::env;

use crate::lib::{Box, Vec};

/// An iterable implementation of a map that stores its content directly on the trie.
/// * NOTE: This structure is just used for compatibility with old contracts that use this type.
/// * Please do not use this otherwise.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct UnorderedMap<K, V>
where
    K: BorshSerialize,
    V: BorshSerialize,
{
    key_index_prefix: Vec<u8>,
    keys: Vector<K>,
    values: Vector<V>,
}

impl<K, V> UnorderedMap<K, V>
where
    K: BorshSerialize,
    V: BorshSerialize,
{
    /// Returns the number of elements in the map, also referred to as its size.
    pub fn len(&self) -> u32 {
        let keys_len = self.keys.len();
        let values_len = self.values.len();
        if keys_len != values_len {
            unreachable!()
        } else {
            keys_len
        }
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        let keys_is_empty = self.keys.is_empty();
        let values_is_empty = self.values.is_empty();
        if keys_is_empty != values_is_empty {
            unreachable!()
        } else {
            keys_is_empty
        }
    }

    /// Create new map with zero elements. Use `prefix` as a unique identifier.
    pub fn new(prefix: Box<[u8]>) -> Self {
        let key_index_prefix = append_slice(&prefix, &[b'i']);
        let index_key_id = append_slice(&prefix, &[b'k']);
        let index_value_id = append_slice(&prefix, &[b'v']);

        Self {
            key_index_prefix,
            keys: Vector::new(index_key_id.into_boxed_slice()),
            values: Vector::new(index_value_id.into_boxed_slice()),
        }
    }

    fn serialize_index(index: u32) -> [u8; size_of::<u32>()] {
        index.to_le_bytes()
    }

    fn deserialize_index(raw_index: &[u8]) -> u32 {
        let mut result = [0u8; size_of::<u32>()];
        result.copy_from_slice(raw_index);
        u32::from_le_bytes(result)
    }

    fn raw_key_to_index_lookup(&self, raw_key: &[u8]) -> Vec<u8> {
        append_slice(&self.key_index_prefix, raw_key)
    }
}

impl<K, V> UnorderedMap<K, V>
where
    K: BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
{
    fn get_index(&self, key: &K) -> Option<u32> {
        let key_raw = key.try_to_vec().ok()?;
        let index_lookup = self.raw_key_to_index_lookup(&key_raw);
        alloc_storage_read(&index_lookup).map(|raw_index| Self::deserialize_index(&raw_index))
    }

    /// Returns the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_index(key)
            .map(|index| self.values.get(index).unwrap_or_else(|| unreachable!()))
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let key_raw = key.try_to_vec().ok()?;
        let index_lookup = self.raw_key_to_index_lookup(&key_raw);
        let index_raw = alloc_storage_read(&index_lookup)?;
        #[allow(clippy::branches_sharing_code)]
        if self.len() == 1 {
            // If there is only one element then swap remove simply removes it without
            // swapping with the last element.
            env::storage_remove(&index_lookup);
        } else {
            // If there is more than one element then swap remove swaps it with the last
            // element.
            let last_key = self
                .keys
                .get(self.len() - 1)
                .unwrap_or_else(|| unreachable!());
            env::storage_remove(&index_lookup);

            let last_key_raw = last_key.try_to_vec().unwrap_or_else(|_| unreachable!());
            // If the removed element was the last element from keys, then we don't need to
            // reinsert the lookup back.
            if last_key_raw != key_raw {
                let last_lookup_key = self.raw_key_to_index_lookup(&last_key_raw);
                env::storage_write(&last_lookup_key, &index_raw);
            }
        }
        let index = Self::deserialize_index(&index_raw);
        self.keys.swap_remove(index);
        Some(self.values.swap_remove(index))
    }

    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, `None` is returned. Otherwise returns
    /// a value. Note, the keys that have the same hash value are undistinguished by
    /// the implementation.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_raw = key.try_to_vec().unwrap_or_else(|_| unreachable!());
        let index_lookup = self.raw_key_to_index_lookup(&key_raw);
        match alloc_storage_read(&index_lookup) {
            Some(index_raw) => {
                // The element already exists.
                let index = Self::deserialize_index(&index_raw);
                Some(self.values.replace(index, value))
            }
            None => {
                // The element does not exist yet.
                let next_index = self.len();
                let next_index_raw = Self::serialize_index(next_index);
                env::storage_write(&index_lookup, &next_index_raw);
                self.keys.push(key);
                self.values.push(value);
                None
            }
        }
        // self.insert_raw(&Self::serialize_key(key), &Self::serialize_value(&value))
        //     .map(|value_raw| Self::deserialize_value(&value_raw))
    }

    /// Clears the map, removing all elements.
    pub fn clear(&mut self) {
        for key in self.keys.iter() {
            let raw_key = key.try_to_vec().unwrap_or_else(|_| unreachable!());
            let index_lookup = self.raw_key_to_index_lookup(&raw_key);
            env::storage_remove(&index_lookup);
        }
        self.keys.clear();
        self.values.clear();
    }

    /// An iterator visiting all keys. The iterator element type is `K`.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.keys.iter()
    }

    /// An iterator visiting all values. The iterator element type is `V`.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.values.iter()
    }

    /// Iterate over deserialized keys and values.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys.iter().zip(self.values.iter())
    }

    pub fn extend<IT: IntoIterator<Item = (K, V)>>(&mut self, iter: IT) {
        for (el_key, el_value) in iter {
            self.insert(el_key, el_value);
        }
    }

    /// Returns a view of keys as a vector.
    /// It's sometimes useful to have random access to the keys.
    pub fn keys_as_vector(&self) -> &Vector<K> {
        &self.keys
    }

    /// Returns a view of values as a vector.
    /// It's sometimes useful to have random access to the values.
    pub fn values_as_vector(&self) -> &Vector<V> {
        &self.values
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::UnorderedMap;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};
    use std::collections::{HashMap, HashSet};

    #[test]
    pub fn test_insert() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0);
        for _ in 0..500 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            map.insert(key, value);
        }
    }

    #[test]
    pub fn test_insert_remove() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(1);
        let mut keys = vec![];
        let mut key_to_value = HashMap::new();
        for _ in 0..100 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            keys.push(key);
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        keys.shuffle(&mut rng);
        for key in keys {
            let actual = map.remove(&key).unwrap();
            assert_eq!(actual, key_to_value[&key]);
        }
    }

    #[test]
    pub fn test_remove_last_reinsert() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let key1 = 1u64;
        let value1 = 2u64;
        map.insert(key1, value1);
        let key2 = 3u64;
        let value2 = 4u64;
        map.insert(key2, value2);

        let actual_value2 = map.remove(&key2).unwrap();
        assert_eq!(actual_value2, value2);

        let actual_insert_value2 = map.insert(key2, value2);
        assert_eq!(actual_insert_value2, None);
    }

    #[test]
    pub fn test_insert_override_remove() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(2);
        let mut keys = vec![];
        let mut key_to_value = HashMap::new();
        for _ in 0..100 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            keys.push(key);
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        keys.shuffle(&mut rng);
        for key in &keys {
            let value = rng.gen::<u64>();
            let actual = map.insert(*key, value).unwrap();
            assert_eq!(actual, key_to_value[key]);
            key_to_value.insert(*key, value);
        }
        keys.shuffle(&mut rng);
        for key in keys {
            let actual = map.remove(&key).unwrap();
            assert_eq!(actual, key_to_value[&key]);
        }
    }

    #[test]
    pub fn test_get_non_existent() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(3);
        let mut key_to_value = HashMap::new();
        for _ in 0..500 {
            let key = rng.gen::<u64>() % 20_000;
            let value = rng.gen::<u64>();
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        for _ in 0..500 {
            let key = rng.gen::<u64>() % 20_000;
            assert_eq!(map.get(&key), key_to_value.get(&key));
        }
    }

    #[test]
    pub fn test_to_vec() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(4);
        let mut key_to_value = HashMap::new();
        for _ in 0..400 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        let actual: HashMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(actual, key_to_value);
    }

    #[test]
    pub fn test_clear() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(5);
        for _ in 0..10 {
            for _ in 0..=(rng.gen::<u64>() % 20 + 1) {
                let key = rng.gen::<u64>();
                let value = rng.gen::<u64>();
                map.insert(key, value);
            }
            assert!(!map.iter().next().is_none());
            map.clear();
            assert!(map.iter().next().is_none());
        }
    }

    #[test]
    pub fn test_keys_values() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(4);
        let mut key_to_value = HashMap::new();
        for _ in 0..400 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        let actual: HashMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(
            actual.keys().collect::<HashSet<_>>(),
            key_to_value.keys().collect::<HashSet<_>>()
        );
        assert_eq!(
            actual.values().collect::<HashSet<_>>(),
            key_to_value.values().collect::<HashSet<_>>()
        );
    }

    #[test]
    pub fn test_iter() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(4);
        let mut key_to_value = HashMap::new();
        for _ in 0..400 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        let actual: HashMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(actual, key_to_value);
    }

    #[test]
    pub fn test_extend() {
        let mut map = UnorderedMap::new(Box::new(*b"m") as Box<_>);
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(4);
        let mut key_to_value = HashMap::new();
        for _ in 0..100 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            key_to_value.insert(key, value);
            map.insert(key, value);
        }
        for _ in 0..10 {
            let mut tmp = vec![];
            for _ in 0..=(rng.gen::<u64>() % 20 + 1) {
                let key = rng.gen::<u64>();
                let value = rng.gen::<u64>();
                tmp.push((key, value));
            }
            key_to_value.extend(tmp.iter().cloned());
            map.extend(tmp.iter().cloned());
        }

        let actual: HashMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(actual, key_to_value);
    }
}
