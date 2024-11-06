use crate::claim::Claim;
use crate::try_clone::TryClone;
use alloc::collections::vec_deque::{Drain, VecDeque as InnerVecDeque};
use alloc::collections::vec_deque::{Iter, IterMut};
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::ops::RangeBounds;

pub struct VecDeque<T, A: Allocator> {
    inner: InnerVecDeque<T, A>,
}

impl<T, A: Allocator> VecDeque<T, A> {
    #[inline]
    pub fn new_in(alloc: A) -> Self {
        Self {
            inner: InnerVecDeque::new_in(alloc),
        }
    }

    #[inline]
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Result<Self, TryReserveError> {
        let backing = Vec::try_with_capacity_in(capacity, alloc)?;
        let inner = backing.into();
        Ok(Self { inner })
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn allocator(&self) -> &A {
        self.inner.allocator()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
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
    pub fn range<R>(&self, range: R) -> Iter<'_, T>
    where
        R: RangeBounds<usize>,
    {
        self.inner.range(range)
    }

    #[inline]
    pub fn range_mut<R>(&mut self, range: R) -> IterMut<'_, T>
    where
        R: RangeBounds<usize>,
    {
        self.inner.range_mut(range)
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T, A>
    where
        R: RangeBounds<usize>,
    {
        self.inner.drain(range)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    #[inline]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.inner.contains(x)
    }

    #[inline]
    pub fn front(&self) -> Option<&T> {
        self.inner.front()
    }

    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.inner.front_mut()
    }

    #[inline]
    pub fn back(&self) -> Option<&T> {
        self.inner.back()
    }

    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.inner.back_mut()
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    #[inline]
    pub fn push_front(&mut self, item: T) -> Result<(), TryReserveError> {
        self.reserve(1)?;
        self.inner.push_front(item);
        Ok(())
    }

    #[inline]
    pub fn push_back(&mut self, item: T) -> Result<(), TryReserveError> {
        self.reserve(1)?;
        self.inner.push_back(item);
        Ok(())
    }

    #[inline]
    pub fn insert(&mut self, index: usize, item: T) -> Result<(), TryReserveError> {
        self.reserve(1)?;
        self.inner.insert(index, item);
        Ok(())
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<T> {
        self.inner.remove(index)
    }

    #[inline]
    pub fn append(&mut self, other: &mut Self) -> Result<(), TryReserveError> {
        self.reserve(other.len())?;
        self.inner.append(&mut other.inner);
        Ok(())
    }

    #[inline]
    pub fn make_contiguous(&mut self) -> &mut [T] {
        self.inner.make_contiguous()
    }
}

impl<T: Claim, A: Allocator + Claim> TryClone for VecDeque<T, A> {
    type Error = TryReserveError;

    fn try_clone(&self) -> Result<Self, Self::Error> {
        let mut cloned = Self::with_capacity_in(self.len(), self.allocator().clone())?;
        cloned.inner.extend(self.iter().cloned());
        Ok(cloned)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::string::String;
    use super::*;
    use crate::test_util::WatermarkAllocator;

    #[test]
    fn test_new_in() {
        let alloc = WatermarkAllocator::new(1024);
        let deque: VecDeque<i32, _> = VecDeque::new_in(alloc.clone());
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
        assert_eq!(alloc.in_use(), 0);
    }

    #[test]
    fn test_with_capacity_in_success() {
        let alloc = WatermarkAllocator::new(128);
        let deque: Result<VecDeque<i32, _>, _> = VecDeque::with_capacity_in(10, alloc.clone());
        assert!(deque.is_ok());
        assert_eq!(alloc.in_use(), deque.unwrap().capacity() * size_of::<i32>());
    }

    #[test]
    fn test_with_capacity_in_failure() {
        let alloc = WatermarkAllocator::new(4); // Set a low watermark to trigger failure
        let deque = VecDeque::<i32, _>::with_capacity_in(10, alloc.clone());
        assert!(deque.is_err());
        assert_eq!(alloc.in_use(), 0);
    }

    #[test]
    fn test_push_front_back() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());

        // Push elements to the front and back
        assert!(deque.push_back(1).is_ok());
        assert!(deque.push_front(2).is_ok());
        assert_eq!(deque.len(), 2);
        assert_eq!(deque.front(), Some(&2));
        assert_eq!(deque.back(), Some(&1));
    }

    #[test]
    fn test_push_front_back_allocation_failure() {
        let alloc = WatermarkAllocator::new(16); // Small watermark to limit allocations
        let mut deque = VecDeque::with_capacity_in(1, alloc.clone()).expect("should allocate");
        assert_eq!(deque.capacity(), 1); // overallocated by default.

        // Push first element should work
        assert!(deque.push_back(1).is_ok());
        // Second push should fail due to allocation error
        assert!(deque.push_back(2).is_err());
    }

    #[test]
    fn test_insert_remove() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());

        // Insert elements
        assert!(deque.push_back(1).is_ok());
        assert!(deque.push_back(3).is_ok());
        assert!(deque.insert(1, 2).is_ok());
        assert_eq!(deque.len(), 3);

        // Check order after insertion
        assert_eq!(deque.get(0), Some(&1));
        assert_eq!(deque.get(1), Some(&2));
        assert_eq!(deque.get(2), Some(&3));

        // Remove an element and check results
        assert_eq!(deque.remove(1), Some(2));
        assert_eq!(deque.len(), 2);
        assert_eq!(deque.get(1), Some(&3));
    }

    #[test]
    fn test_insert_allocation_failure() {
        let alloc = WatermarkAllocator::new(16); // Limited allocation capacity
        let mut deque = VecDeque::with_capacity_in(1, alloc.clone()).expect("should allocate");

        // First insert should succeed
        assert!(deque.push_back(1).is_ok());
        // Second insert, due to allocation, should fail
        assert!(deque.insert(1, 2).is_err());
    }

    #[test]
    fn test_append() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque1 = VecDeque::new_in(alloc.clone());
        let mut deque2 = VecDeque::new_in(alloc.clone());

        // Fill both deques
        assert!(deque1.push_back(1).is_ok());
        assert!(deque1.push_back(2).is_ok());
        assert!(deque2.push_back(3).is_ok());

        // Append deque2 into deque1
        assert!(deque1.append(&mut deque2).is_ok());
        assert_eq!(deque1.len(), 3);
        assert!(deque2.is_empty());
        assert_eq!(deque1.get(2), Some(&3));
    }

    #[test]
    fn test_append_allocation_failure() {
        let alloc = WatermarkAllocator::new(16);
        let mut deque1 = VecDeque::with_capacity_in(1, alloc.clone()).expect("should allocate");
        assert_eq!(deque1.capacity(), 1);
        assert_eq!(alloc.in_use(), deque1.capacity() * size_of::<i32>());
        assert_eq!(alloc.in_use(), 4);
        let mut deque2 = VecDeque::with_capacity_in(2, alloc.clone()).expect("should allocate");
        assert_eq!(deque2.capacity(), 2);
        assert_eq!(
            alloc.in_use(),
            deque1.capacity() * size_of::<i32>() + deque2.capacity() * size_of::<i32>()
        );
        assert_eq!(alloc.in_use(), 12);

        // Push items into deque2
        assert!(deque2.push_back(1).is_ok());
        assert!(deque2.push_back(2).is_ok());

        // Append should fail due to insufficient allocation capacity in deque1
        assert!(deque1.append(&mut deque2).is_err());
        assert!(!deque2.is_empty()); // deque2 should remain intact
    }

    #[test]
    fn test_try_clone() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();

        let cloned = deque.try_clone();
        assert!(cloned.is_ok());
        let cloned = cloned.unwrap();
        assert_eq!(cloned.len(), deque.len());
        assert_eq!(cloned.get(0), Some(&1));
        assert_eq!(cloned.get(1), Some(&2));
    }

    #[test]
    fn test_try_clone_allocation_failure() {
        let alloc = WatermarkAllocator::new(16); // Low watermark for testing allocation failure
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();

        // Cloning should fail due to allocation constraints
        let cloned = deque.try_clone();
        assert!(cloned.is_err());
    }

    #[test]
    fn test_get_mut() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();

        if let Some(value) = deque.get_mut(1) {
            *value = 3;
        }
        assert_eq!(deque.get(1), Some(&3));
    }

    #[test]
    fn test_iter() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();

        let values: Vec<_> = deque.iter().cloned().collect();
        assert_eq!(values, [1, 2, 3]);
    }

    #[test]
    fn test_iter_mut() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();

        for value in deque.iter_mut() {
            *value *= 2;
        }
        let values: Vec<_> = deque.iter().cloned().collect();
        assert_eq!(values, [2, 4, 6]);
    }

    #[test]
    fn test_range() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(10).unwrap();
        deque.push_back(20).unwrap();
        deque.push_back(30).unwrap();
        deque.push_back(40).unwrap();

        let values: Vec<_> = deque.range(1..3).cloned().collect();
        assert_eq!(values, [20, 30]);
    }

    #[test]
    fn test_range_mut() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(5).unwrap();
        deque.push_back(10).unwrap();
        deque.push_back(15).unwrap();

        for value in deque.range_mut(1..3) {
            *value += 10;
        }
        let values: Vec<_> = deque.iter().cloned().collect();
        assert_eq!(values, [5, 20, 25]);
    }

    #[test]
    fn test_drain() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();
        deque.push_back(4).unwrap();

        let drained: Vec<_> = deque.drain(1..3).collect();
        assert_eq!(drained, [2, 3]);
        assert_eq!(deque.len(), 2);
        assert_eq!(deque.get(1), Some(&4));
    }

    #[test]
    fn test_clear() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();

        deque.clear();
        assert!(deque.is_empty());
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn test_contains() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(42).unwrap();
        deque.push_back(99).unwrap();

        assert!(deque.contains(&42));
        assert!(!deque.contains(&1));
    }

    #[test]
    fn test_front_mut() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(5).unwrap();
        deque.push_back(10).unwrap();

        if let Some(value) = deque.front_mut() {
            *value = 7;
        }
        assert_eq!(deque.front(), Some(&7));
    }

    #[test]
    fn test_back_mut() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(5).unwrap();
        deque.push_back(10).unwrap();

        if let Some(value) = deque.back_mut() {
            *value = 15;
        }
        assert_eq!(deque.back(), Some(&15));
    }

    #[test]
    fn test_pop_front() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();

        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(deque.pop_front(), Some(2));
        assert!(deque.is_empty());
    }

    #[test]
    fn test_pop_back() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());
        deque.push_back(3).unwrap();
        deque.push_back(4).unwrap();

        assert_eq!(deque.pop_back(), Some(4));
        assert_eq!(deque.pop_back(), Some(3));
        assert!(deque.is_empty());
    }

    #[test]
    fn test_make_contiguous() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());

        // Alternate between front and back pushes to create a discontinuous buffer.
        deque.push_back(1).unwrap();
        deque.push_front(2).unwrap();
        deque.push_back(3).unwrap();
        deque.push_front(4).unwrap();
        deque.push_back(5).unwrap();

        // Calling make_contiguous should arrange elements in a contiguous slice.
        let slice = deque.make_contiguous();

        // Verify the order matches the intended sequence as if the buffer were continuous.
        assert_eq!(slice, &[4, 2, 1, 3, 5]);
    }

    #[test]
    fn test_try_clone_success() {
        let alloc = WatermarkAllocator::new(128);
        let mut deque = VecDeque::new_in(alloc.clone());

        // Populate the deque with some elements.
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();

        // Attempt to clone the deque.
        let cloned = deque.try_clone();

        // Verify the clone was successful and matches the original.
        assert!(cloned.is_ok());
        let cloned = cloned.unwrap();
        assert_eq!(cloned.len(), deque.len());
        assert_eq!(
            cloned.iter().collect::<Vec<_>>(),
            deque.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_try_clone_failure() {
        // Set a low watermark to trigger allocation failure during cloning.
        let alloc = WatermarkAllocator::new(16); // Low watermark for small allocations.
        let mut deque = VecDeque::new_in(alloc.clone());

        // Fill deque so it requires more allocation on cloning.
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();
        deque.push_back(4).unwrap();

        // Attempt to clone the deque. Expect an error due to allocation limit.
        let cloned = deque.try_clone();
        assert!(cloned.is_err());
    }

    #[test]
    fn test_try_clone_from_success() {
        let alloc = WatermarkAllocator::new(128);
        let mut original = VecDeque::new_in(alloc.clone());

        // Populate the original deque with some elements.
        original.push_back(1).unwrap();
        original.push_back(2).unwrap();
        original.push_back(3).unwrap();

        // Create a target deque with different contents to clone into.
        let mut target = VecDeque::new_in(alloc.clone());
        target.push_back(10).unwrap();
        target.push_back(20).unwrap();

        // Use try_clone_from to clone from the original deque into the target.
        let result = target.try_clone_from(&original);

        // Verify that the clone was successful.
        assert!(result.is_ok());

        // Check that the target now matches the original.
        assert_eq!(target.len(), original.len());
        assert_eq!(
            target.iter().collect::<Vec<_>>(),
            original.iter().collect::<Vec<_>>()
        );
    }
}
