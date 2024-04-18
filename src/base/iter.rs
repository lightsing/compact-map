use std::collections::hash_map;
use std::fmt::Debug;
use std::iter::FusedIterator;
use std::{fmt, slice};

pub(crate) enum IterInner<'a, K, V, const N: usize> {
    Heapless {
        next: usize,
        vec: &'a heapless::Vec<(K, V), N>,
    },
    Spilled(hash_map::Iter<'a, K, V>),
}

impl<K, V, const N: usize> Clone for IterInner<'_, K, V, N> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::Heapless { next, vec } => Self::Heapless {
                next: *next,
                vec: *vec,
            },
            Self::Spilled(iter) => Self::Spilled(iter.clone()),
        }
    }
}
impl<K: Debug, V: Debug, const N: usize> Debug for IterInner<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V, const N: usize> Iterator for IterInner<'a, K, V, N> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        match self {
            Self::Heapless { next, vec } => {
                if *next < vec.len() {
                    let (k, v) = unsafe { vec.get_unchecked(*next) };
                    *next += 1;
                    Some((k, v))
                } else {
                    None
                }
            }
            Self::Spilled(iter) => iter.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Heapless { .. } => (self.len(), Some(self.len())),
            Self::Spilled(iter) => iter.size_hint(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Heapless { .. } => self.len(),
            Self::Spilled(iter) => iter.count(),
        }
    }
    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Heapless { next, vec } => {
                let mut acc = init;
                for i in next..vec.len() {
                    let (k, v) = unsafe { vec.get_unchecked(i) };
                    acc = f(acc, (k, v));
                }
                acc
            }
            Self::Spilled(iter) => iter.fold(init, f),
        }
    }
}

impl<'a, K, V, const N: usize> ExactSizeIterator for IterInner<'a, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Heapless { next, vec } => {
                if *next < vec.len() {
                    vec.len() - *next
                } else {
                    0
                }
            }
            Self::Spilled(iter) => iter.len(),
        }
    }
}

impl<'a, K, V, const N: usize> FusedIterator for IterInner<'a, K, V, N> {}

pub(crate) enum IterMutInner<'a, K, V, const N: usize> {
    Heapless(slice::IterMut<'a, (K, V)>),
    Spilled(hash_map::IterMut<'a, K, V>),
}

impl<'a, K, V, const N: usize> Iterator for IterMutInner<'a, K, V, N> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        match self {
            Self::Heapless(iter) => match iter.next() {
                Some((k, v)) => Some((k, v)),
                None => None,
            },
            Self::Spilled(iter) => iter.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Heapless { .. } => (self.len(), Some(self.len())),
            Self::Spilled(iter) => iter.size_hint(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Heapless { .. } => self.len(),
            Self::Spilled(iter) => iter.count(),
        }
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Heapless(iter) => iter.map(|(k, v)| (&*k, v)).fold(init, f),
            Self::Spilled(iter) => iter.fold(init, f),
        }
    }
}
impl<K, V, const N: usize> ExactSizeIterator for IterMutInner<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Heapless(iter) => iter.len(),
            Self::Spilled(iter) => iter.len(),
        }
    }
}
impl<K, V, const N: usize> FusedIterator for IterMutInner<'_, K, V, N> {}

pub(crate) enum IntoIterInner<K, V, const N: usize> {
    Heapless(heapless::Vec<(K, V), N>),
    Spilled(hash_map::IntoIter<K, V>),
}

impl<K: Debug, V: Debug, const N: usize> Debug for IntoIterInner<K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heapless(vec) => f.debug_list().entries(vec.iter()).finish(),
            Self::Spilled(iter) => iter.fmt(f),
        }
    }
}

impl<K, V, const N: usize> Iterator for IntoIterInner<K, V, N> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        match self {
            Self::Heapless(iter) => iter.pop(),
            Self::Spilled(iter) => iter.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Heapless(vec) => (vec.len(), Some(vec.len())),
            Self::Spilled(iter) => iter.size_hint(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Heapless(vec) => vec.len(),
            Self::Spilled(iter) => iter.count(),
        }
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Heapless(vec) => vec.into_iter().fold(init, f),
            Self::Spilled(iter) => iter.fold(init, f),
        }
    }
}
impl<K, V, const N: usize> ExactSizeIterator for IntoIterInner<K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Heapless(vec) => vec.len(),
            Self::Spilled(iter) => iter.len(),
        }
    }
}
impl<K, V, const N: usize> FusedIterator for IntoIterInner<K, V, N> {}
