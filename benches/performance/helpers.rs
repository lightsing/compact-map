use compact_map::CompactMap;
use rand::distributions::{Distribution, Standard};
use rand::prelude::IteratorRandom;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub trait Map<K, V>
where
    K: Eq + Hash,
{
    fn len(&self) -> usize;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

impl<K: Eq + Hash, V> Map<K, V> for HashMap<K, V> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
    #[inline(always)]
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
    #[inline(always)]
    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }
}

impl<K: Eq + Hash, V, const N: usize> Map<K, V> for CompactMap<K, V, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
    #[inline(always)]
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }
    #[inline(always)]
    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }
}

#[derive(Clone)]
pub struct RandomTest<R, M, K, V> {
    rng: R,
    pub map: M,
    max_entries: usize,
    keys: HashSet<K>,
    _marker: std::marker::PhantomData<V>,
}

impl<R, K, V, M> RandomTest<R, M, K, V>
where
    R: rand::Rng,
    K: Eq + Hash + Clone,
    M: Map<K, V>,
    Standard: Distribution<K>,
    Standard: Distribution<V>,
{
    pub fn new(rng: R, map: M, max_entries: usize) -> Self {
        Self {
            rng,
            map,
            max_entries,
            keys: HashSet::with_capacity(max_entries),
            _marker: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn random_step(&mut self) {
        let remaining = self.max_entries - self.map.len();
        let insert = self
            .rng
            .gen_ratio(remaining as u32, self.max_entries as u32);
        if insert {
            let key: K = self.rng.gen();
            let value: V = self.rng.gen();
            self.keys.insert(key.clone());
            self.map.insert(key, value);
        } else {
            let key = self.keys.iter().choose(&mut self.rng).cloned().unwrap();
            self.keys.remove(&key);
            self.map.remove(&key);
        }
    }
}
