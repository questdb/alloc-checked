use alloc::alloc::Allocator;
use alloc::collections::TryReserveError;
use alloc::vec::Vec as InnerVec;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};
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
    pub fn try_push(&mut self, value: T) -> Result<(), TryReserveError> {
        self.reserve(1)?;
        // SAFETY: we just reserved space for one more element.
        unsafe {
            let len = self.inner.len();
            let end = self.inner.as_mut_ptr().add(len);
            core::ptr::write(end, value);
            self.inner.set_len(len + 1)
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
    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
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
    fn test_try_push() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        vec.try_push(1).unwrap();
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
        assert_eq!(vec.len(), 4);
        let _err: TryReserveError = vec.try_push(5).unwrap_err();
        assert_eq!(vec.inner.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(vec.len(), 4);
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
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
        assert_eq!(format!("{:?}", vec), "[1, 2, 3, 4]");
    }

    #[test]
    fn test_iter() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
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
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
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
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
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
        vec.try_push('a').unwrap();
        vec.try_push('b').unwrap();
        vec.try_push('c').unwrap();
        vec.try_push('d').unwrap();
        vec.try_push('e').unwrap();
        vec.try_push('f').unwrap();
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
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[3], 4);
    }
}
