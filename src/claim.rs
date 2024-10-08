use alloc::rc::Rc;
use alloc::sync::Arc;
use core::convert::Infallible;
use core::marker::PhantomData;

/// A marker trait for infallible cloneable objects.
/// Only implement this for your type if you can guarantee that cloning it
/// is guaranteed not to panic.
///
/// For details on the idea, read the [Claiming, auto and
/// otherwise](https://smallcultfollowing.com/babysteps/blog/2024/06/21/claim-auto-and-otherwise/)
/// blog post.
pub trait Claim: Clone {}

// Anything which is trivially copiable is automatically infallible
// We need to list these out since the compiler will not allow us to `impl <T: Copy> impl Claim {}`
impl Claim for () {}
impl Claim for u8 {}
impl Claim for u16 {}
impl Claim for u32 {}
impl Claim for u64 {}
impl Claim for u128 {}
impl Claim for usize {}
impl Claim for i8 {}
impl Claim for i16 {}
impl Claim for i32 {}
impl Claim for i64 {}
impl Claim for i128 {}
impl Claim for isize {}
impl Claim for f32 {}
impl Claim for f64 {}
impl Claim for bool {}
impl Claim for char {}
impl<T: ?Sized> Claim for *const T {}
impl<T: ?Sized> Claim for *mut T {}
impl<T: Copy, const N: usize> Claim for [T; N] {}
impl<T: ?Sized> Claim for PhantomData<T> {}
impl<T: ?Sized> Claim for &T {}

// A few other common impls, non-exhaustive
impl<T> Claim for Arc<T> {}
impl<T> Claim for Rc<T> {}
impl Claim for Infallible {}
impl<T: Claim> Claim for Option<T> {}
impl<T: Claim, E: Claim> Claim for Result<T, E> {}
