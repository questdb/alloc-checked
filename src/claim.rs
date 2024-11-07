use alloc::alloc::Global;
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
macro_rules! impl_claim_for {
    ($($t:ty),*) => {
        $(
            impl Claim for $t {}
        )*
    };
}

// Generate impls for simple types
impl_claim_for! {
    (), u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize,
    f32, f64, bool, char, Global
}

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