use alloc::alloc::{Allocator, Global};
use alloc::collections::TryReserveError;
use alloc::vec::Vec as InnerVec;

pub struct Vec<T, A: Allocator = Global> {
    inner: InnerVec<T, A>,
}

impl <T, A: Allocator> Vec<T, A> {
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
    use core::alloc::{AllocError, Layout};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use super::*;

    struct WatermarkAllocator {
        watermark: usize,
        in_use: AtomicUsize,
    }

    unsafe impl Allocator for WatermarkAllocator {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            let current_in_use = self.in_use.load(Ordering::SeqCst);
            let new_in_use = current_in_use + layout.size();
            if new_in_use > self.watermark {
                return Err(AllocError);
            }
            let allocated = Global.allocate(layout)?;
            let true_new_in_use = self.in_use
                .fetch_add(layout.size(), Ordering::SeqCst);
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
            todo!()
        }
    }

    #[test]
    fn test_vec() {
        let mut vec = Vec::new_in(Global);
        vec.try_push(1).unwrap();
        vec.try_push(2).unwrap();
        vec.try_push(3).unwrap();
        assert_eq!(vec.inner.as_slice(), &[1, 2, 3]);
    }

}