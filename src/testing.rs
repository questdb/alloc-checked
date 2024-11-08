use crate::claim::Claim;
use alloc::sync::Arc;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::alloc::{AllocError, Allocator, Global, GlobalAlloc, Layout, System};

thread_local! {
    static GLOBAL_ALLOC_ALLOWED: std::cell::RefCell<bool> = std::cell::RefCell::new(true);
}

struct NoPubCtor;

/// A guard that temporarily error if a test performs global allocation in the current thread.
pub struct NoGlobalAllocGuard(NoPubCtor);

impl NoGlobalAllocGuard {
    pub fn new() -> Self {
        GLOBAL_ALLOC_ALLOWED.with(|alloc_allowed| {
            let mut alloc_allowed = alloc_allowed.borrow_mut();
            if !*alloc_allowed {
                panic!("NoGlobalAllocGuard is not re-entrant.");
            }
            *alloc_allowed = false; // Disable global allocation
        });

        Self(NoPubCtor)
    }
}

impl Drop for NoGlobalAllocGuard {
    fn drop(&mut self) {
        GLOBAL_ALLOC_ALLOWED.with(|alloc_allowed| {
            let mut alloc_allowed = alloc_allowed.borrow_mut();
            *alloc_allowed = true;
        });
    }
}

pub struct AllowGlobalAllocGuard {
    was_allowed: bool,
}

impl AllowGlobalAllocGuard {
    pub fn new() -> Self {
        let was_allowed = GLOBAL_ALLOC_ALLOWED.with(|alloc_allowed| {
            let was_allowed = *alloc_allowed.borrow();
            if !was_allowed {
                let mut alloc_allowed = alloc_allowed.borrow_mut();
                *alloc_allowed = true;
            }
            was_allowed
        });

        Self { was_allowed }
    }
}

impl Drop for AllowGlobalAllocGuard {
    fn drop(&mut self) {
        GLOBAL_ALLOC_ALLOWED.with(|alloc_allowed| {
            let mut alloc_allowed = alloc_allowed.borrow_mut();
            *alloc_allowed = self.was_allowed;
        });
    }
}

/// Enables the `NoGlobalAllocGuard` by acting as a global allocator.
pub struct GlobalAllocTestGuardAllocator;

impl GlobalAllocTestGuardAllocator {
    fn is_allowed(&self) -> bool {
        GLOBAL_ALLOC_ALLOWED.with(|alloc_allowed| {
            *alloc_allowed.borrow() // Check if allocation is allowed for the current thread
        })
    }

    fn guard(&self) {
        if !self.is_allowed() {
            panic!("Caught unexpected global allocation with the NoGlobalAllocGuard. Run tests under debugger.");
        }
    }
}

unsafe impl GlobalAlloc for GlobalAllocTestGuardAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.guard();
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.guard();
        System.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.guard();
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.guard();
        System.realloc(ptr, layout, new_size)
    }
}

#[derive(Clone)]
pub struct WatermarkAllocator {
    watermark: usize,
    in_use: Option<Arc<AtomicUsize>>,
}

impl Drop for WatermarkAllocator {
    fn drop(&mut self) {
        let in_use = self.in_use.take().unwrap();
        let _g = AllowGlobalAllocGuard::new();
        drop(in_use);
    }
}

impl WatermarkAllocator {
    pub fn new(watermark: usize) -> Self {
        let in_use = Some({
            let _g = AllowGlobalAllocGuard::new();
            AtomicUsize::new(0).into()
        });
        Self { watermark, in_use }
    }

    pub fn in_use(&self) -> usize {
        self.in_use.as_ref().unwrap().load(Ordering::SeqCst)
    }
}

impl Claim for WatermarkAllocator {}

unsafe impl Allocator for WatermarkAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let current_in_use = self.in_use.as_ref().unwrap().load(Ordering::SeqCst);
        let new_in_use = current_in_use + layout.size();
        if new_in_use > self.watermark {
            return Err(AllocError);
        }
        let allocated = {
            let _g = AllowGlobalAllocGuard::new();
            Global.allocate(layout)?
        };
        let true_new_in_use = self
            .in_use
            .as_ref()
            .unwrap()
            .fetch_add(allocated.len(), Ordering::SeqCst);
        unsafe {
            if true_new_in_use > self.watermark {
                let ptr = allocated.as_ptr() as *mut u8;
                let to_dealloc = NonNull::new_unchecked(ptr);
                {
                    let _g = AllowGlobalAllocGuard::new();
                    Global.deallocate(to_dealloc, layout);
                }
                Err(AllocError)
            } else {
                Ok(allocated)
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let _g = AllowGlobalAllocGuard::new();
        Global.deallocate(ptr, layout);
        self.in_use
            .as_ref()
            .unwrap()
            .fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

/// A second watermark allocator. This is just to test cases where we need generic types
/// to interoperate, even when their allocator differs. E.g. `lhs: Vec<T, A1> == rhs: Vec<T, A2>`.
#[derive(Clone)]
pub struct WatermarkAllocator2(WatermarkAllocator);

impl WatermarkAllocator2 {
    pub fn new(watermark: usize) -> Self {
        let inner = WatermarkAllocator::new(watermark);
        Self(inner)
    }

    pub fn in_use(&self) -> usize {
        self.0.in_use()
    }
}

impl Claim for WatermarkAllocator2 {}

unsafe impl Allocator for WatermarkAllocator2 {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.0.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.0.deallocate(ptr, layout)
    }
}
