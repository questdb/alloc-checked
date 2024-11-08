use crate::claim::Claim;
use core::alloc::Allocator;
use core::hash::{BuildHasher, Hash};
pub use hashbrown::hash_map::{Keys, Values, ValuesMut, Iter, IterMut};
pub use hashbrown::DefaultHashBuilder;
use hashbrown::{HashMap as InnerHashMap, TryReserveError};

pub struct HashMap<K, V, A: Allocator, S = DefaultHashBuilder> {
    inner: InnerHashMap<K, V, S, A>,
}

impl<K, V, A: Allocator + Claim> HashMap<K, V, A, DefaultHashBuilder> {
    #[inline]
    pub fn new_in(alloc: A) -> Self {
        Self::with_hasher_in(DefaultHashBuilder::default(), alloc)
    }

    #[inline]
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        Self::with_capacity_and_hasher_in(capacity, DefaultHashBuilder::default(), alloc)
    }
}

impl<K, V, A: Allocator + Claim, S> HashMap<K, V, A, S> {
    #[inline]
    pub fn allocator(&self) -> &A {
        self.inner.allocator()
    }

    #[inline]
    pub fn with_hasher_in(hash_builder: S, alloc: A) -> Self {
        Self {
            inner: InnerHashMap::with_hasher_in(hash_builder, alloc),
        }
    }

    #[inline]
    pub fn with_capacity_and_hasher_in(capacity: usize, hash_builder: S, alloc: A) -> Self {
        Self {
            inner: InnerHashMap::with_capacity_and_hasher_in(capacity, hash_builder, alloc),
        }
    }

    #[inline]
    pub fn hasher(&self) -> &S {
        self.inner.hasher()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn clear(&mut self) {
        // TODO(amunra): May this reallocate memory?
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.inner.keys()
    }

    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.inner.values()
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.inner.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.inner.iter_mut()
    }
}

impl<K, V, A, S> HashMap<K, V, A, S>
where
    K: Eq + Hash,
    A: Allocator + Claim,
    S: BuildHasher,
{
    #[inline]
    pub fn reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    // #[inline]
    // pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V> {
    //     // TODO(amunra): May this reallocate memory?
    //     self.inner.remove(k)
    // }
}

