use std::collections::hash_map;
use std::iter::FusedIterator;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) enum ExtractIfInner<'a, K, V, F, const N: usize>
where
    F: FnMut(&K, &mut V) -> bool,
{
    Heapless {
        base: &'a mut heapless::Vec<(K, V), N>,
        next: usize,
        pred: F,
    },
    Spilled(hash_map::ExtractIf<'a, K, V, F>),
}

impl<K, V, F, const N: usize> Iterator for ExtractIfInner<'_, K, V, F, N>
where
    F: FnMut(&K, &mut V) -> bool,
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        match self {
            Self::Heapless { base, next, pred } => {
                while *next < base.len() {
                    let cond = {
                        let elem = &mut base[*next];
                        pred(&elem.0, &mut elem.1)
                    };
                    if cond {
                        return Some(base.swap_remove(*next));
                    } else {
                        *next += 1;
                    }
                }
                None
            }
            Self::Spilled(extract_if) => extract_if.next(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Heapless { base, next, .. } => (0, Some(base.len() - *next)),
            Self::Spilled(extract_if) => extract_if.size_hint(),
        }
    }
}

impl<K, V, F, const N: usize> FusedIterator for ExtractIfInner<'_, K, V, F, N> where
    F: FnMut(&K, &mut V) -> bool
{
}
