use alloc::alloc::Allocator;
use alloc::collections::TryReserveError;
use alloc::vec::Vec as InnerVec;

pub struct Vec<T, A: Allocator> {
    inner: InnerVec<T, A>,
}

impl<T, A: Allocator> Vec<T, A> {
    pub fn new_in(alloc: A) -> Self {
        Self {
            inner: InnerVec::new_in(alloc),
        }
    }

    pub fn reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, TryReserveError> {
        let mut vec = Self::new_in(alloc);
        vec.reserve(capacity)?;
        Ok(vec)
    }

    pub fn try_push(&mut self, value: T) -> Result<(), TryReserveError> {
        self.inner.try_reserve(1)?;
        // SAFETY: we just reserved space for one more element.
        unsafe {
            let len = self.inner.len();
            let end = self.inner.as_mut_ptr().add(len);
            core::ptr::write(end, value);
            self.inner.set_len(len + 1)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::alloc::Global;
    use alloc::collections::TryReserveError;
    use core::alloc::{AllocError, Layout};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicUsize, Ordering};

    struct WatermarkAllocator {
        watermark: usize,
        in_use: AtomicUsize,
    }

    impl WatermarkAllocator {
        fn new(watermark: usize) -> Self {
            Self {
                watermark,
                in_use: AtomicUsize::new(0),
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
    fn test_vec() {
        let wma = WatermarkAllocator::new(32);
        let mut vec = Vec::new_in(wma);
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        vec.try_push(4).unwrap();
        let _err: TryReserveError = vec.try_push(5).unwrap_err();
        assert_eq!(vec.inner.as_slice(), &[1, 2, 3, 4]);
    }
}
