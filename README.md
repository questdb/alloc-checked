# Overview

The `alloc-checked` crate provides wrapper types for common collections that require an explicit allocator and
return a `Result` instead of panicking on allocation failure.

The wrapper collection types provide two main benefits:
* They can't be used incorrectly by virtue of lacking APIs which panic.
* Provide additional convenience methods for working with the collection in a checked manner.

*N.B.:* You might find the [`heapless`](https://docs.rs/heapless/latest/heapless/) crate useful too!

## Restrictions

The crate requires a recent build of the Rust "nightly" compiler, as it uses the
[`allocator_api`](https://doc.rust-lang.org/std/alloc/trait.Allocator.html) feature.

## No Std

By default, the crate compiles against the Rust standard library.

The crate is also `#![no_std]` compatible via the `no_std` feature.
When compiled in `no_std` mode, it still relies on the `alloc`, `core` crates.

## Usage

Add the dependency

```bash
cargo add alloc-checked # --features no_std
```

Chances are you might have been using `Vec` in a few places. Import the type you need

```rust
use alloc_checked::vec::Vec;

let vec = Vec::new_in(your_allocator);
vec.push(42)?;  // -> Result<(), TryReserveError>
```

and fix any resulting compile errors.

Along the way, you probably want to implement the `From` trait for your error type to make it easier to bubble up
allocation errors.

```rust
use alloc::AllocError;
use alloc::collections::TryReserveError;

impl From<AllocError> for YourErrorType { /* ... */ }
impl From<TryReserveError> for YourErrorType { /* ... */ }

```

## Current state of the project and design philosophy

We (QuestDB) are adding to this crate on a per-need basis. There's currently decent support for the `Vec` type,
with more types to come. Contributions are quite welcome even if they extend beyond our needs.

The core design philosophy here is that it should never be possible to use this crate in a way that it silently
allocates memory. Having a different type that can't be misused also allows for improved API ergonomics. 

As a small example, std's `Vec` has both `fn with_capacity_in(alloc: Allocator) -> Vec` and
`fn try_with_capacity_in(alloc: Allocator) -> Result<Vec, TryReserveError>` variants.

The `Vec` implementation in `alloc-checked` has a single `fn with_capacity_in(alloc: Allocator) -> Vec`,
likewise many common methods that silently panic in the standard library, here return a `Result` instead with the idea
of an easier code migration.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
