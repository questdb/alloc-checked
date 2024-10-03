# Overview

The `alloc-checked` crate provides wrapper types for common collections that require an explicit allocator and
return a `Result` instead of panicking on allocation failure.

The wrapper collection types provide two main benefits:
* They can't be used incorrectly by virtue of lacking APIs which panic.
* Provide additional convenience methods for working with the collection in a checked manner.

## Restrictions

The crate requires a recent build of the Rust "nightly" compiler, as it uses the `allocator_api` feature.

## No Std

By default, the crate compiles against the Rust standard library.

The crate is also `#![no_std]` compatible via the `no_std` feature.
When compiled in `no_std` mode, it still relies on the `alloc`, `core` crates.
