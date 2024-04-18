use crate::base::{
    drain::{DrainInner, HeaplessDrain},
    entry::{Entry, HeaplessEntry, OccupiedEntry, VacantEntry},
    iter::{IntoIterInner, IterInner, IterMutInner},
};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::hint::unreachable_unchecked;
use std::mem::ManuallyDrop;
use std::ptr;

pub(crate) mod drain;
pub(crate) mod entry;
#[cfg(feature = "extract_if")]
pub(crate) mod extract_if;
pub(crate) mod iter;

pub(crate) enum MapImpl<K, V, const N: usize> {
    Heapless(heapless::Vec<(K, V), N>),
    Spilled(HashMap<K, V>),
}

impl<K, V, const N: usize> MapImpl<K, V, N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self::Heapless(heapless::Vec::new())
    }

    #[inline(always)]
    pub const fn spilled(&self) -> bool {
        matches!(self, Self::Spilled(_))
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        match self {
            Self::Heapless(_) => N,
            Self::Spilled(m) => m.capacity(),
        }
    }

    #[inline]
    pub fn iter(&self) -> IterInner<'_, K, V, N> {
        match self {
            Self::Heapless(vec) => IterInner::Heapless { next: 0, vec },
            Self::Spilled(map) => IterInner::Spilled(map.iter()),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMutInner<'_, K, V, N> {
        match self {
            Self::Heapless(vec) => IterMutInner::Heapless(vec.iter_mut()),
            Self::Spilled(map) => IterMutInner::Spilled(map.iter_mut()),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Heapless(vec) => vec.len(),
            Self::Spilled(map) => map.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Heapless(vec) => vec.is_empty(),
            Self::Spilled(m) => m.is_empty(),
        }
    }

    #[inline]
    pub fn drain(&mut self) -> DrainInner<'_, K, V, N> {
        match self {
            Self::Heapless(base) => DrainInner::Heapless(HeaplessDrain { base }),
            Self::Spilled(map) => DrainInner::Spilled(map.drain()),
        }
    }

    #[cfg(feature = "extract_if")]
    #[inline]
    pub fn extract_if<F>(&mut self, pred: F) -> extract_if::ExtractIfInner<'_, K, V, F, N>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        match self {
            Self::Heapless(base) => extract_if::ExtractIfInner::Heapless {
                base,
                next: 0,
                pred,
            },
            Self::Spilled(map) => extract_if::ExtractIfInner::Spilled(map.extract_if(pred)),
        }
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        match self {
            Self::Heapless(vec) => {
                vec.retain_mut(|(k, v)| f(k, v));
            }
            Self::Spilled(map) => {
                map.retain(f);
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        match self {
            Self::Heapless(vec) => vec.clear(),
            Self::Spilled(m) => m.clear(),
        }
    }

    /// # Safety
    ///
    /// `MapImpl` must be in the `Heapless` variant.
    #[inline]
    unsafe fn into_heapless_unchecked(self) -> heapless::Vec<(K, V), N> {
        match self {
            Self::Heapless(v) => v,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// # Safety
    ///
    /// `MapImpl` must be in the `Spilled` variant.
    #[inline]
    unsafe fn into_spilled_unchecked(self) -> HashMap<K, V> {
        match self {
            Self::Spilled(m) => m,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// # Safety
    ///
    /// `MapImpl` must be in the `Heapless` variant.
    #[inline]
    unsafe fn as_heapless_unchecked(&self) -> &heapless::Vec<(K, V), N> {
        match self {
            Self::Heapless(m) => m,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    /// # Safety
    ///
    /// `MapImpl` must be in the `Heapless` variant.
    #[inline]
    unsafe fn as_heapless_mut_unchecked(&mut self) -> &mut heapless::Vec<(K, V), N> {
        match self {
            Self::Heapless(m) => m,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    // /// # Safety
    // ///
    // /// `MapImpl` must be in the `Spilled` variant.
    // #[inline]
    // unsafe fn as_spilled_unchecked(&self) -> &HashMap<K, V> {
    //     match self {
    //         Self::Spilled(m) => m,
    //         _ => unsafe { unreachable_unchecked() },
    //     }
    // }

    /// # Safety
    ///
    /// `MapImpl` must be in the `Spilled` variant.
    #[inline]
    unsafe fn as_spilled_mut_unchecked(&mut self) -> &mut HashMap<K, V> {
        match self {
            Self::Spilled(m) => m,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl<K, V, const N: usize> MapImpl<K, V, N>
where
    K: Eq + Hash,
{
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if !self.spilled() && self.len() + additional > N {
            // Safety: we just checked the variant
            unsafe { self.try_spill(additional) }.unwrap();
        } else {
            // Safety: we just checked the variant
            unsafe {
                self.as_spilled_mut_unchecked().reserve(additional);
            }
        }
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        if !self.spilled() {
            if self.len() + additional > N {
                // Safety: we just checked the variant
                unsafe { self.try_spill(additional) }?;
            }
            // otherwise, we're good
        } else {
            // Safety: we just checked the variant
            unsafe {
                self.as_spilled_mut_unchecked().try_reserve(additional)?;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn spill(&mut self) {
        if !self.spilled() {
            // Safety: we just checked the variant
            unsafe { self.try_spill(0) }.unwrap();
        }
    }

    pub fn shrink_into_heapless<const M: usize>(
        self,
    ) -> Result<MapImpl<K, V, M>, MapImpl<K, V, N>> {
        if self.len() > M {
            return Err(self);
        }

        let heapless = match self {
            MapImpl::Heapless(vec) => {
                let vec = ManuallyDrop::new(vec);
                let mut new = heapless::Vec::new();
                unsafe {
                    // Safety: vec won't be dropped, vec cannot overlap with new
                    ptr::copy_nonoverlapping(vec.as_ptr(), new.as_mut_ptr(), vec.len());
                    new.set_len(vec.len());
                }
                new
            }
            MapImpl::Spilled(map) => map.into_iter().collect::<heapless::Vec<(K, V), M>>(),
        };

        Ok(MapImpl::Heapless(heapless))
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        if let Self::Spilled(map) = self {
            map.shrink_to_fit()
        }
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        if let Self::Spilled(map) = self {
            map.shrink_to(min_capacity)
        }
    }

    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, N> {
        match self {
            Self::Heapless(vec) => {
                if vec.is_empty() {
                    Entry::Vacant(VacantEntry::Heapless(HeaplessEntry {
                        key: Some(key),
                        inner: self,
                        index: 0,
                    }))
                } else if let Some(index) = vec.iter().position(|(k, _)| k == &key) {
                    Entry::Occupied(OccupiedEntry::Heapless(HeaplessEntry {
                        key: Some(key),
                        inner: self,
                        index,
                    }))
                } else {
                    let len = vec.len();
                    Entry::Vacant(VacantEntry::Heapless(HeaplessEntry {
                        key: Some(key),
                        inner: self,
                        index: len,
                    }))
                }
            }
            Self::Spilled(map) => match map.entry(key) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    Entry::Occupied(OccupiedEntry::Spilled(entry))
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    Entry::Vacant(VacantEntry::Spilled(entry))
                }
            },
        }
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.get_key_value(k) {
            Some((_, value)) => Some(value),
            None => None,
        }
    }

    #[inline]
    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self {
            Self::Heapless(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    match vec.iter().find(|(key, _)| key.borrow() == k) {
                        Some((key, value)) => Some((key, value)),
                        None => None,
                    }
                }
            }
            Self::Spilled(map) => map.get_key_value(k),
        }
    }

    #[cfg(feature = "many_mut")]
    #[inline]
    pub fn get_many_mut<Q, const M: usize>(&mut self, ks: [&Q; M]) -> Option<[&'_ mut V; M]>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self {
            Self::Heapless(vec) => {
                let is =
                    ks.map(|k| {
                        vec.iter().enumerate().find_map(|(i, (key, _))| {
                            if key.borrow() == k {
                                Some(i)
                            } else {
                                None
                            }
                        })
                    });
                if is.iter().any(|i| i.is_none()) {
                    return None;
                }
                let is = is.map(|i| unsafe { i.unwrap_unchecked() });
                Some(vec.get_many_mut(is).ok()?.map(|(_, v)| v))
            }
            Self::Spilled(map) => map.get_many_mut(ks),
        }
    }

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
        match self {
            Self::Heapless(vec) => {
                let is =
                    ks.map(|k| {
                        vec.iter().enumerate().find_map(|(i, (key, _))| {
                            if key.borrow() == k {
                                Some(i)
                            } else {
                                None
                            }
                        })
                    });
                if is.iter().any(|i| i.is_none()) {
                    return None;
                }
                let is = is.map(|i| unsafe { i.unwrap_unchecked() });
                let es = unsafe { vec.get_many_unchecked_mut(is) };
                Some(es.map(|(_, v)| v))
            }
            Self::Spilled(map) => unsafe { map.get_many_unchecked_mut(ks) },
        }
    }

    #[inline]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(k).is_some()
    }

    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self {
            Self::Heapless(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    match vec.iter_mut().find(|(key, _)| key.borrow() == k) {
                        Some((_, value)) => Some(value),
                        None => None,
                    }
                }
            }
            Self::Spilled(map) => map.get_mut(k),
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self {
            Self::Heapless(vec) => {
                // Scan for equivalent key
                for (key, value) in vec.iter_mut() {
                    if key == &k {
                        return Some(std::mem::replace(value, v));
                    }
                }
                // No equivalent key found, insert new entry
                // find first None slot (previous removal)
                match vec.push((k, v)) {
                    Ok(()) => None,
                    Err((k, v)) => {
                        // No None slot found, spill to HashMap
                        // Safety: we just checked the variant
                        let map = unsafe { self.try_spill(1) };
                        map.unwrap().insert(k, v);
                        None
                    }
                }
            }
            Self::Spilled(m) => m.insert(k, v),
        }
    }

    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.remove_entry(k) {
            Some((_, value)) => Some(value),
            None => None,
        }
    }

    #[inline]
    pub fn remove_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self {
            Self::Heapless(vec) => {
                // find index
                let index = vec.iter().position(|(key, _)| key.borrow() == k)?;
                // Safety: index is in bounds
                Some(unsafe { vec.swap_remove_unchecked(index) })
            }
            Self::Spilled(m) => m.remove_entry(k),
        }
    }

    #[inline]
    pub fn into_hashmap(mut self) -> HashMap<K, V> {
        if !self.spilled() {
            // Safety: we just checked the variant
            unsafe { self.try_spill(0) }.unwrap();
        }
        // Safety: we just spilled the map
        unsafe { self.into_spilled_unchecked() }
    }

    /// # Safety
    ///
    /// Must be in the `Heapless` variant.
    #[inline]
    unsafe fn try_spill(
        &mut self,
        additional: usize,
    ) -> Result<&mut HashMap<K, V>, TryReserveError> {
        let cap_needed = N
            .checked_add(additional)
            .ok_or(TryReserveError { kind: () })?;
        let mut map = HashMap::new();
        map.try_reserve(cap_needed)?;
        let vec = std::mem::replace(self, Self::Spilled(map));
        let (vec, map) = unsafe {
            // Safety: we just swapped the variant
            (
                vec.into_heapless_unchecked(),
                self.as_spilled_mut_unchecked(),
            )
        };
        map.extend(vec);
        Ok(map)
    }

    pub fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        if let MapImpl::Spilled(map) = self {
            map.extend(iter);
            return;
        }
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V, const N: usize> IntoIterator for MapImpl<K, V, N> {
    type Item = (K, V);
    type IntoIter = IntoIterInner<K, V, N>;

    #[inline]
    fn into_iter(self) -> IntoIterInner<K, V, N> {
        match self {
            MapImpl::Heapless(vec) => IntoIterInner::Heapless(vec),
            MapImpl::Spilled(map) => IntoIterInner::Spilled(map.into_iter()),
        }
    }
}

impl<K, V, const N: usize, const M: usize> From<[(K, V); N]> for MapImpl<K, V, M>
where
    K: Eq + Hash,
{
    fn from(arr: [(K, V); N]) -> Self {
        if N <= M {
            Self::Heapless(heapless::Vec::from_iter(arr))
        } else {
            Self::Spilled(HashMap::from(arr))
        }
    }
}

/// The error type for `try_reserve` methods.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TryReserveError {
    kind: (),
}

impl From<std::collections::TryReserveError> for TryReserveError {
    fn from(_: std::collections::TryReserveError) -> Self {
        Self { kind: () }
    }
}

impl Display for TryReserveError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        fmt.write_str("memory allocation failed")
    }
}

impl std::error::Error for TryReserveError {}
