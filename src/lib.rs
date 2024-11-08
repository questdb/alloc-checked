#![cfg_attr(not(test), cfg_attr(feature = "no_std", no_std))]
#![feature(allocator_api)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

extern crate alloc;
extern crate core;

pub mod claim;
pub mod try_clone;
pub mod vec;

#[cfg(feature = "hash_collections")]
pub mod hash;

#[cfg(test)]
pub(crate) mod global_alloc_test_guard;


#[cfg(test)]
#[global_allocator]
static GLOBAL: global_alloc_test_guard::GlobalAllocTestGuardAllocator = global_alloc_test_guard::GlobalAllocTestGuardAllocator;