use std::alloc::{GlobalAlloc, Layout, System};

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

pub struct AllowNextGlobalAllocGuard {
    was_allowed: bool,
}

impl AllowNextGlobalAllocGuard {
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

impl Drop for AllowNextGlobalAllocGuard {
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
            panic!("Global allocation disabled by a NoGlobalAllocGuard.");
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
