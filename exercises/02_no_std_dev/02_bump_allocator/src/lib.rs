//! # Bump Allocator (no_std)
//!
//! Implement the simplest heap memory allocator: a Bump Allocator (bump pointer allocator).
//!
//! ## How It Works
//!
//! A Bump Allocator maintains a pointer `next` to the "next available address".
//! On each allocation, it aligns `next` to the requested alignment, then advances by `size` bytes.
//! It does not support freeing individual objects (`dealloc` is a no-op).
//!
//! ```text
//! heap_start                              heap_end
//! |----[allocated]----[allocated]----| next |---[free]---|
//!                                        ^
//!                                    next allocation starts here
//! ```
//!
//! ## Task
//!
//! Implement `BumpAllocator`'s `GlobalAlloc::alloc` method:
//! 1. Align the current `next` up to `layout.align()`
//!    Hint: `align_up(addr, align) = (addr + align - 1) & !(align - 1)`
//! 2. Check if the aligned address plus `layout.size()` exceeds `heap_end`
//! 3. If it exceeds, return `null_mut()`; otherwise atomically update `next` with `compare_exchange`
//!
//! ## Key Concepts
//!
//! - `core::alloc::{GlobalAlloc, Layout}`
//! - Memory alignment calculation
//! - `AtomicUsize` and `compare_exchange` (CAS loop)

#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: AtomicUsize,
}

impl BumpAllocator {
    /// Create a new BumpAllocator.
    ///
    /// # Safety
    /// `heap_start..heap_end` must be a valid, readable and writable memory region,
    /// and must not be used by other code during this allocator's lifetime.
    pub const unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }

    /// Reset the allocator (free all allocated memory).
    pub fn reset(&self) {  
        self.next.store(self.heap_start, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // TODO: Implement bump allocation
        //
        // Steps:
        // 1. Load current next (use Ordering::SeqCst)
        // 2. Align next up to layout.align()
        //    Hint: align_up(addr, align) = (addr + align - 1) & !(align - 1)
        // 3. Compute allocation end = aligned + layout.size()
        // 4. If end > heap_end, return null_mut()
        // 5. Atomically update next to end using compare_exchange
        //    (if CAS fails, another thread raced — retry in a loop)
        // 6. Return the aligned address as a pointer
        // CAS 循环：如果 compare_exchange 失败，说明其他线程修改了 next，需要重试
    loop {
            // 1. 加载当前的 next
            let current_next = self.next.load(Ordering::SeqCst);
            
            // 2. 将 next 向上对齐到 layout.align()
            let align = layout.align();
            let aligned = (current_next + align - 1) & !(align - 1);
            
            // 3. 计算分配结束地址
            let allocation_end = aligned.checked_add(layout.size()).unwrap_or(usize::MAX);
            
            // 4. 检查是否超出堆范围
            if allocation_end > self.heap_end {
                return null_mut(); // 内存不足
            }
            
            // 5. 尝试原子地更新 next
            match self.next.compare_exchange(
                current_next,  // 期望当前值是 current_next
                allocation_end, // 如果匹配，设置为 allocation_end
                Ordering::SeqCst, // 成功时的内存顺序
                Ordering::SeqCst, // 失败时的内存顺序
            ) {
                Ok(_) => {
                    // CAS 成功：返回对齐后的地址
                    return aligned as *mut u8;
                }
                Err(_) => {
                    // CAS 失败：其他线程已经更新了 next，重新开始循环
                    continue;
                }
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator does not reclaim individual objects — leave empty
    }
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    const HEAP_SIZE: usize = 4096;

    fn make_allocator() -> (BumpAllocator, Vec<u8>) {
        let mut heap = vec![0u8; HEAP_SIZE];
        let start = heap.as_mut_ptr() as usize;
        let alloc = unsafe { BumpAllocator::new(start, start + HEAP_SIZE) };
        (alloc, heap)
    }

    #[test]
    fn test_alloc_basic() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null(), "allocation should succeed");
    }

    #[test]
    fn test_alloc_alignment() {
        let (alloc, _heap) = make_allocator();
        for align in [1, 2, 4, 8, 16, 64] {
            let layout = Layout::from_size_align(1, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null());
            assert_eq!(
                ptr as usize % align,
                0,
                "returned address must satisfy align={align}"
            );
        }
    }

    #[test]
    fn test_alloc_no_overlap() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();
        let p1 = unsafe { alloc.alloc(layout) } as usize;
        let p2 = unsafe { alloc.alloc(layout) } as usize;
        assert!(
            p1 + 64 <= p2 || p2 + 64 <= p1,
            "two allocations must not overlap"
        );
    }

    #[test]
    fn test_alloc_oom() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE + 1, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when exceeding heap");
    }

    #[test]
    fn test_alloc_fill_heap() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(256, 1).unwrap();
        for i in 0..16 {
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null(), "allocation #{i} should succeed");
        }
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when heap is full");
    }

    #[test]
    fn test_reset() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE, 1).unwrap();
        let p1 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null());
        alloc.reset();
        let p2 = unsafe { alloc.alloc(layout) };
        assert!(!p2.is_null(), "should be able to allocate after reset");
        assert_eq!(
            p1, p2,
            "address after reset should match the first allocation"
        );
    }
}
