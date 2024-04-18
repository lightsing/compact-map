use crate::base::MapImpl;
use std::collections::hash_map::{
    OccupiedEntry as HashMapOccupiedEntry, VacantEntry as HashMapVacantEntry,
};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::hint::unreachable_unchecked;

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`CompactMap`].
///
/// [`entry`]: crate::CompactMap::entry
/// [`CompactMap`]: crate::CompactMap
pub enum Entry<'a, K: 'a, V: 'a, const N: usize> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, N>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, N>),
}

impl<K: Debug, V: Debug, const N: usize> Debug for Entry<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Entry::Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Entry::Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

/// A view into an occupied entry in a `CompactMap`.
/// It is part of the [`Entry`] enum.
pub enum OccupiedEntry<'a, K: 'a, V: 'a, const N: usize> {
    /// An entry in the heapless state.
    Heapless(HeaplessEntry<'a, K, V, N>),
    /// An entry in the spilled state.
    Spilled(HashMapOccupiedEntry<'a, K, V>),
}

impl<K: Debug, V: Debug, const N: usize> Debug for OccupiedEntry<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish_non_exhaustive()
    }
}

/// A view into a vacant entry in a `CompactMap`.
/// It is part of the [`Entry`] enum.
pub enum VacantEntry<'a, K: 'a, V: 'a, const N: usize> {
    /// An entry in the heapless state.
    Heapless(HeaplessEntry<'a, K, V, N>),
    /// An entry in the spilled state.
    Spilled(HashMapVacantEntry<'a, K, V>),
}

impl<K: Debug, V, const N: usize> Debug for VacantEntry<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").field(self.key()).finish()
    }
}

/// The error returned by [`try_insert`](crate::CompactMap::try_insert) when the key already exists.
///
/// Contains the occupied entry, and the value that was not inserted.
#[cfg(feature = "map_try_insert")]
pub struct OccupiedError<'a, K: 'a, V: 'a, const N: usize> {
    /// The entry in the map that was already occupied.
    pub entry: OccupiedEntry<'a, K, V, N>,
    /// The value which was not inserted, because the entry was already occupied.
    pub value: V,
}

#[cfg(feature = "map_try_insert")]
impl<K: Debug, V: Debug, const N: usize> Debug for OccupiedError<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedError")
            .field("key", self.entry.key())
            .field("old_value", self.entry.get())
            .field("new_value", &self.value)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "map_try_insert")]
impl<'a, K: Debug, V: Debug, const N: usize> fmt::Display for OccupiedError<'a, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to insert {:?}, key {:?} already exists with value {:?}",
            self.value,
            self.entry.key(),
            self.entry.get(),
        )
    }
}

#[cfg(feature = "map_try_insert")]
impl<'a, K: Debug, V: Debug, const N: usize> std::error::Error for OccupiedError<'a, K, V, N> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "key already exists"
    }
}

/// A view into an entry in a `CompactMap`.
/// It is part of the [`Entry`] enum.
pub struct HeaplessEntry<'a, K: 'a, V: 'a, const N: usize> {
    pub(crate) index: usize,
    pub(crate) key: Option<K>,
    pub(crate) inner: &'a mut MapImpl<K, V, N>,
}

impl<'a, K, V, const N: usize> Entry<'a, K, V, N>
where
    K: Eq + Hash,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    ///
    /// map.entry("poneyland").or_insert(3);
    /// assert_eq!(map["poneyland"], 3);
    ///
    /// *map.entry("poneyland").or_insert(10) *= 2;
    /// assert_eq!(map["poneyland"], 6);
    /// ```
    #[inline]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// let value = "hoho";
    ///
    /// map.entry("poneyland").or_insert_with(|| value);
    ///
    /// assert_eq!(map["poneyland"], "hoho");
    /// ```
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    ///
    /// map.entry("poneyland").or_insert_with_key(|key| key.chars().count());
    ///
    /// assert_eq!(map["poneyland"], 9usize);
    /// ```
    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Sets the value of the entry, and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, String, 16> = CompactMap::new();
    /// let entry = map.entry("poneyland").insert_entry("hoho".to_string());
    ///
    /// assert_eq!(entry.key(), &"poneyland");
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "entry_insert")))]
    #[cfg(feature = "entry_insert")]
    #[inline]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, N> {
        match self {
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
            Entry::Vacant(entry) => entry.insert_entry(value),
        }
    }
}

impl<'a, K, V, const N: usize> Entry<'a, K, V, N> {
    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        match *self {
            Entry::Occupied(ref entry) => entry.key(),
            Entry::Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 42);
    ///
    /// map.entry("poneyland")
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map["poneyland"], 43);
    /// ```
    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K, V: Default, const N: usize> Entry<'a, K, V, N>
where
    K: Eq + Hash,
{
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() {
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, Option<u32>, 16> = CompactMap::new();
    /// map.entry("poneyland").or_default();
    ///
    /// assert_eq!(map["poneyland"], None);
    /// # }
    /// ```
    #[inline]
    pub fn or_default(self) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

impl<'a, K, V, const N: usize> OccupiedEntry<'a, K, V, N> {
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        match self {
            Self::Heapless(entry) => entry.key(),
            Self::Spilled(entry) => entry.key(),
        }
    }

    /// Take the ownership of the key and value from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     // We delete the entry from the map.
    ///     o.remove_entry();
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// ```
    #[inline]
    pub fn remove_entry(self) -> (K, V) {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: the entry is occupied
                unsafe {
                    entry
                        .inner
                        .as_heapless_mut_unchecked()
                        .swap_remove_unchecked(entry.index)
                }
            }
            Self::Spilled(entry) => entry.remove_entry(),
        }
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    #[inline]
    pub fn get(&self) -> &V {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: the entry is occupied
                unsafe { entry.get_unchecked() }
            }
            Self::Spilled(entry) => entry.get(),
        }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: Self::into_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     *o.get_mut() += 10;
    ///     assert_eq!(*o.get(), 22);
    ///
    ///     // We can use the same Entry multiple times.
    ///     *o.get_mut() += 2;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 24);
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: the entry is occupied
                unsafe { entry.get_unchecked_mut() }
            }
            Self::Spilled(entry) => entry.get_mut(),
        }
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: Self::get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// assert_eq!(map["poneyland"], 12);
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     *o.into_mut() += 10;
    /// }
    ///
    /// assert_eq!(map["poneyland"], 22);
    /// ```
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        match self {
            Self::Heapless(HeaplessEntry { index, inner, .. }) => {
                // SAFETY: the entry is occupied
                unsafe { &mut inner.as_heapless_mut_unchecked().get_unchecked_mut(index).1 }
            }
            Self::Spilled(entry) => entry.into_mut(),
        }
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(mut o) = map.entry("poneyland") {
    ///     assert_eq!(o.insert(15), 12);
    /// }
    ///
    /// assert_eq!(map["poneyland"], 15);
    /// ```
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: the entry is occupied
                unsafe { std::mem::replace(entry.get_unchecked_mut(), value) }
            }
            Self::Spilled(entry) => entry.insert(value),
        }
    }

    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// map.entry("poneyland").or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry("poneyland") {
    ///     assert_eq!(o.remove(), 12);
    /// }
    ///
    /// assert_eq!(map.contains_key("poneyland"), false);
    /// ```
    #[inline]
    pub fn remove(self) -> V {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: the entry is occupied
                unsafe {
                    entry
                        .inner
                        .as_heapless_mut_unchecked()
                        .swap_remove_unchecked(entry.index)
                        .1
                }
            }
            Self::Spilled(entry) => entry.remove(),
        }
    }
}

impl<'a, K: Clone, V, const N: usize> OccupiedEntry<'a, K, V, N> {
    /// Replaces the entry, returning the old key and value. The new key in the hash map will be
    /// the key used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    /// use std::rc::Rc;
    ///
    /// let mut map: CompactMap<Rc<String>, u32, 16> = CompactMap::new();
    /// map.insert(Rc::new("Stringthing".to_string()), 15);
    ///
    /// let my_key = Rc::new("Stringthing".to_string());
    ///
    /// if let Entry::Occupied(entry) = map.entry(my_key) {
    ///     // Also replace the key with a handle to our other key.
    ///     let (old_key, old_value): (Rc<String>, u32) = entry.replace_entry(16);
    /// }
    ///
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "map_entry_replace")))]
    #[cfg(feature = "map_entry_replace")]
    #[inline]
    pub fn replace_entry(self, value: V) -> (K, V) {
        match self {
            Self::Heapless(mut entry) => {
                let key = entry.key_owned();
                // SAFETY: it is in heapless state
                let vec = unsafe { entry.inner.as_heapless_mut_unchecked() };
                // SAFETY: the entry is occupied
                let (old_key, old_value) = unsafe { vec.swap_remove_unchecked(entry.index) };
                // SAFETY: We just removed an element, so the push is safe
                unsafe {
                    vec.push((key, value)).unwrap_unchecked();
                }
                (old_key, old_value)
            }
            Self::Spilled(entry) => entry.replace_entry(value),
        }
    }

    /// Replaces the key in the hash map with the key used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    /// use std::rc::Rc;
    ///
    /// let mut map: CompactMap<Rc<String>, u32, 16> = CompactMap::new();
    /// let known_strings: Vec<Rc<String>> = Vec::new();
    ///
    /// // Initialise known strings, run program, etc.
    ///
    /// reclaim_memory(&mut map, &known_strings);
    ///
    /// fn reclaim_memory(map: &mut CompactMap<Rc<String>, u32, 16>, known_strings: &[Rc<String>] ) {
    ///     for s in known_strings {
    ///         if let Entry::Occupied(entry) = map.entry(Rc::clone(s)) {
    ///             // Replaces the entry's key with our version of it in `known_strings`.
    ///             entry.replace_key();
    ///         }
    ///     }
    /// }
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "map_entry_replace")))]
    #[cfg(feature = "map_entry_replace")]
    #[inline]
    pub fn replace_key(self) -> K {
        match self {
            Self::Heapless(mut entry) => {
                let key = entry.key_owned();
                // SAFETY: it is in heapless state
                let vec = unsafe { entry.inner.as_heapless_mut_unchecked() };
                // SAFETY: the entry is occupied
                let (old_key, value) = unsafe { vec.swap_remove_unchecked(entry.index) };
                // SAFETY: We just removed an element, so the push is safe
                unsafe {
                    vec.push_unchecked((key, value));
                }
                old_key
            }
            Self::Spilled(entry) => entry.replace_key(),
        }
    }
}

impl<'a, K: 'a, V: 'a, const N: usize> VacantEntry<'a, K, V, N> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    /// assert_eq!(map.entry("poneyland").key(), &"poneyland");
    /// ```
    #[inline]
    pub fn key(&self) -> &K {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: vacant entry always has a key
                unsafe { entry.key_unchecked() }
            }
            Self::Spilled(entry) => entry.key(),
        }
    }

    /// Take ownership of the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    ///
    /// if let Entry::Vacant(v) = map.entry("poneyland") {
    ///     v.into_key();
    /// }
    /// ```
    #[inline]
    pub fn into_key(self) -> K {
        match self {
            Self::Heapless(entry) => {
                // SAFETY: vacant entry always has a key
                unsafe { entry.key.unwrap_unchecked() }
            }
            Self::Spilled(entry) => entry.into_key(),
        }
    }
}

impl<'a, K: 'a, V: 'a, const N: usize> VacantEntry<'a, K, V, N>
where
    K: Eq + Hash,
{
    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    ///
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     o.insert(37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        match self {
            Self::Heapless(HeaplessEntry { index, key, inner }) => {
                // SAFETY: vacant entry always has a key
                let k = unsafe { key.unwrap_unchecked() };
                // SAFETY: HeaplessEntry only constructed when the in heapless state
                let vec_is_full = unsafe { inner.as_heapless_unchecked().is_full() };
                if !vec_is_full {
                    let vec = unsafe { inner.as_heapless_mut_unchecked() };
                    // SAFETY: We just checked that the vec is not full
                    unsafe { vec.push_unchecked((k, value)) };
                    debug_assert!(vec.len() - 1 == index);
                    // SAFETY: index is in bounds
                    unsafe { &mut vec.get_unchecked_mut(index).1 }
                } else {
                    // SAFETY: current in heapless
                    let map = unsafe { inner.try_spill(1) };
                    map.unwrap().entry(k).or_insert(value)
                }
            }
            Self::Spilled(entry) => entry.insert(value),
        }
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns an `OccupiedEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::{CompactMap, Entry};
    ///
    /// let mut map: CompactMap<&str, u32, 16> = CompactMap::new();
    ///
    /// if let Entry::Vacant(o) = map.entry("poneyland") {
    ///     o.insert_entry(37);
    /// }
    /// assert_eq!(map["poneyland"], 37);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "entry_insert")))]
    #[cfg(feature = "entry_insert")]
    #[inline]
    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V, N> {
        match self {
            Self::Heapless(HeaplessEntry { index, key, inner }) => {
                // SAFETY: vacant entry always has a key
                let k = unsafe { key.unwrap_unchecked() };
                // SAFETY: HeaplessEntry only constructed when the in heapless state
                let vec = unsafe { inner.as_heapless_mut_unchecked() };
                if !vec.is_full() {
                    // SAFETY: We just checked that the vec is not full
                    unsafe { vec.push_unchecked((k, value)) };
                    debug_assert!(vec.len() - 1 == index);
                    OccupiedEntry::Heapless(HeaplessEntry {
                        index,
                        key: None,
                        inner,
                    })
                } else {
                    // SAFETY: current in heapless
                    let map = unsafe { inner.try_spill(1) };
                    OccupiedEntry::Spilled(map.unwrap().entry(k).insert_entry(value))
                }
            }
            Self::Spilled(entry) => OccupiedEntry::Spilled(entry.insert_entry(value)),
        }
    }
}

impl<K, V, const N: usize> HeaplessEntry<'_, K, V, N> {
    #[inline]
    fn key(&self) -> &K {
        match self.key {
            Some(ref k) => k,
            None => {
                // SAFETY: vacant entry always has a key
                unsafe {
                    &self
                        .inner
                        .as_heapless_unchecked()
                        .get_unchecked(self.index)
                        .0
                }
            }
        }
    }

    /// # Safety
    ///
    /// Must be called when key is Some.
    #[inline]
    unsafe fn key_unchecked(&self) -> &K {
        match self.key {
            Some(ref k) => k,
            None => unreachable_unchecked(),
        }
    }

    /// # Safety
    ///
    /// Must be called when the entry is occupied.
    #[inline]
    unsafe fn get_unchecked(&self) -> &V {
        &self
            .inner
            .as_heapless_unchecked()
            .get_unchecked(self.index)
            .1
    }

    /// # Safety
    ///
    /// Must be called when the entry is occupied.
    #[inline]
    unsafe fn get_unchecked_mut(&mut self) -> &mut V {
        &mut self
            .inner
            .as_heapless_mut_unchecked()
            .get_unchecked_mut(self.index)
            .1
    }
}

#[cfg(feature = "map_entry_replace")]
impl<K: Clone, V, const N: usize> HeaplessEntry<'_, K, V, N> {
    #[inline]
    fn key_owned(&mut self) -> K {
        match self.key.take() {
            Some(k) => k,
            None => {
                // SAFETY: vacant entry always has a key
                unsafe {
                    self.inner
                        .as_heapless_mut_unchecked()
                        .get_unchecked(self.index)
                        .0
                        .clone()
                }
            }
        }
    }
}
