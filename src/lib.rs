#![cfg_attr(feature = "no_std", no_std)]
#![feature(allocator_api)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

extern crate alloc;
extern crate core;

pub mod claim;
pub mod try_clone;
pub mod vec;
pub mod vec_deque;

#[cfg(test)]
pub(crate) mod test_util {
    use crate::claim::Claim;
    use alloc::alloc::Global;
    use alloc::sync::Arc;
    use core::alloc::{AllocError, Allocator, Layout};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone)]
    pub struct WatermarkAllocator {
        watermark: usize,
        in_use: Arc<AtomicUsize>,
    }

    impl Claim for WatermarkAllocator {}

    impl WatermarkAllocator {
        pub(crate) fn in_use(&self) -> usize {
            self.in_use.load(Ordering::SeqCst)
        }
    }

    impl WatermarkAllocator {
        pub fn new(watermark: usize) -> Self {
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
            let true_new_in_use = self.in_use.fetch_add(allocated.len(), Ordering::SeqCst);
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
}
