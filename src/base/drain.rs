use std::collections::hash_map;
use std::fmt;
use std::fmt::Debug;
use std::iter::FusedIterator;

pub(crate) enum DrainInner<'a, K, V, const N: usize> {
    Heapless(HeaplessDrain<'a, K, V, N>),
    Spilled(hash_map::Drain<'a, K, V>),
}

impl<K: Debug, V: Debug, const N: usize> Debug for DrainInner<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heapless(drain) => drain.fmt(f),
            Self::Spilled(drain) => drain.fmt(f),
        }
    }
}

impl<'a, K, V, const N: usize> Iterator for DrainInner<'a, K, V, N> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        match self {
            Self::Heapless(drain) => drain.next(),
            Self::Spilled(drain) => drain.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Heapless(drain) => drain.size_hint(),
            Self::Spilled(drain) => drain.size_hint(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Heapless(drain) => drain.len(),
            Self::Spilled(drain) => drain.count(),
        }
    }
    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Heapless(drain) => drain.fold(init, f),
            Self::Spilled(drain) => drain.fold(init, f),
        }
    }
}
impl<K, V, const N: usize> ExactSizeIterator for DrainInner<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Heapless(drain) => drain.len(),
            Self::Spilled(drain) => drain.len(),
        }
    }
}
impl<K, V, const N: usize> FusedIterator for DrainInner<'_, K, V, N> {}

pub(crate) struct HeaplessDrain<'a, K, V, const N: usize> {
    pub(crate) base: &'a mut heapless::Vec<(K, V), N>,
}

impl<K: Debug, V: Debug, const N: usize> Debug for HeaplessDrain<'_, K, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.base.iter()).finish()
    }
}

impl<'a, K, V, const N: usize> Iterator for HeaplessDrain<'a, K, V, N> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.base.pop()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.base.len(), Some(self.base.len()))
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
        let mut acc = init;
        while let Some(x) = self.base.pop() {
            acc = f(acc, x);
        }
        acc
    }
}
impl<K, V, const N: usize> ExactSizeIterator for HeaplessDrain<'_, K, V, N> {
    #[inline]
    fn len(&self) -> usize {
        self.base.len()
    }
}
impl<K, V, const N: usize> FusedIterator for HeaplessDrain<'_, K, V, N> {}

impl<K, V, const N: usize> Drop for HeaplessDrain<'_, K, V, N> {
    fn drop(&mut self) {
        self.base.clear();
    }
}
