use alloc::collections::{BTreeMap, BTreeSet};
use core::ops::Bound::*;

pub struct MultiValueIndex<Key: Ord + Copy, Value: Ord + Copy> {
    map: BTreeMap<Key, BTreeSet<Value>>,
    reverse_map: BTreeMap<Value, BTreeSet<Key>>,
}

impl<Key: Ord + Copy, Value: Ord + Copy> Default for MultiValueIndex<Key, Value> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            reverse_map: Default::default(),
        }
    }
}

impl<Key: Ord + Copy, Value: Ord + Copy> MultiValueIndex<Key, Value> {
    /// Inserts a pair (Key, Value) into the index
    pub fn insert_pair(&mut self, key: Key, value: Value) -> bool {
        if self.map.entry(key).or_default().insert(value) {
            self.reverse_map.entry(value).or_default().insert(key);
            true
        } else {
            false
        }
    }

    /// Removes a pair (Key, Value) from the index
    pub fn remove_pair(&mut self, key: Key, value: Value) -> bool {
        if let Some(set) = self.map.get_mut(&key)
            && set.remove(&value)
        {
            if set.is_empty() {
                self.map.remove(&key);
            }

            if let Some(rset) = self.reverse_map.get_mut(&value) {
                rset.remove(&key);
                if rset.is_empty() {
                    self.reverse_map.remove(&value);
                }
            }
            true
        } else {
            false
        }
    }

    /// Removes all pairs with the specified Key from the index
    pub fn remove_key(&mut self, key: Key) {
        if let Some(set) = self.map.remove(&key) {
            for value in set.iter() {
                if let Some(rset) = self.reverse_map.get_mut(value) {
                    rset.remove(&key);
                    if rset.is_empty() {
                        self.reverse_map.remove(value);
                    }
                }
            }
        }
    }

    /// Removes all pairs with the specified Value from the index
    pub fn remove_value(&mut self, value: Value) -> bool {
        if let Some(rset) = self.reverse_map.remove(&value) {
            for key in rset.iter() {
                if let Some(set) = self.map.get_mut(key) {
                    set.remove(&value);
                    if set.is_empty() {
                        self.map.remove(key);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Checks if a pair (Key, Value) is in the index
    pub fn contains_pair(&self, key: Key, value: Value) -> bool {
        self.map
            .get(&key)
            .is_some_and(|values| values.contains(&value))
    }

    /// Checks if a Key is in the index
    pub fn contains_key(&self, key: Key) -> bool {
        self.map.contains_key(&key)
    }

    /// Checks if a Value is in the index
    pub fn contains_value(&self, value: Value) -> bool {
        self.reverse_map.contains_key(&value)
    }

    /// Retrieves all the Values for the specified Key from the index
    pub fn get_values_from_key(&self, key: Key) -> Option<&BTreeSet<Value>> {
        self.map.get(&key)
    }

    /// Retrieves all the Values for the specified Key, and all the Keys above it, from the index
    pub fn get_values_from_key_and_above(
        &self,
        key: Key,
    ) -> impl DoubleEndedIterator<Item = (&Key, &Value)> {
        self.map
            .range((Included(key), Unbounded))
            .flat_map(|(current_key, set)| set.iter().map(move |value| (current_key, value)))
    }

    /// Retrieves all the Values for the specified Key, and all the Keys below it, from the index
    pub fn get_values_from_key_and_below(
        &self,
        key: Key,
    ) -> impl DoubleEndedIterator<Item = (&Key, &Value)> {
        self.map
            .range((Unbounded, Included(key)))
            .flat_map(|(current_key, set)| set.iter().map(move |value| (current_key, value)))
    }

    /// Retrieves all the Keys for the specified Value from the index
    pub fn get_keys_from_value(&self, value: Value) -> Option<&BTreeSet<Key>> {
        self.reverse_map.get(&value)
    }

    /// Retrieves all the Keys for the specified Value, and all Values above it, from the index
    pub fn get_keys_from_value_and_above(
        &self,
        value: Value,
    ) -> impl DoubleEndedIterator<Item = (&Value, &Key)> {
        self.reverse_map
            .range((Included(value), Unbounded))
            .flat_map(|(current_value, set)| set.iter().map(move |key| (current_value, key)))
    }

    /// Retrieves all the Keys for the specified Value, and all Values below it, from the index
    pub fn get_keys_from_value_and_below(
        &self,
        value: Value,
    ) -> impl DoubleEndedIterator<Item = (&Value, &Key)> {
        self.reverse_map
            .range((Unbounded, Included(value)))
            .flat_map(|(current_value, set)| set.iter().map(move |key| (current_value, key)))
    }
}
