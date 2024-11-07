use core::alloc::Allocator;
use hashbrown::HashMap as InnerHashMap;
pub use hashbrown::DefaultHashBuilder;
pub use hashbrown::hash_map::{Keys, Values, ValuesMut};
use crate::claim::Claim;

pub struct HashMap<K, V, A: Allocator, S = DefaultHashBuilder> {
    inner: InnerHashMap<K, V, S, A>
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
            inner: InnerHashMap::with_hasher_in(hash_builder, alloc)
        }
    }
    
    #[inline]
    pub fn with_capacity_and_hasher_in(capacity: usize, hash_builder: S, alloc: A) -> Self {
        Self {
            inner: InnerHashMap::with_capacity_and_hasher_in(capacity, hash_builder, alloc)
        }
    }

    #[inline]
    pub fn hasher(&self) -> &S {
        &self.inner.hasher()
    }
    
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
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
}