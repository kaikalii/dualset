use std::{
    borrow::Borrow,
    collections::hash_map,
    hash::{Hash, Hasher},
    ops::Index,
};

use nohash_hasher::IntMap;

use crate::Dual;

/// A set of values that can be accessed by their key
///
/// Values in this set must implement the [`Dual`] trait.
///
/// Unlike [`std::collections::HashSet`] or [`std::collections::HashMap`], modifying a
/// key in a way that changes its hash is *not* a logic error. The item's place in the
/// set will be updated to reflect the new key.
#[derive(Clone)]
pub struct DualHashSet<T>(IntMap<u64, T>);

impl<T> Default for DualHashSet<T> {
    fn default() -> Self {
        Self(IntMap::default())
    }
}

impl<T> DualHashSet<T> {
    /// Create a new set
    pub fn new() -> Self {
        Self::default()
    }
    /// Get an iterator over the keys
    pub fn keys(&self) -> Keys<T> {
        Keys(self.0.values())
    }
    /// Get an iterator over the values
    pub fn iter(&self) -> Iter<T> {
        Iter(self.0.values())
    }
}

impl<T: Dual> DualHashSet<T> {
    /// Remove a value from the set
    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        let key = hash(key);
        self.0.remove(&key)
    }
    /// Check if the set contains a value with the given key
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        let key = hash(key);
        self.0.contains_key(&key)
    }
    /// Get a value from the set
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        let key = hash(key);
        self.0.get(&key)
    }
}

impl<T> DualHashSet<T>
where
    T: Dual,
    T::Key: Hash + Eq,
{
    /// Insert a value into the set
    pub fn insert(&mut self, value: T) -> Option<T> {
        let key = hash(value.key());
        self.0.insert(key, value)
    }
    /// Modify a value in the set.
    /// If the key changes, the value will be moved to the new key.
    pub fn modify<Q, F, R>(&mut self, key: &Q, mut f: F) -> Option<R>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
        F: FnMut(&mut T) -> R,
    {
        let key = hash(key);
        if let Some(value) = self.0.get_mut(&key) {
            let res = f(value);
            let new_key = hash(value.key());
            if new_key != key {
                let value = self.0.remove(&key).unwrap();
                self.0.insert(new_key, value);
            }
            Some(res)
        } else {
            None
        }
    }
}

impl<Q, T> Index<&Q> for DualHashSet<T>
where
    Q: Hash + Eq + ?Sized,
    T: Dual,
    T::Key: Hash + Eq + Borrow<Q>,
{
    type Output = T;
    #[track_caller]
    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("key not found")
    }
}

pub struct Keys<'a, T>(hash_map::Values<'a, u64, T>);
pub struct Iter<'a, T>(hash_map::Values<'a, u64, T>);
pub struct IntoIter<T>(hash_map::IntoValues<u64, T>);

impl<'a, T: Dual> Iterator for Keys<'a, T> {
    type Item = &'a T::Key;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|value| value.key())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> IntoIterator for DualHashSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_values())
    }
}

impl<'a, T> IntoIterator for &'a DualHashSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.values())
    }
}

fn hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(PartialEq, Eq)]
    struct Test {
        key: String,
        value: u32,
    }

    impl Dual for Test {
        type Key = String;
        fn key(&self) -> &Self::Key {
            &self.key
        }
    }

    #[test]
    fn modify() {
        let mut set = DualHashSet::new();
        for i in 0..10 {
            set.insert(Test {
                key: i.to_string(),
                value: i,
            });
        }
        assert_eq!(set["3"].key, "3");
        assert_eq!(set["3"].value, 3);
        assert_eq!(set["4"].key, "4");
        assert_eq!(set["4"].value, 4);
        set.modify("3", |test| test.value += 1);
        set.modify("4", |test| test.key = "four".into());
        assert_eq!(set["3"].key, "3");
        assert_eq!(set["3"].value, 4);
        assert!(!set.contains("4"));
        assert_eq!(set["four"].key, "four");
        assert_eq!(set["four"].value, 4);
    }
}
