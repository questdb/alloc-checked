use crate::claim::Claim;
use crate::try_clone::TryClone;
use alloc::alloc::Allocator;
use alloc::collections::TryReserveError;
use alloc::vec::Vec as InnerVec;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice::SliceIndex;

pub struct Vec<T, A: Allocator> {
    inner: InnerVec<T, A>,
}

impl<T, A: Allocator> Vec<T, A> {
    #[inline]
    pub fn new_in(alloc: A) -> Self {
        Self {
            inner: InnerVec::new_in(alloc),
        }
    }

    pub fn allocator(&self) -> &A {
        self.inner.allocator()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    #[inline]
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, TryReserveError> {
        Ok(Self {
            inner: InnerVec::try_with_capacity_in(capacity, alloc)?,
        })
    }

    #[inline]
    pub fn push(&mut self, value: T) -> Result<(), TryReserveError> {
        self.reserve(1)?;
        // SAFETY: we just reserved space for one more element.
        unsafe {
            self.unsafe_push(value);
        }
        Ok(())
    }

    #[inline]
    unsafe fn unsafe_push(&mut self, value: T) {
        let len = self.inner.len();
        let end = self.inner.as_mut_ptr().add(len);
        core::ptr::write(end, value);
        self.inner.set_len(len + 1)
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) -> Result<(), TryReserveError> {
        let mut iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();

        // Extend N with pre-allocation from the iterator
        self.reserve(lower_bound)?;
        for _ in 0..lower_bound {
            let Some(value) = iter.next() else {
                return Ok(());
            };
            unsafe {
                self.unsafe_push(value);
            }
        }

        // Dynamically append the rest
        for value in iter {
            self.push(value)?;
        }
        Ok(())
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.inner.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.inner.iter_mut()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.inner.truncate(new_len);
    }

    #[inline]
    pub fn resize_with<F: FnMut() -> T>(
        &mut self,
        new_len: usize,
        mut f: F,
    ) -> Result<(), TryReserveError> {
        let len = self.len();
        if new_len > len {
            self.reserve(new_len - len)?;
            for index in len..new_len {
                unsafe {
                    let end = self.inner.as_mut_ptr().add(index);
                    core::ptr::write(end, f());
                }
            }
            unsafe { self.inner.set_len(new_len) }
        } else {
            self.truncate(new_len);
        }
        Ok(())
    }
}

impl<T: Claim, A: Allocator> Vec<T, A> {
    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[T]) -> Result<(), TryReserveError> {
        self.reserve(slice.len())?;

        // Yes, we re-evaluate the capacity by delegating to the inner Vec,
        // but we also gain the optimizations available via specific implementations
        // for anything that supports the `Copy` trait.
        self.inner.extend_from_slice(slice);
        Ok(())
    }

    #[inline]
    pub fn extend_with(&mut self, additional: usize, value: T) -> Result<(), TryReserveError> {
        self.reserve(additional)?;
        let len = self.inner.len();
        let new_len = len + additional;
        for index in len..new_len {
            unsafe {
                let end = self.inner.as_mut_ptr().add(index);
                core::ptr::write(end, value.clone());
            }
        }
        unsafe { self.inner.set_len(new_len) }
        Ok(())
    }

    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T) -> Result<(), TryReserveError> {
        let len = self.len();
        if new_len > len {
            self.extend_with(new_len - len, value)?;
        } else {
            self.truncate(new_len);
        }
        Ok(())
    }
}

impl<T: Claim, A: Allocator + Claim> TryClone for Vec<T, A> {
    type Error = TryReserveError;

    fn try_clone(&self) -> Result<Self, Self::Error> {
        let mut cloned = Self::with_capacity_in(self.len(), self.allocator().clone())?;
        cloned.extend_from_slice(self.inner.as_slice())?;
        Ok(cloned)
    }
}

impl<T, I: SliceIndex<[T]>, A: Allocator> Index<I> for Vec<T, A> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.inner.index(index)
    }
}

impl<T, I: SliceIndex<[T]>, A: Allocator> IndexMut<I> for Vec<T, A> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.inner.index_mut(index)
    }
}

impl<T, A: Allocator> Deref for Vec<T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, A: Allocator> DerefMut for Vec<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Debug, A: Allocator> Debug for Vec<T, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

macro_rules! __impl_slice_eq1 {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs
        where
            T: PartialEq<U>,
            $($ty: $bound)?
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }

            #[allow(clippy::partialeq_ne_impl)]
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    }
}

__impl_slice_eq1! { [A1: Allocator, A2: Allocator] Vec<T, A1>, Vec<U, A2> }
__impl_slice_eq1! { [A: Allocator] Vec<T, A>, &[U] }
__impl_slice_eq1! { [A: Allocator] Vec<T, A>, &mut [U] }
__impl_slice_eq1! { [A: Allocator] &[T], Vec<U, A> }
__impl_slice_eq1! { [A: Allocator] &mut [T], Vec<U, A> }
__impl_slice_eq1! { [A: Allocator] Vec<T, A>, [U] }
__impl_slice_eq1! { [A: Allocator] [T], Vec<U, A> }
__impl_slice_eq1! { [A: Allocator, const N: usize] Vec<T, A>, [U; N] }
__impl_slice_eq1! { [A: Allocator, const N: usize] [T; N], Vec<U, A> }
__impl_slice_eq1! { [A: Allocator, const N: usize] Vec<T, A>, &[U; N] }
__impl_slice_eq1! { [A: Allocator, const N: usize] &[T; N], Vec<U, A> }

impl<T, A: Allocator> AsRef<Vec<T, A>> for Vec<T, A> {
    fn as_ref(&self) -> &Vec<T, A> {
        self
    }
}

impl<T, A: Allocator> AsMut<Vec<T, A>> for Vec<T, A> {
    fn as_mut(&mut self) -> &mut Vec<T, A> {
        self
    }
}

impl<T, A: Allocator> AsRef<[T]> for Vec<T, A> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, A: Allocator> AsMut<[T]> for Vec<T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Claim;
    use crate::global_alloc_test_guard::{AllowNextGlobalAllocGuard, NoGlobalAllocGuard};
    use alloc::alloc::Global;
    use alloc::boxed::Box;
    use alloc::collections::TryReserveError;
    use alloc::sync::Arc;
    use alloc::{format, vec};
    use core::alloc::{AllocError, Layout};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    struct WatermarkAllocator {
        watermark: usize,
        in_use: Option<Arc<AtomicUsize>>,
    }

    impl Drop for WatermarkAllocator {
        fn drop(&mut self) {
            let in_use = self.in_use.take().unwrap();
            let _g = AllowNextGlobalAllocGuard::new();
            drop(in_use);
        }
    }

    impl Claim for WatermarkAllocator {}

    impl WatermarkAllocator {
        pub(crate) fn in_use(&self) -> usize {
            self.in_use.as_ref().unwrap().load(Ordering::SeqCst)
        }
    }

    impl WatermarkAllocator {
        fn new(watermark: usize) -> Self {
            let in_use = Some({
                let _g = AllowNextGlobalAllocGuard::new();
                AtomicUsize::new(0).into()
            });
            Self {
                watermark,
                in_use,
            }
        }
    }

    unsafe impl Allocator for WatermarkAllocator {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            let current_in_use = self.in_use.as_ref().unwrap().load(Ordering::SeqCst);
            let new_in_use = current_in_use + layout.size();
            if new_in_use > self.watermark {
                return Err(AllocError);
            }
            let allocated = {
                let _g = AllowNextGlobalAllocGuard::new();
                Global.allocate(layout)?
            };
            let true_new_in_use = self.in_use.as_ref().unwrap().fetch_add(allocated.len(), Ordering::SeqCst);
            unsafe {
                if true_new_in_use > self.watermark {
                    let ptr = allocated.as_ptr() as *mut u8;
                    let to_dealloc = NonNull::new_unchecked(ptr);
                    {
                        let _g = AllowNextGlobalAllocGuard::new();
                        Global.deallocate(to_dealloc, layout);
                    }
                    Err(AllocError)
                } else {
                    Ok(allocated)
                }
            }
        }

        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            let _g = AllowNextGlobalAllocGuard::new();
            Global.deallocate(ptr, layout);
            self.in_use.as_ref().unwrap().fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }

    #[test]
    fn test_basics() {
        let _g = NoGlobalAllocGuard::new();
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma.clone());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
        assert!(vec.is_empty());
        vec.push(1).unwrap();
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.capacity(), 4);
        assert_eq!(
            wma.in_use(),
            vec.capacity() * size_of::<i32>()
        );
        assert_eq!(
            vec.allocator().in_use(),
            vec.capacity() * size_of::<i32>()
        );
        let _err: TryReserveError = vec.push(5).unwrap_err();
        assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(vec.len(), 4);
        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), 4);
    }

    #[test]
    fn test_with_capacity_in() {
        let wma = WatermarkAllocator::new(32);
        let vec: Vec<usize, _> = Vec::with_capacity_in(4, wma.clone()).unwrap();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.as_slice(), &[]);
        assert_eq!(vec.inner.capacity(), 4);
        assert_eq!(wma.in_use(), 4 * size_of::<usize>());

        let _err: TryReserveError = Vec::<i8, _>::with_capacity_in(5, wma).unwrap_err();
    }

    #[test]
    fn test_reserve() {
        let wma = WatermarkAllocator::new(32);
        let mut vec: Vec<bool, _> = Vec::new_in(wma);
        vec.reserve(32).unwrap();
        assert_eq!(vec.inner.capacity(), 32);

        let _err: TryReserveError = vec.reserve(33).unwrap_err();
    }

    #[test]
    fn test_fmt_debug() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        assert_eq!(format!("{:?}", vec), "[1, 2, 3, 4]");
    }

    #[test]
    fn test_iter() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        let mut iter = vec.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_as_ptr() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma.clone());
        assert_eq!(wma.in_use(), 0);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        let ptr = vec.as_ptr();
        unsafe {
            assert_eq!(*ptr, 1);
            assert_eq!(*ptr.add(1), 2);
            assert_eq!(*ptr.add(2), 3);
            assert_eq!(*ptr.add(3), 4);
        }
    }

    #[test]
    fn test_as_mut_ptr() {
        let wma = WatermarkAllocator::new(64);
        let mut vec = Vec::new_in(wma.clone());
        assert_eq!(wma.in_use(), 0);
        vec.push('a').unwrap();
        vec.push('b').unwrap();
        vec.push('c').unwrap();
        vec.push('d').unwrap();
        vec.push('e').unwrap();
        vec.push('f').unwrap();
        let ptr = vec.as_mut_ptr();
        unsafe {
            assert_eq!(*ptr, 'a');
            assert_eq!(*ptr.add(1), 'b');
            assert_eq!(*ptr.add(2), 'c');
            assert_eq!(*ptr.add(3), 'd');
            assert_eq!(*ptr.add(4), 'e');
            assert_eq!(*ptr.add(5), 'f');
        }
    }

    #[test]
    fn test_index() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[3], 4);
    }

    /// A type that implements `Clone` and `Claim`, but not `Copy`.
    #[derive(Clone, Eq, PartialEq)]
    struct Claimable(i32);

    impl Claim for Claimable {}

    #[test]
    fn test_extend_from_slice_clone() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.extend_from_slice(&[Claimable(1), Claimable(2), Claimable(3), Claimable(4)])
            .unwrap();
    }

    #[test]
    fn test_extend_from_slice_copy() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.extend_from_slice(&[1, 2, 3, 4]).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 2, 3, 4]);

        let _err: TryReserveError = vec.extend_from_slice(&[5, 6]).unwrap_err();

        vec.extend_from_slice(&[]).unwrap();
    }

    #[test]
    fn test_deref() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        assert_eq!(&*vec, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_deref_mut() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        let vec: &mut [i32] = &mut vec;
        vec[0] = 5;
        vec[1] = 6;
        vec[2] = 7;
        vec[3] = 8;
        assert_eq!(&*vec, &[5, 6, 7, 8]);
    }

    struct MyIter {
        counter: usize,
        min_size_hint: usize,
    }

    impl MyIter {
        fn new(min_size_hint: usize) -> Self {
            Self {
                counter: 0,
                min_size_hint,
            }
        }
    }

    impl Iterator for MyIter {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            if self.counter >= 10 {
                return None;
            }
            self.counter += 1;
            Some(self.counter - 1)
        }

        // This sort-of lies, but it's here for testing purposes.
        // It states that the iterator has at least, just, 5 elements.
        // This is done so we can get good code coverage and test both
        // the optimised pre-reserve code path for `extend` and the
        // slower dynamic re-allocation code path.
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.min_size_hint, None)
        }
    }

    #[test]
    fn test_extend() {
        // Test the optimised with mixed pre-reserved and dynamic allocation extend paths.
        let wma = WatermarkAllocator::new(32 * size_of::<usize>());
        {
            let mut vec = Vec::new_in(wma.clone());
            vec.extend(MyIter::new(5)).unwrap();
            assert_eq!(vec.inner.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }
        assert_eq!(wma.in_use(), 0);

        // Test with a fully pre-reserved path.
        {
            let mut vec = Vec::new_in(wma.clone());
            vec.extend(MyIter::new(10)).unwrap();
            assert_eq!(vec.inner.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }
        assert_eq!(wma.in_use(), 0);

        // Test with a fully pre-reserved path, but the `min` size_hint lies
        // and exceeds the truth.
        {
            let mut vec = Vec::new_in(wma.clone());
            vec.extend(MyIter::new(20)).unwrap();
            assert_eq!(vec.inner.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }
        assert_eq!(wma.in_use(), 0);

        // The min size hint is zero, all dynamically allocated.
        {
            let mut vec = Vec::new_in(wma.clone());
            vec.extend(MyIter::new(0)).unwrap();
            assert_eq!(vec.inner.as_slice(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }
        assert_eq!(wma.in_use(), 0);
    }

    #[test]
    fn test_truncate() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        vec.push(4).unwrap();
        vec.truncate(2);
        assert_eq!(vec.inner.as_slice(), &[1, 2]);
        vec.truncate(0);
        let empty: &[i32] = &[];
        assert_eq!(vec.inner.as_slice(), empty);
    }

    #[test]
    fn test_extend_with() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.extend_with(3, 1).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1, 1]);
    }

    #[test]
    fn test_resize() {
        let wma = WatermarkAllocator::new(64);
        let mut vec = Vec::new_in(wma);
        vec.resize(3, 1).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1, 1]);
        vec.resize(5, 2).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1, 1, 2, 2]);
        vec.resize(2, 3).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1]);
    }

    #[test]
    fn test_resize_with() {
        let wma = WatermarkAllocator::new(64);
        let mut vec = Vec::new_in(wma);
        vec.resize_with(3, || 1).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1, 1]);
        vec.resize_with(5, || 2).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1, 1, 2, 2]);
        vec.resize_with(2, || 3).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 1]);
    }

    #[derive(PartialEq, Debug)]
    struct IntWrapper(pub i32);

    impl PartialEq<i32> for IntWrapper {
        fn eq(&self, other: &i32) -> bool {
            self.0 == *other
        }
    }

    impl PartialEq<IntWrapper> for i32 {
        fn eq(&self, other: &IntWrapper) -> bool {
            self == &other.0
        }
    }

    fn w(i: i32) -> IntWrapper {
        IntWrapper(i)
    }

    #[test]
    fn test_eq() {
        let wma = WatermarkAllocator::new(64);

        // __impl_slice_eq1! { [A1: Allocator, A2: Allocator] Vec<T, A1>, Vec<U, A2> }
        {
            let mut lhs = Vec::new_in(wma.clone());
            let mut rhs = Vec::new_in(Global);

            lhs.extend(vec![1, 2, 3]).unwrap();
            rhs.extend(vec![w(1), w(2), w(3)]).unwrap();
            assert_eq!(lhs, rhs);
            assert_eq!(rhs, lhs);

            rhs.push(w(4)).unwrap();
            assert_ne!(lhs, rhs);
            assert_ne!(rhs, lhs);
        }

        // __impl_slice_eq1! { [A: Allocator] Vec<T, A>, &[U] }
        // __impl_slice_eq1! { [A: Allocator] &[T], Vec<U, A> }
        {
            let mut lhs = Vec::new_in(wma.clone());
            lhs.extend(vec![1, 2, 3]).unwrap();
            let rhs: &[IntWrapper] = &[w(1), w(2), w(3)];
            assert_eq!(lhs, rhs);
            assert_eq!(rhs, lhs);

            let rhs2: &[IntWrapper] = &[w(1), w(2), w(3), w(4)];
            assert_ne!(lhs, rhs2);
            assert_ne!(rhs2, lhs);
        }

        // __impl_slice_eq1! { [A: Allocator] Vec<T, A>, &mut [U] }
        // __impl_slice_eq1! { [A: Allocator] &mut [T], Vec<U, A> }
        {
            let mut lhs = Vec::new_in(wma.clone());
            lhs.extend(vec![1, 2, 3]).unwrap();

            let mut rhs_vec = vec![w(1), w(2), w(3)];
            let rhs: &mut [IntWrapper] = &mut rhs_vec;

            assert_eq!(lhs, rhs);
            assert_eq!(rhs, lhs);

            rhs_vec.push(w(4));
            let rhs2: &mut [IntWrapper] = &mut rhs_vec;
            assert_ne!(lhs, rhs2);
            assert_ne!(rhs2, lhs);
        }

        // __impl_slice_eq1! { [A: Allocator] Vec<T, A>, [U] }
        // __impl_slice_eq1! { [A: Allocator] [T], Vec<U, A> }
        {
            let mut lhs = Vec::new_in(wma.clone());
            lhs.extend(vec![1, 2, 3]).unwrap();

            let rhs: Box<[IntWrapper]> = Box::new([w(1), w(2), w(3)]);
            assert_eq!(lhs, *rhs);
            assert_eq!(*rhs, lhs);

            let rhs2: Box<[IntWrapper]> = Box::new([w(1), w(2), w(3), w(4)]);
            assert_ne!(lhs, *rhs2);
            assert_ne!(*rhs2, lhs);
        }

        // __impl_slice_eq1! { [A: Allocator, const N: usize] Vec<T, A>, [U; N] }
        // __impl_slice_eq1! { [A: Allocator, const N: usize] [T; N], Vec<U, A> }
        {
            let mut lhs = Vec::new_in(wma.clone());
            lhs.extend(vec![1, 2, 3]).unwrap();

            let rhs: [IntWrapper; 3] = [w(1), w(2), w(3)];
            assert_eq!(lhs, rhs); // Compare Vec with fixed-size array
            assert_eq!(rhs, lhs); // Compare fixed-size array with Vec

            let rhs2: [IntWrapper; 4] = [w(1), w(2), w(3), w(4)];
            assert_ne!(lhs, rhs2); // Compare Vec with longer array
            assert_ne!(rhs2, lhs); // Compare longer array with Vec
        }

        // __impl_slice_eq1! { [A: Allocator, const N: usize] Vec<T, A>, &[U; N] }
        // __impl_slice_eq1! { [A: Allocator, const N: usize] &[T; N], Vec<U, A> }
        {
            let mut lhs = Vec::new_in(wma.clone());
            lhs.extend(vec![1, 2, 3]).unwrap();

            let rhs_arr: [IntWrapper; 3] = [w(1), w(2), w(3)];
            let rhs: &[IntWrapper; 3] = &rhs_arr;
            assert_eq!(lhs, rhs);
            assert_eq!(rhs, lhs);

            lhs.push(4).unwrap();
            assert_ne!(lhs, rhs);
            assert_ne!(rhs, lhs);
        }
    }

    fn get_first_elem_vec<T: Claim, A: Allocator>(vec: impl AsRef<Vec<T, A>>) -> T {
        let vec = vec.as_ref();
        vec.first().unwrap().clone()
    }

    fn get_first_elem_slice<T: Claim>(slice: impl AsRef<[T]>) -> T {
        let vec = slice.as_ref();
        vec.first().unwrap().clone()
    }

    #[test]
    fn test_as_ref() {
        let wma = WatermarkAllocator::new(128);
        let mut vec1 = Vec::new_in(wma);
        vec1.extend(vec![1, 2, 3]).unwrap();
        let vec2 = vec1.try_clone().unwrap();

        assert_eq!(vec1, vec2);
        let e0vec1 = get_first_elem_vec(vec1);
        let e0vec2 = get_first_elem_slice(vec2);
        assert_eq!(e0vec1, 1);
        assert_eq!(e0vec2, 1);
    }

    fn doubled_first_elem_vec(mut vec: impl AsMut<Vec<i32, WatermarkAllocator>>) -> i32 {
        let vec = vec.as_mut();
        vec[0] *= 2;
        vec[0]
    }

    fn doubled_first_elem_slice(mut vec: impl AsMut<[i32]>) -> i32 {
        let vec = vec.as_mut();
        vec[0] *= 2;
        vec[0]
    }

    #[test]
    fn test_as_mut() {
        let wma = WatermarkAllocator::new(128);
        let mut vec1 = Vec::new_in(wma);
        vec1.extend(vec![1, 2, 3]).unwrap();
        let vec2 = vec1.try_clone().unwrap();
        assert_eq!(vec1, vec2);

        let d0vec1 = doubled_first_elem_vec(vec1);
        let d0vec2 = doubled_first_elem_slice(vec2);

        assert_eq!(d0vec1, 2);
        assert_eq!(d0vec2, 2);
    }

    #[test]
    fn test_try_clone() {
        let wma = WatermarkAllocator::new(64);
        let mut vec1 = Vec::new_in(wma.clone());
        vec1.extend([1usize, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(vec1.len(), 8);
        assert_eq!(vec1.capacity(), 8);
        assert_eq!(wma.in_use(), 64);
        assert!(vec1.try_clone().is_err());
    }
}
