#![cfg_attr(not(test), cfg_attr(feature = "no_std", no_std))]
#![feature(allocator_api)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

extern crate alloc;
extern crate core;

pub mod claim;
pub mod try_clone;
pub mod vec;
pub mod vec_deque;

#[cfg(feature = "hash_collections")]
pub mod hash;

#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
#[global_allocator]
static GLOBAL: testing::GlobalAllocTestGuardAllocator = testing::GlobalAllocTestGuardAllocator;
