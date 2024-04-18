// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Small map in various sizes. These store a certain number of entries inline, and fall back
//! to a `HashMap` for larger allocations.
//!
//! Provides (almost) 1:1 API compatibility with [`HashMap`].
//!
//! # Examples
//!
//! ```
//! use compact_map::CompactMap;
//!
//! // Type inference lets us omit an explicit type signature except for the capacity
//! // (which would be `CompactMap<String, String, N>` in this example).
//! // Also with default capacity of 16.
//! let mut book_reviews = CompactMap::default();
//!
//! // Review some books.
//! book_reviews.insert(
//!     "Adventures of Huckleberry Finn".to_string(),
//!     "My favorite book.".to_string(),
//! );
//! book_reviews.insert(
//!     "Grimms' Fairy Tales".to_string(),
//!     "Masterpiece.".to_string(),
//! );
//! book_reviews.insert(
//!     "Pride and Prejudice".to_string(),
//!     "Very enjoyable.".to_string(),
//! );
//! book_reviews.insert(
//!     "The Adventures of Sherlock Holmes".to_string(),
//!     "Eye lyked it alot.".to_string(),
//! );
//!
//! // Check for a specific one.
//! // When collections store owned values (String), they can still be
//! // queried using references (&str).
//! if !book_reviews.contains_key("Les Misérables") {
//!     println!("We've got {} reviews, but Les Misérables ain't one.",
//!              book_reviews.len());
//! }
//!
//! // oops, this review has a lot of spelling mistakes, let's delete it.
//! book_reviews.remove("The Adventures of Sherlock Holmes");
//!
//! // Look up the values associated with some keys.
//! let to_find = ["Pride and Prejudice", "Alice's Adventure in Wonderland"];
//! for &book in &to_find {
//!     match book_reviews.get(book) {
//!         Some(review) => println!("{book}: {review}"),
//!         None => println!("{book} is unreviewed.")
//!     }
//! }
//!
//! // Look up the value for a key (will panic if the key is not found).
//! println!("Review for Jane: {}", book_reviews["Pride and Prejudice"]);
//!
//! // Iterate over everything.
//! for (book, review) in &book_reviews {
//!     println!("{book}: \"{review}\"");
//! }
//! ```
//!
//! A `CompactMap` with a known list of items can be initialized from an array:
//!
//! ```
//! use compact_map::CompactMap;
//!
//! // You need to specify the size of the map.
//! let solar_distance = CompactMap::<_, _, 4>::from([
//!     ("Mercury", 0.4),
//!     ("Venus", 0.7),
//!     ("Earth", 1.0),
//!     ("Mars", 1.5),
//! ]);
//! ```
//!
//! `CompactMap` implements an [`Entry` API](CompactMap::entry), which allows
//! for complex methods of getting, setting, updating and removing keys and
//! their values:
//!
//! ```
//! use compact_map::CompactMap;
//!
//! // type inference lets us omit an explicit type signature except for the capacity
//! // (which would be `CompactMap<&str, u8, N>` in this example).
//! let mut player_stats = CompactMap::<_, _, 5>::new();
//!
//! fn random_stat_buff() -> u8 {
//!     // could actually return some random value here - let's just return
//!     // some fixed value for now
//!     42
//! }
//!
//! // insert a key only if it doesn't already exist
//! player_stats.entry("health").or_insert(100);
//!
//! // insert a key using a function that provides a new value only if it
//! // doesn't already exist
//! player_stats.entry("defence").or_insert_with(random_stat_buff);
//!
//! // update a key, guarding against the key possibly not being set
//! let stat = player_stats.entry("attack").or_insert(100);
//! *stat += random_stat_buff();
//!
//! // modify an entry before an insert with in-place mutation
//! player_stats.entry("mana").and_modify(|mana| *mana += 200).or_insert(100);
//! ```
//!
//! ## Optional Features
//!
//! ### `map_entry_replace`
//!
//! **This feature is unstable and requires a nightly build of the Rust toolchain.**
//!
//! *This feature enables the `map_entry_replace` feature gate.*
//!
//! This feature enables the [`OccupiedEntry::replace_entry`] method,
//! it makes operations that would otherwise require two look-ups into operations that require only one.
//!
//! Tracking issue: [rust-lang/rust#44286](https://github.com/rust-lang/rust/issues/44286)
//!
//! ### `extract_if`
//!
//! **This feature is unstable and requires a nightly build of the Rust toolchain.**
//!
//! *This feature enables the `hash_extract_if` feature gate.*
//!
//! This feature enables the [`CompactMap::extract_if`] method,
//! provides a draining, filtering iterator over the entries of a Map.
//!
//! Tracking issue: [rust-lang/rust#59618](https://github.com/rust-lang/rust/issues/59618)
//!
//! ### `entry_insert`
//!
//! **This feature is unstable and requires a nightly build of the Rust toolchain.**
//!
//! *This feature enables the `entry_insert` feature gate.*
//!
//! This feature enables the [`Entry::insert_entry`] method.
//!
//! Tracking issue: [rust-lang/rust#65225](https://github.com/rust-lang/rust/issues/65225)
//!
//! ### `map_try_insert`
//!
//! **This feature is unstable and requires a nightly build of the Rust toolchain.**
//!
//! *This feature enables the `map_try_insert` feature gate.*
//!
//! This feature enables the [`CompactMap::try_insert`] method.
//!
//! Tracking issue: [rust-lang/rust#82766](https://github.com/rust-lang/rust/issues/82766)
//!
//! ### `many_mut`
//!
//! **This feature is unstable and requires a nightly build of the Rust toolchain.**
//!
//! *This feature enables the `map_many_mut`, `get_many_mut` feature gate.*
//!
//! This feature enables the [`CompactMap::get_many_mut`], [`CompactMap::get_many_unchecked_mut`] methods.
//!
//! Tracking issue:
//! - [rust-lang/rust#97601](https://github.com/rust-lang/rust/issues/97601)
//! - [rust-lang/rust#104642](https://github.com/rust-lang/rust/issues/104642)

#![deny(missing_docs)]
#![allow(clippy::manual_map)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "map_entry_replace", feature(map_entry_replace))] // issue 44286
#![cfg_attr(feature = "extract_if", feature(hash_extract_if))] // issue 59618
#![cfg_attr(feature = "entry_insert", feature(entry_insert))] // issue 65225
#![cfg_attr(feature = "map_try_insert", feature(map_try_insert))] // issue 82766
#![cfg_attr(feature = "many_mut", feature(map_many_mut))] // issue 97601
#![cfg_attr(feature = "many_mut", feature(get_many_mut))] // issue 104642

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};
use std::iter::FusedIterator;
use std::ops::Index;

mod base;
mod utils;
#[cfg(feature = "map_try_insert")]
pub use base::entry::OccupiedError;
pub use base::{
    entry::{Entry, OccupiedEntry, VacantEntry},
    TryReserveError,
};

const DEFAULT_MAX_INLINE_ENTRIES: usize = 16;

/// A map that inlines entries to avoid heap allocations for small maps.
pub struct CompactMap<K, V, const N: usize> {
    base: base::MapImpl<K, V, N>,
}

impl<K, V, const N: usize> CompactMap<K, V, N> {
    /// Creates an empty `CompactMap`.
    ///
    /// The compact map will be able to hold up to `N` entries without spilling to the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    /// let mut map: CompactMap<&str, i32, 16> = CompactMap::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base: base::MapImpl::new(),
        }
    }

    /// Returns `true` if the data has spilled into an std `HashMap`.
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<i32, i32, 2>  = CompactMap::new();
    /// assert!(!map.spilled());
    ///
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// map.insert(5, 6);
    /// assert!(map.spilled());
    /// ```
    #[inline(always)]
    pub const fn spilled(&self) -> bool {
        self.base.spilled()
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// When spilled, this number is a lower bound;
    /// the `CompactMap<K, V>` might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    /// let map: CompactMap<i32, i32, 100> = CompactMap::new();
    /// assert!(map.capacity() >= 100);
    /// ```
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.base.capacity()
    }

    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// for key in map.keys() {
    ///     println!("{key}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per docs in [HashMap::keys], iterating over keys takes O(capacity) time.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V, N> {
        Keys {
            inner: self.base.iter(),
        }
    }

    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// let mut vec: Vec<&str> = map.into_keys().collect();
    /// // The `IntoKeys` iterator produces keys in arbitrary order, so the
    /// // keys must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, ["a", "b", "c"]);
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V, N> {
        IntoKeys {
            inner: self.base.into_iter(),
        }
    }

    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// for val in map.values() {
    ///     println!("{val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    pub fn values(&self) -> Values<'_, K, V, N> {
        Values {
            base: self.base.iter(),
        }
    }

    /// An iterator visiting all values mutably in arbitrary order.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// for val in map.values_mut() {
    ///     *val = *val + 10;
    /// }
    ///
    /// for val in map.values() {
    ///     println!("{val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V, N> {
        ValuesMut {
            inner: self.base.iter_mut(),
        }
    }

    /// Creates a consuming iterator visiting all the values in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// let mut vec: Vec<i32> = map.into_values().collect();
    /// // The `IntoValues` iterator produces values in arbitrary order, so
    /// // the values must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V, N> {
        IntoValues {
            inner: self.base.into_iter(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {key} val: {val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V, N> {
        Iter {
            base: self.base.iter(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<&str, i32, 3> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// for (key, val) in &map {
    ///     println!("key: {key} val: {val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, N> {
        IterMut {
            base: self.base.iter_mut(),
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut a = CompactMap::default();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.base.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut a = CompactMap::default();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.base.is_empty()
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed, it
    /// drops the remaining key-value pairs. The returned iterator keeps a
    /// mutable borrow on the map to optimize its implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut a = CompactMap::default();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    ///
    /// for (k, v) in a.drain().take(1) {
    ///     assert!(k == 1 || k == 2);
    ///     assert!(v == "a" || v == "b");
    /// }
    ///
    /// assert!(a.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V, N> {
        Drain {
            base: self.base.drain(),
        }
    }

    /// Creates an iterator which uses a closure to determine if an element should be removed.
    ///
    /// If the closure returns true, the element is removed from the map and yielded.
    /// If the closure returns false, or panics, the element remains in the map and will not be
    /// yielded.
    ///
    /// Note that `extract_if` lets you mutate every value in the filter closure, regardless of
    /// whether you choose to keep or remove it.
    ///
    /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
    /// or the iteration short-circuits, then the remaining elements will be retained.
    /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
    ///
    /// [`retain`]: CompactMap::retain
    ///
    /// # Examples
    ///
    /// Splitting a map into even and odd keys, reusing the original map:
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<i32, i32, 8> = (0..8).map(|x| (x, x)).collect();
    /// let extracted: CompactMap<i32, i32, 8> = map.extract_if(|k, _v| k % 2 == 0).collect();
    ///
    /// let mut evens = extracted.keys().copied().collect::<Vec<_>>();
    /// let mut odds = map.keys().copied().collect::<Vec<_>>();
    /// evens.sort();
    /// odds.sort();
    ///
    /// assert_eq!(evens, vec![0, 2, 4, 6]);
    /// assert_eq!(odds, vec![1, 3, 5, 7]);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "extract_if")))]
    #[cfg(feature = "extract_if")]
    #[inline]
    pub fn extract_if<F>(&mut self, pred: F) -> ExtractIf<'_, K, V, F, N>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        ExtractIf {
            base: self.base.extract_if(pred),
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)` returns `false`.
    /// The elements are visited in unsorted (and unspecified) order.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<i32, i32, 16> = (0..8).map(|x| (x, x*10)).collect();
    /// map.retain(|&k, _| k % 2 == 0);
    /// assert_eq!(map.len(), 4);
    /// ```
    ///
    /// # Performance
    ///
    /// - When heapless: iterating over keys takes O(len) time.
    /// - When spilled: as per in [std docs](HashMap::keys), iterating over keys takes O(capacity) time.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.base.retain(f)
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut a = CompactMap::default();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.base.clear();
    }
}

impl<K, V, const N: usize> CompactMap<K, V, N>
where
    K: Eq + Hash,
{
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `CompactMap`. The collection may reserve more space to speculatively
    /// avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// If current variant is heapless and `self.len() + additional` is greater than `N`,
    /// the map will spill to [`HashMap`] immediately; otherwise, it's a no-op.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    /// let mut map: CompactMap<&str, i32, 16> = CompactMap::new();
    /// map.reserve(32);
    /// assert!(map.capacity() >= 32);
    /// assert!(map.spilled());
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.base.reserve(additional)
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the `HashMap`. The collection may reserve more space to speculatively
    /// avoid frequent reallocations. After calling `try_reserve`,
    /// capacity will be greater than or equal to `self.len() + additional` if
    /// it returns `Ok(())`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// If current variant is heapless and `self.len() + additional` is greater than `N`,
    /// the map will spill to [`HashMap`] immediately; otherwise, it's a no-op.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    /// let mut map: CompactMap<&str, i32, 16> = CompactMap::new();
    ///
    /// map.try_reserve(10).expect("why is the test harness OOMing on a handful of bytes?");
    /// ```
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.base.try_reserve(additional)
    }

    /// Manually spills to a [`HashMap`].
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    /// let mut map: CompactMap<i32, i32, 16> = CompactMap::new();
    /// map.spill();
    /// assert!(map.spilled());
    #[inline]
    pub fn spill(&mut self) {
        self.base.spill()
    }

    /// Shrinks the map into a heapless map with capacity `M`.
    ///
    /// # Errors
    ///
    /// If `M` is less than the current length of the map, the original map is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<i32, i32, 16> = CompactMap::new();
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// let map = map.shrink_into_heapless::<2>().unwrap();
    /// ```
    #[inline]
    pub fn shrink_into_heapless<const M: usize>(
        self,
    ) -> Result<CompactMap<K, V, M>, CompactMap<K, V, N>> {
        self.base
            .shrink_into_heapless()
            .map(|base| CompactMap { base })
            .map_err(|base| CompactMap { base })
    }

    /// This is a proxy to the underlying [`HashMap::shrink_to_fit`] method.
    /// And it's a no-op if the map is heapless.
    ///
    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<i32, i32, 10> = CompactMap::new();
    /// map.reserve(90);
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to_fit();
    /// assert!(map.capacity() >= 2);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.base.shrink_to_fit();
    }

    /// This is a proxy to the underlying [`HashMap::shrink_to`] method.
    /// And it's a no-op if the map is heapless.
    ///
    /// Shrinks the capacity of the map with a lower limit. It will drop
    /// down no lower than the supplied limit while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// If the current capacity is less than the lower limit, this is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map: CompactMap<i32, i32, 10> = CompactMap::new();
    /// map.reserve(90);
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to(10);
    /// assert!(map.capacity() >= 10);
    /// map.shrink_to(0);
    /// assert!(map.capacity() >= 2);
    /// ```
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.base.shrink_to(min_capacity);
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut letters = CompactMap::default();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     letters.entry(ch).and_modify(|counter| *counter += 1).or_insert(1);
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// ```
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, N> {
        self.base.entry(key)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.get(k)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    #[inline]
    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.get_key_value(k)
    }

    /// Attempts to get mutable references to `N` values in the map at once.
    ///
    /// Returns an array of length `N` with the results of each query. For soundness, at most one
    /// mutable reference will be returned to any value. `None` will be returned if any of the
    /// keys are duplicates or missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut libraries = CompactMap::default();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     Some([
    ///         &mut 1807,
    ///         &mut 1800,
    ///     ]),
    /// );
    ///
    /// // Missing keys result in None
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    /// ]);
    /// assert_eq!(got, None);
    ///
    /// // Duplicate keys result in None
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "Athenæum",
    /// ]);
    /// assert_eq!(got, None);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "many_mut")))]
    #[cfg(feature = "many_mut")]
    #[inline]
    pub fn get_many_mut<Q, const M: usize>(&mut self, ks: [&Q; M]) -> Option<[&'_ mut V; M]>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.get_many_mut(ks)
    }

    /// Attempts to get mutable references to `N` values in the map at once, without validating that
    /// the values are unique.
    ///
    /// Returns an array of length `N` with the results of each query. `None` will be returned if
    /// any of the keys are missing.
    ///
    /// For a safe alternative see [`get_many_mut`](Self::get_many_mut).
    ///
    /// # Safety
    ///
    /// Calling this method with overlapping keys is *[undefined behavior]* even if the resulting
    /// references are not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut libraries = CompactMap::default();
    /// libraries.insert("Bodleian Library".to_string(), 1602);
    /// libraries.insert("Athenæum".to_string(), 1807);
    /// libraries.insert("Herzogin-Anna-Amalia-Bibliothek".to_string(), 1691);
    /// libraries.insert("Library of Congress".to_string(), 1800);
    ///
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "Library of Congress",
    /// ]);
    /// assert_eq!(
    ///     got,
    ///     Some([
    ///         &mut 1807,
    ///         &mut 1800,
    ///     ]),
    /// );
    ///
    /// // Missing keys result in None
    /// let got = libraries.get_many_mut([
    ///     "Athenæum",
    ///     "New York Public Library",
    /// ]);
    /// assert_eq!(got, None);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "many_mut")))]
    #[cfg(feature = "many_mut")]
    #[inline]
    pub unsafe fn get_many_unchecked_mut<Q, const M: usize>(
        &mut self,
        ks: [&Q; M],
    ) -> Option<[&'_ mut V; M]>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.get_many_unchecked_mut(ks)
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    #[inline]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.contains_key(k)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.get_mut(k)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [module-level
    /// documentation] for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.base.insert(k, v)
    }

    /// Tries to insert a key-value pair into the map, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// If the map already had this key present, nothing is updated, and
    /// an error containing the occupied entry and the value is returned.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// assert_eq!(map.try_insert(37, "a").unwrap(), &"a");
    ///
    /// let err = map.try_insert(37, "b").unwrap_err();
    /// assert_eq!(err.entry.key(), &37);
    /// assert_eq!(err.entry.get(), &"a");
    /// assert_eq!(err.value, "b");
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "map_try_insert")))]
    #[cfg(feature = "map_try_insert")]
    pub fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedError<'_, K, V, N>> {
        match self.entry(key) {
            Entry::Occupied(entry) => Err(OccupiedError { entry, value }),
            Entry::Vacant(entry) => Ok(entry.insert(value)),
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let mut map = CompactMap::default();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.remove(k)
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// # fn main() {
    /// let mut map = CompactMap::default();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove(&1), None);
    /// # }
    /// ```
    #[inline]
    pub fn remove_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.base.remove_entry(k)
    }

    /// Converts the map into a [`HashMap`].
    ///
    /// If the map has spilled into a `HashMap`, this will return that `HashMap`.
    /// Otherwise, it will create a new `HashMap` and move all the entries into it.
    #[inline]
    pub fn into_hashmap(self) -> HashMap<K, V> {
        self.base.into_hashmap()
    }

    /// Converts the map into a [`HashMap`] with a given hasher.
    ///
    /// This will always create a new `HashMap` and move all the entries into it.
    ///
    /// See also [`HashMap::with_hasher`].
    #[inline]
    pub fn into_hashmap_with_hasher<S: BuildHasher>(self, hash_builder: S) -> HashMap<K, V, S> {
        let mut map = HashMap::with_capacity_and_hasher(self.len(), hash_builder);
        map.extend(self.base);
        map
    }

    /// Converts the map into a [`HashMap`] with at least the specified capacity, using
    /// `hasher` to hash the keys. The capacity will always be at least `self.len()`.
    ///
    /// This will always create a new `HashMap` and move all the entries into it.
    ///
    /// See also [`HashMap::with_capacity_and_hasher`].
    #[inline]
    pub fn into_hashmap_with_capacity_and_hasher<S: BuildHasher>(
        self,
        capacity: usize,
        hash_builder: S,
    ) -> HashMap<K, V, S> {
        let mut map = HashMap::with_capacity_and_hasher(capacity.max(self.len()), hash_builder);
        map.extend(self.base);
        map
    }
}

impl<K, V, const N: usize, const M: usize> PartialEq<CompactMap<K, V, M>> for CompactMap<K, V, N>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &CompactMap<K, V, M>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, const N: usize> Eq for CompactMap<K, V, N>
where
    K: Eq + Hash,
    V: Eq,
{
}

impl<K, V, const N: usize> Debug for CompactMap<K, V, N>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for CompactMap<K, V, DEFAULT_MAX_INLINE_ENTRIES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, Q: ?Sized, V, const N: usize> Index<&Q> for CompactMap<K, V, N>
where
    K: Eq + Hash + Borrow<Q>,
    Q: Eq + Hash,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `CompactMap`.
    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V, const N: usize, const M: usize> From<[(K, V); N]> for CompactMap<K, V, M>
where
    K: Eq + Hash,
{
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let map1: CompactMap<i32, i32, 16> = CompactMap::from([(1, 2), (3, 4)]);
    /// let map2: CompactMap<i32, i32, 32> = [(1, 2), (3, 4)].into();
    /// assert_eq!(map1, map2);
    /// ```
    fn from(arr: [(K, V); N]) -> Self {
        Self {
            base: base::MapImpl::from(arr),
        }
    }
}

/// An iterator over the keys of a `CompactMap`.
///
/// This `struct` is created by the [`keys`] method on [`CompactMap`]. See its
/// documentation for more.
///
/// [`keys`]: CompactMap::keys
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter_keys = map.keys();
/// ```
pub struct Keys<'a, K, V, const N: usize> {
    pub(crate) inner: base::iter::IterInner<'a, K, V, N>,
}

impl<K, V, const N: usize> Clone for Keys<'_, K, V, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<K: Debug, V, const N: usize> Debug for Keys<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V, const N: usize> Iterator for Keys<'a, K, V, N> {
    type Item = &'a K;

    #[inline]
    fn next(&mut self) -> Option<&'a K> {
        self.inner.next().map(|(k, _)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.inner.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, |acc, (k, _)| f(acc, k))
    }
}
impl<K, V, const N: usize> ExactSizeIterator for Keys<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}
impl<K, V, const N: usize> FusedIterator for Keys<'_, K, V, N> {}

/// An owning iterator over the keys of a `CompactMap`.
///
/// This `struct` is created by the [`into_keys`] method on [`CompactMap`].
/// See its documentation for more.
///
/// [`into_keys`]: CompactMap::into_keys
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter_keys = map.into_keys();
/// ```
pub struct IntoKeys<K, V, const N: usize> {
    pub(crate) inner: base::iter::IntoIterInner<K, V, N>,
}

impl<K, V, const N: usize> Iterator for IntoKeys<K, V, N> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<K> {
        self.inner.next().map(|(k, _)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.inner.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, |acc, (k, _)| f(acc, k))
    }
}
impl<K, V, const N: usize> ExactSizeIterator for IntoKeys<K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}
impl<K, V, const N: usize> FusedIterator for IntoKeys<K, V, N> {}

/// An iterator over the values of a `CompactMap`.
///
/// This `struct` is created by the [`values`] method on [`CompactMap`]. See its
/// documentation for more.
///
/// [`values`]: CompactMap::values
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter_values = map.values();
/// ```
pub struct Values<'a, K: 'a, V: 'a, const N: usize> {
    base: base::iter::IterInner<'a, K, V, N>,
}

impl<'a, K, V, const N: usize> Iterator for Values<'a, K, V, N> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<&'a V> {
        self.base.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.base.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.base.fold(init, |acc, (_, v)| f(acc, v))
    }
}
impl<K, V, const N: usize> ExactSizeIterator for Values<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }
}
impl<K, V, const N: usize> FusedIterator for Values<'_, K, V, N> {}

/// A mutable iterator over the values of a `CompactMap`.
///
/// This `struct` is created by the [`values_mut`] method on [`CompactMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: CompactMap::values_mut
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let mut map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter_values = map.values_mut();
/// ```
pub struct ValuesMut<'a, K: 'a, V: 'a, const N: usize> {
    inner: base::iter::IterMutInner<'a, K, V, N>,
}

impl<'a, K, V, const N: usize> Iterator for ValuesMut<'a, K, V, N> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<&'a mut V> {
        self.inner.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.inner.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, |acc, (_, v)| f(acc, v))
    }
}
impl<K, V, const N: usize> ExactSizeIterator for ValuesMut<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}
impl<K, V, const N: usize> FusedIterator for ValuesMut<'_, K, V, N> {}

/// An owning iterator over the values of a `CompactMap`.
///
/// This `struct` is created by the [`into_values`] method on [`CompactMap`].
/// See its documentation for more.
///
/// [`into_values`]: HashMap::into_values
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter_keys = map.into_values();
/// ```
pub struct IntoValues<K, V, const N: usize> {
    inner: base::iter::IntoIterInner<K, V, N>,
}

impl<K, V, const N: usize> Iterator for IntoValues<K, V, N> {
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<V> {
        self.inner.next().map(|(_, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.inner.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, |acc, (_, v)| f(acc, v))
    }
}
impl<K, V, const N: usize> ExactSizeIterator for IntoValues<K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}
impl<K, V, const N: usize> FusedIterator for IntoValues<K, V, N> {}

/// An iterator over the entries of a `CompactMap`.
///
/// This `struct` is created by the [`iter`] method on [`CompactMap`]. See its
/// documentation for more.
///
/// [`iter`]: CompactMap::iter
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter = map.iter();
/// ```
pub struct Iter<'a, K, V, const N: usize> {
    pub(crate) base: base::iter::IterInner<'a, K, V, N>,
}

impl<'a, K, V, const N: usize> Iterator for Iter<'a, K, V, N> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.base.count()
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.base.fold(init, f)
    }
}
impl<'a, K, V, const N: usize> ExactSizeIterator for Iter<'a, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }
}
impl<'a, K, V, const N: usize> FusedIterator for Iter<'a, K, V, N> {}

/// A mutable iterator over the entries of a `CompactMap`.
///
/// This `struct` is created by the [`iter_mut`] method on [`CompactMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: CompactMap::iter_mut
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let mut map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter = map.iter_mut();
/// ```
pub struct IterMut<'a, K: 'a, V: 'a, const N: usize> {
    base: base::iter::IterMutInner<'a, K, V, N>,
}

impl<'a, K, V, const N: usize> Iterator for IterMut<'a, K, V, N> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.base.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.base.fold(init, f)
    }
}
impl<K, V, const N: usize> ExactSizeIterator for IterMut<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }
}
impl<K, V, const N: usize> FusedIterator for IterMut<'_, K, V, N> {}

/// An owning iterator over the entries of a `CompactMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`CompactMap`]
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
/// ]);
/// let iter = map.into_iter();
/// ```
pub struct IntoIter<K, V, const N: usize> {
    base: base::iter::IntoIterInner<K, V, N>,
}

impl<K: Debug, V: Debug, const N: usize> Debug for IntoIter<K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.base.fmt(f)
    }
}

impl<K, V, const N: usize> Iterator for IntoIter<K, V, N> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.base.len()
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.base.fold(init, f)
    }
}
impl<K, V, const N: usize> ExactSizeIterator for IntoIter<K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }
}
impl<K, V, const N: usize> FusedIterator for IntoIter<K, V, N> {}

impl<'a, K, V, const N: usize> IntoIterator for &'a CompactMap<K, V, N> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, N>;

    #[inline]
    fn into_iter(self) -> Iter<'a, K, V, N> {
        self.iter()
    }
}

impl<'a, K, V, const N: usize> IntoIterator for &'a mut CompactMap<K, V, N> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V, N>;

    #[inline]
    fn into_iter(self) -> IterMut<'a, K, V, N> {
        self.iter_mut()
    }
}

/// A draining iterator over the entries of a `CompactMap`.
///
/// This `struct` is created by the [`drain`] method on [`CompactMap`]. See its
/// documentation for more.
///
/// [`drain`]: CompactMap::drain
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let mut map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
///     ("b", 2),
///     ("c", 3),
/// ]);
/// let iter = map.drain();
/// ```
pub struct Drain<'a, K: 'a, V: 'a, const N: usize> {
    base: base::drain::DrainInner<'a, K, V, N>,
}

impl<K: Debug, V: Debug, const N: usize> Debug for Drain<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.base.fmt(f)
    }
}

impl<'a, K, V, const N: usize> Iterator for Drain<'a, K, V, N> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.base.count()
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.base.fold(init, f)
    }
}
impl<K, V, const N: usize> ExactSizeIterator for Drain<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }
}
impl<K, V, const N: usize> FusedIterator for Drain<'_, K, V, N> {}

/// A draining, filtering iterator over the entries of a `CompactMap`.
///
/// This `struct` is created by the [`extract_if`] method on [`CompactMap`].
///
/// [`extract_if`]: CompactMap::extract_if
///
/// # Example
///
/// ```
/// use compact_map::CompactMap;
///
/// let mut map: CompactMap<&str, i32, 16> = CompactMap::from([
///     ("a", 1),
///     ("b", 2),
///     ("c", 3),
/// ]);
/// let iter = map.extract_if(|_k, v| *v % 2 == 0);
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "extract_if")))]
#[cfg(feature = "extract_if")]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ExtractIf<'a, K, V, F, const N: usize>
where
    F: FnMut(&K, &mut V) -> bool,
{
    base: base::extract_if::ExtractIfInner<'a, K, V, F, N>,
}

#[cfg_attr(docsrs, doc(cfg(feature = "extract_if")))]
#[cfg(feature = "extract_if")]
impl<K, V, F, const N: usize> Iterator for ExtractIf<'_, K, V, F, N>
where
    F: FnMut(&K, &mut V) -> bool,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.base.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "extract_if")))]
#[cfg(feature = "extract_if")]
impl<K, V, F, const N: usize> FusedIterator for ExtractIf<'_, K, V, F, N> where
    F: FnMut(&K, &mut V) -> bool
{
}

#[cfg_attr(docsrs, doc(cfg(feature = "extract_if")))]
#[cfg(feature = "extract_if")]
impl<'a, K, V, F, const N: usize> Debug for ExtractIf<'a, K, V, F, N>
where
    F: FnMut(&K, &mut V) -> bool,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtractIf").finish_non_exhaustive()
    }
}

impl<K, V, const N: usize> IntoIterator for CompactMap<K, V, N> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, N>;

    /// Creates a consuming iterator, that is, one that moves each key-value
    /// pair out of the map in arbitrary order. The map cannot be used after
    /// calling this.
    ///
    /// # Examples
    ///
    /// ```
    /// use compact_map::CompactMap;
    ///
    /// let map: CompactMap<&str, i32, 16> = CompactMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// // Not possible with .iter()
    /// let vec: Vec<(&str, i32)> = map.into_iter().collect();
    /// ```
    #[inline]
    fn into_iter(self) -> IntoIter<K, V, N> {
        IntoIter {
            base: self.base.into_iter(),
        }
    }
}

impl<K, V, const N: usize> FromIterator<(K, V)> for CompactMap<K, V, N>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = CompactMap::new();
        map.extend(iter);
        map
    }
}

impl<K, V, const N: usize> Extend<(K, V)> for CompactMap<K, V, N>
where
    K: Eq + Hash,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.base.extend(iter);
    }
}
