use std::{
    borrow::{Borrow, BorrowMut},
    collections::{hash_map, HashMap},
    hash::Hash,
    ops::{Deref, DerefMut, Index},
};

use crate::Dual;

/// A set of values that can be accessed by their key
///
/// Values in this set must implement the [`Dual`] trait,
/// and their key type must implement [`Hash`].
///
/// Unlike [`std::collections::HashSet`] or [`std::collections::HashMap`], modifying a
/// key in a way that changes its hash is *not* a logic error. The item's place in the
/// set will be updated to reflect the new key.
#[derive(Clone)]
pub struct DualHashSet<T: Dual>(HashMap<T::Key, T>);

impl<T: Dual> Default for DualHashSet<T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<T: Dual> DualHashSet<T> {
    /// Create a new set
    #[must_use]
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

impl<T> DualHashSet<T>
where
    T: Dual,
    T::Key: Hash,
{
    /// Insert a value into the set
    pub fn insert(&mut self, value: T) -> Option<T> {
        self.0.insert(value.key().clone(), value)
    }
    /// Remove a value from the set
    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        self.0.remove(key)
    }
    /// Get the number of values in the set
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Check if the set is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Remove all items from the set
    pub fn clear(&mut self) {
        self.0.clear()
    }
    /// Check if the set contains a value with the given key
    #[must_use]
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        self.0.contains_key(key)
    }
    /// Get a value from the set
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        self.0.get(key)
    }
    /// Get a mutable reference to a value in the set
    ///
    /// When the reference is dropped, the value will be moved to the new
    /// key if it has changed.
    ///
    /// For simple modifications, prefer [`DualHashSet::modify`].
    #[allow(clippy::manual_map)]
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<DualHashSetRef<T>>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
    {
        if let Some(value) = self.get(key) {
            Some(DualHashSetRef {
                key: value.key().clone(),
                set: self,
            })
        } else {
            None
        }
    }
    /// Get a value from the set, or insert a new value if it does not exist
    pub fn get_or_insert_with<F>(&mut self, key: T::Key, f: F) -> DualHashSetRef<T>
    where
        F: FnOnce(T::Key) -> T,
    {
        if !self.contains(&key) {
            self.insert(f(key.clone()));
        }
        DualHashSetRef { key, set: self }
    }
    /// Modify a value in the set.
    /// If the key changes, the value will be moved to the new key.
    pub fn modify<Q, F, R>(&mut self, key: &Q, mut f: F) -> Option<R>
    where
        Q: Hash + Eq + ?Sized,
        T::Key: Borrow<Q>,
        F: FnMut(&mut T) -> R,
    {
        if let Some(value) = self.0.get_mut(key) {
            let res = f(value);
            let new_key = value.key();
            if new_key.borrow() != key {
                let new_key = new_key.clone();
                let value = self.0.remove(key).unwrap();
                self.0.insert(new_key, value);
            }
            Some(res)
        } else {
            None
        }
    }
    /// Modify every value in the set.
    /// If a key changes, the value will be moved to the new key.
    pub fn modify_all<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        self.retain(|value| {
            f(value);
            true
        });
    }
    /// Remove all values from the set that do not satisfy the predicate
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        for key in self.keys().cloned().collect::<Vec<_>>() {
            let value = self.0.get_mut(&key).unwrap();
            let keep = predicate(value);
            if !keep || value.key() != &key {
                let value = self.0.remove(&key).unwrap();
                if keep {
                    self.0.insert(value.key().clone(), value);
                }
            }
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

/// Iterator returned by [`DualHashSet::keys`]
#[must_use]
pub struct Keys<'a, T: Dual>(hash_map::Values<'a, T::Key, T>);
/// Iterator returned by [`DualHashSet::iter`]
#[must_use]
pub struct Iter<'a, T: Dual>(hash_map::Values<'a, T::Key, T>);
/// Iterator returned by [`DualHashSet::into_iter`]
#[must_use]
pub struct IntoIter<T: Dual>(hash_map::IntoValues<T::Key, T>);

impl<'a, T: Dual> Iterator for Keys<'a, T> {
    type Item = &'a T::Key;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|value| value.key())
    }
}

impl<'a, T: Dual> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T: Dual> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T: Dual> IntoIterator for DualHashSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_values())
    }
}

impl<'a, T: Dual> IntoIterator for &'a DualHashSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.values())
    }
}

/// A mutable reference to a value in a [`DualHashSet`]
///
/// When the reference is dropped, the value will be moved to the new
/// key if it has changed.
#[must_use]
pub struct DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    set: &'a mut DualHashSet<T>,
    key: T::Key,
}

impl<'a, T> Deref for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.set.0.get(&self.key).unwrap()
    }
}

impl<'a, T> DerefMut for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set.0.get_mut(&self.key).unwrap()
    }
}

impl<'a, T> AsRef<T> for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    fn as_ref(&self) -> &T {
        self
    }
}

impl<'a, T> AsMut<T> for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T> Borrow<T> for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    fn borrow(&self) -> &T {
        self
    }
}

impl<'a, T> BorrowMut<T> for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T> Drop for DualHashSetRef<'a, T>
where
    T: Dual,
    T::Key: Hash,
{
    fn drop(&mut self) {
        let new_key = self.key();
        if new_key != &self.key {
            let new_key = new_key.clone();
            let value = self.set.0.remove(&self.key).unwrap();
            self.set.0.insert(new_key, value);
        }
    }
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

    #[test]
    fn get_mut() {
        let mut set = DualHashSet::new();
        for i in 0..10 {
            set.insert(Test {
                key: i.to_string(),
                value: i,
            });
        }

        let mut value = set.get_mut("3").unwrap();
        (*value).key = "three".into();
        drop(value);
        assert!(!set.contains("3"));
        assert_eq!(set["three"].key, "three");
    }
    #[test]
    fn retain() {
        let mut set = DualHashSet::new();
        for i in 0..10 {
            set.insert(Test {
                key: i.to_string(),
                value: i,
            });
        }
        set.retain(|test| {
            test.key = format!("{}{}", test.value, test.value);
            test.value % 2 == 0
        });
        assert_eq!(set.len(), 5);
        for i in 0..10 {
            if i % 2 == 0 {
                assert!(set.contains(&format!("{i}{i}")));
            }
            assert!(!set.contains(&format!("{i}")));
        }
    }
}
