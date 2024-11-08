use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;
use heapless::Vec;
use std::thread::{self, ThreadId};

static NO_ALLOC_THREADS: Mutex<Vec<ThreadId, 128>> = Mutex::new(Vec::new());

pub struct NoGlobalAllocGuard {}

impl NoGlobalAllocGuard {
    pub fn new() -> Self {
        let tid = thread::current().id();

        let mut vec = NO_ALLOC_THREADS.lock()
            .expect("NO_ALLOC_THREADS lockable");

        if vec.contains(&tid) {
            panic!("NoGlobalAllocGuard is not re-entrant");
        }

        vec.push(tid).expect("NO_ALLOC_THREADS capacity exceeded");

        Self {}
    }
}

impl Drop for NoGlobalAllocGuard {
    fn drop(&mut self) {
        let mut vec = NO_ALLOC_THREADS.lock()
            .expect("NO_ALLOC_THREADS lockable");

        let tid = thread::current().id();
        let idx = vec
            .iter()
            .position(|recorded_tid| tid == *recorded_tid)
            .expect("unmatched thread ID");

        vec.remove(idx);
    }
}

pub struct GlobalAllocTestGuardAllocator;

impl GlobalAllocTestGuardAllocator {
    fn is_allowed(&self) -> bool {
        let tid = thread::current().id();
        let vec = NO_ALLOC_THREADS.lock()
            .expect("NO_ALLOC_THREADS lockable");
        !vec.contains(&tid)
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
        System.dealloc(ptr, layout);
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
