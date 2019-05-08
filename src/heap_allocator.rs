use crate::hole::{HoleList, Hole, align_up};
use alloc::alloc::{Alloc, AllocErr, Layout};
use core::ptr::NonNull;
use core::ptr::null_mut;
use core::alloc::{GlobalAlloc};
use core::mem;

use core::ops::Deref;

use spin::Mutex;

/// A fixed size heap backed by a linked list of free memory blocks.
pub struct HeapAllocator {
    bottom: usize,
    size: usize,
    holes: HoleList,
}

impl HeapAllocator {
    pub const fn empty() -> HeapAllocator {
        HeapAllocator {
            bottom: 0,
            size: 0,
            holes: HoleList::empty(),
        }
    }

    /// init help with given start point and size
    /// The allocator will claim memory `[heap_bottom, heap_bottom + heap_size)`
    pub unsafe fn init(&mut self, heap_bottom: usize, heap_size: usize) {

        self.bottom = heap_bottom;
        self.size = heap_size;
        self.holes = HoleList::new(heap_bottom, heap_size);
    }


    /// call allocate_first_fit in Holes;
    /// If the layout size is smaller than the min_size, function will extend the layout
    /// to the min_size;
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let mut size = layout.size();
        if size < HoleList::min_size() {
            size = HoleList::min_size();
        }

        let layout = Layout::from_size_align(size, layout.align()).unwrap();

        self.holes.alloc(layout)
    }


    /// Deallocate the given pointer 'ptr' which point memory allocate by the current allocator
    /// If the layout size is smaller than the min_size, function will extend the layout
    /// to the min_size;
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let mut size = layout.size();
        if size < HoleList::min_size() {
            size = HoleList::min_size();
        }
        let layout = Layout::from_size_align(size, layout.align()).unwrap();

        self.holes.deallocate(ptr, layout);
    }

    /// Returns the bottom address of the heap.
    pub fn bottom(&self) -> usize {
        self.bottom
    }

    /// Returns the size of the heap.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Return the top address of the heap
    pub fn top(&self) -> usize {
        self.bottom + self.size
    }

    pub fn first_hole(&self) -> Option<(usize, usize)> {
        self.holes.first_hole()
    }

}

unsafe impl Alloc for HeapAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        self.alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.dealloc(ptr, layout)
    }
}



// Mutex ensured the Alloc can be shared reference
pub struct GlobalHeapAllocator(Mutex<HeapAllocator>);


impl GlobalHeapAllocator {
    pub const fn empty() -> GlobalHeapAllocator {
        GlobalHeapAllocator(Mutex::new(HeapAllocator::empty()))
    }

    pub unsafe fn new(heap_bottom: usize, heap_size: usize) -> GlobalHeapAllocator {
        GlobalHeapAllocator(Mutex::new(HeapAllocator {
            bottom: heap_bottom,
            size: heap_size,
            holes: HoleList::new(heap_bottom, heap_size),
        }))
    }
}


//Dereference implementation for LockedHeap for lock()
impl Deref for GlobalHeapAllocator {
    type Target = Mutex<HeapAllocator>;

    fn deref(&self) -> &Mutex<HeapAllocator> {
        &self.0
    }
}

// Implement GlobalAllocator as required by alloc
unsafe impl GlobalAlloc for GlobalHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .dealloc(NonNull::new_unchecked(ptr), layout)
    }
}



