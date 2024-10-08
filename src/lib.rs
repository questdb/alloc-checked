#![cfg_attr(feature = "no_std", no_std)]
#![feature(allocator_api)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

extern crate alloc;
extern crate core;

pub mod try_clone;
pub mod vec;
