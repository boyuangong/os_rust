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