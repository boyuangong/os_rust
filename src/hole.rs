use alloc::alloc::{AllocErr, Layout};
use core::ptr::NonNull;
use core::mem::size_of;

pub struct HoleList {
    first: Hole, // dummy
}

impl HoleList {
    /// Creates an empty `HoleList`.
    pub const fn empty() -> HoleList {
        HoleList {
            first: Hole {
                size: 0,
                next: None,
            },
        }
    }

    /// Creates a `HoleList` that contains the given hole. This function is unsafe because it
    /// creates a hole at the given `hole_addr`. This can cause undefined behavior if this address
    /// is invalid or if memory from the `[hole_addr, hole_addr+size) range is used somewhere else.
    pub unsafe fn new(hole_addr: usize, hole_size: usize) -> HoleList {
        assert!(size_of::<Hole>() == Self::min_size());

        let ptr = hole_addr as *mut Hole;
        ptr.write(Hole {
            size: hole_size,
            next: None,
        });

        HoleList {
            first: Hole {
                size: 0,
                next: Some(&mut *ptr),
            },
        }
    }

    /// Searches the list for a big enough hole. A hole is big enough if it can hold an allocation
    /// of `layout.size()` bytes with the given `layout.align()`. If such a hole is found in the
    /// list, a block of the required size is allocated from it. Then the start address of that
    /// block is returned.
    /// This function uses the “first fit” strategy, so it uses the first hole that is big
    /// enough. Thus the runtime is in O(n) but it should be reasonably fast for small allocations.
    pub fn allocate_first_fit(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        assert!(layout.size() >= Self::min_size());

        allocate_first_fit(&mut self.first, layout).map(|allocation| {
            NonNull::new(allocation.info.addr as *mut u8).unwrap()
        })
    }

    /// Returns the minimal allocation size. Smaller allocations or deallocations are not allowed.
    pub fn min_size() -> usize {
        size_of::<usize>() * 2
    }

}

pub struct Hole {
    size: usize,
    next: Option<&'static mut Hole>,
}

impl Hole {
    /// Returns basic information about the hole.
    fn info(&self) -> HoleInfo {
        HoleInfo {
            addr: self as *const _ as usize,
            size: self.size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct HoleInfo {
    addr: usize,
    size: usize,
}

/// The result returned by `split_hole` and `allocate_first_fit`. Contains the address and size of
/// the allocation (in the `info` field), and the front and back padding.
struct Allocation {
    info: HoleInfo,
    front_padding: Option<HoleInfo>,
    back_padding: Option<HoleInfo>,
}

/// Searches the list starting at the next hole of `previous` for a big enough hole. A hole is big
/// enough if it can hold an allocation of `layout.size()` bytes with the given `layou.align()`.
/// When a hole is used for an allocation, there may be some needed padding before and/or after
/// the allocation. This padding is returned as part of the `Allocation`. The caller must take
/// care of freeing it again.
/// This function uses the “first fit” strategy, so it breaks as soon as a big enough hole is
/// found (and returns it).
fn allocate_first_fit(mut previous: &mut Hole, layout: Layout) -> Result<Allocation, AllocErr> {
    loop {
        let allocation: Option<Allocation> = previous
            .next
            .as_mut()
            .and_then(|current| split_hole(current.info(), layout.clone()));
        match allocation {
            Some(allocation) => {
                // hole is big enough, so remove it from the list by updating the previous pointer
                previous.next = previous.next.as_mut().unwrap().next.take();
                return Ok(allocation);
            }
            None if previous.next.is_some() => {
                // try next hole
                previous = previous.next.as_mut().unwrap();
            }
            None => {
                // this was the last hole, so no hole is big enough -> allocation not possible
                return Err(AllocErr);
            }
        }
    }
}


/// Splits the given hole into `(front_padding, hole, back_padding)` if it's big enough to allocate
/// `required_layout.size()` bytes with the `required_layout.align()`. Else `None` is returned.
/// Front padding occurs if the required alignment is higher than the hole's alignment. Back
/// padding occurs if the required size is smaller than the size of the aligned hole. All padding
/// must be at least `HoleList::min_size()` big or the hole is unusable.
fn split_hole(hole: HoleInfo, required_layout: Layout) -> Option<Allocation> {
    return None;
}



/// A fixed size heap backed by a linked list of free memory blocks.
pub struct Heap {
    bottom: usize,
    size: usize,
    holes: HoleList,
}

impl Heap {
    /// Creates an empty heap. All allocate calls will return `None`.
    pub const fn empty() -> Heap {
        Heap {
            bottom: 0,
            size: 0,
            holes: HoleList::empty(),
        }
    }
}