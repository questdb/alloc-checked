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
        let mut vec = Self::new_in(alloc);
        vec.reserve(capacity)?;
        Ok(vec)
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
}

impl<T: Clone, A: Allocator> Vec<T, A> {
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
        for _ in 0..additional {
            unsafe {
                self.unsafe_push(value.clone());
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::alloc::Global;
    use alloc::collections::TryReserveError;
    use alloc::format;
    use alloc::sync::Arc;
    use core::alloc::{AllocError, Layout};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    struct WatermarkAllocator {
        watermark: usize,
        in_use: Arc<AtomicUsize>,
    }

    impl WatermarkAllocator {
        pub(crate) fn in_use(&self) -> usize {
            self.in_use.load(Ordering::SeqCst)
        }
    }

    impl WatermarkAllocator {
        fn new(watermark: usize) -> Self {
            Self {
                watermark,
                in_use: AtomicUsize::new(0).into(),
            }
        }
    }

    unsafe impl Allocator for WatermarkAllocator {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            let current_in_use = self.in_use.load(Ordering::SeqCst);
            let new_in_use = current_in_use + layout.size();
            if new_in_use > self.watermark {
                return Err(AllocError);
            }
            let allocated = Global.allocate(layout)?;
            let true_new_in_use = self.in_use.fetch_add(layout.size(), Ordering::SeqCst);
            unsafe {
                if true_new_in_use > self.watermark {
                    let ptr = allocated.as_ptr() as *mut u8;
                    let to_dealloc = NonNull::new_unchecked(ptr);
                    Global.deallocate(to_dealloc, layout);
                    Err(AllocError)
                } else {
                    Ok(allocated)
                }
            }
        }

        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            Global.deallocate(ptr, layout);
            self.in_use.fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }

    #[test]
    fn test_basics() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
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
        assert_eq!(vec.inner.capacity(), 4);

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

    /// A type that implements `Clone` but not `Copy`.
    #[derive(Clone, Eq, PartialEq)]
    struct Cloneable(i32);

    #[test]
    fn test_extend_from_slice_clone() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.extend_from_slice(&[Cloneable(1), Cloneable(2), Cloneable(3), Cloneable(4)])
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
        assert_eq!(vec.inner.as_slice(), &[]);
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
}
