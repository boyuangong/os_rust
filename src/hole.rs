use alloc::alloc::{AllocErr, Layout};
use core::ptr::NonNull;
use core::mem::size_of;


pub struct HoleList {
    head: Hole,
}

impl HoleList {
    /// Creates an empty `HoleList`.
    pub const fn empty() -> HoleList {
        HoleList {
            head: Hole {
                size: 0,
                next: None,
            },
        }
    }

    /// Create new hole for empty
    /// # Unsafe
    /// The call will be invalide if `[hole_addr, hole_addr+size) range is used somewhere else.
    pub unsafe fn new(hole_addr: usize, hole_size: usize) -> HoleList {
        assert_eq!(size_of::<Hole>(), Self::min_size());

        let ptr = hole_addr as *mut Hole;
        ptr.write(Hole {
            size: hole_size,
            next: None,
        });

        HoleList {
            head: Hole {
                size: 0,
                next: Some(&mut *ptr),
            },
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        assert!(layout.size() >= Self::min_size());

        allocate_first_fit(&mut self.head, layout).map(|allocation| {
            if let Some(front_hole_info) = allocation.front_hole_info {
                deallocate(&mut self.head, front_hole_info.addr, front_hole_info.size);
            }
            if let Some(back_hole_info) = allocation.back_hole_info {
                deallocate(&mut self.head, back_hole_info.addr, back_hole_info.size);
            }
            NonNull::new(allocation.allocated_info.addr as *mut u8).unwrap()
        })
    }

    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        deallocate(&mut self.head, ptr.as_ptr() as usize, layout.size())
    }

    /// Returns the minimal allocation size. Smaller allocations or deallocations are not allowed.
    pub fn min_size() -> usize {
        size_of::<usize>() * 2
    }

    pub fn first_hole(&self) -> Option<(usize, usize)> {
        self.head
            .next
            .as_ref()
            .map(|hole| ((*hole) as *const Hole as usize, hole.size))
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

/// Contains the address and size of the allocation (in the `info` field),
/// and the front and back padding.
struct AllocInfo {
    allocated_info: HoleInfo,
    front_hole_info: Option<HoleInfo>,
    back_hole_info: Option<HoleInfo>,
}

// Search for the first fit hole
fn allocate_first_fit(mut previous: &mut Hole, layout: Layout) -> Result<AllocInfo, AllocErr> {
    loop {
        let alloc_info: Option<AllocInfo> = previous
            .next
            .as_mut()
            .and_then(|current| split_hole(current.info(), layout.clone()));
        match alloc_info {
            Some(alloc_info) => {
                // hole is big enough, so remove it from the list by updating the previous pointer
                previous.next = previous.next.as_mut().unwrap().next.take();
                return Ok(alloc_info);
            }
            None if previous.next.is_some() => {
                // try next hole
                previous = previous.next.as_mut().unwrap();
            }
            None => {
                // this was the last hole, so no hole is big enough -> Allocation not possible
                return Err(AllocErr);
            }
        }
    }
}


/// Splits the given hole into `(front_padding, hole, back_padding)`. None will returned if
/// size is bigger than hole
fn split_hole(hole: HoleInfo, layout: Layout) -> Option<AllocInfo> {
    if layout.size() > hole.size {
        return None;
    }

    // Work with alignment to decide front hole
    let (aligned_addr, front_hole) = if hole.addr == align_up(hole.addr, layout.align()) {
        // hole has already the required alignment
        (hole.addr, None)
    } else {
        // the required alignment causes some padding before the allocation
        let aligned_addr = align_up(hole.addr + HoleList::min_size(), layout.align());
        (
            aligned_addr,
            Some(HoleInfo {
                addr: hole.addr,
                size: aligned_addr - hole.addr,
            }),
        )
    };

    if aligned_addr - hole.addr > hole.size - layout.size() {
        // hole is too small after alignment
        return None;
    }

    // Work with allocation information
    let allocated_hole = {
        HoleInfo {
            addr: aligned_addr,
            size: hole.size - (aligned_addr - hole.addr),
        }
    };

    let back_hole = if allocated_hole.size - layout.size() < HoleList::min_size() {
        None
    } else {
        // the hole is bigger than necessary, so there is some padding behind the allocation
        Some(HoleInfo {
            addr: allocated_hole.addr + layout.size(),
            size: allocated_hole.size - layout.size(),
        })
    };

    Some(AllocInfo {
        allocated_info: HoleInfo {
            addr: allocated_hole.addr,
            size: layout.size(),
        },
        front_hole_info: front_hole,
        back_hole_info: back_hole,
    })
}




/// refer https://os.phil-opp.com/kernel-heap/#alignment
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

/// Frees the allocation given by `(addr, size)`.
fn deallocate(mut hole: &mut Hole, addr: usize, mut size: usize) {
    loop {
        assert!(size >= HoleList::min_size());

        let hole_addr = if hole.size == 0 {
            // It's the dummy hole, which is the head of the HoleList.
            0
        } else {
            hole as *mut _ as usize
        };

        // Each freed block must be handled by the previous hole in memory. Thus the freed
        // address must be always behind the current hole.
        assert!(
            hole_addr <= addr - hole.size,
            "invalid deallocation (probably a double free)"
        );

        // get information about the next block
        let next_hole_info = hole.next.as_ref().map(|next| next.info());

        match next_hole_info {
            Some(next) if hole_addr + hole.size == addr && addr + size == next.addr => {
                // block fills the gap between two holes, merge the three block
                // remove the second hole
                hole.size += size + next.size;
                hole.next = hole.next.as_mut().unwrap().next.take();
            }
            _ if hole_addr + hole.size == addr => {
                // block concatenate with the hole before
                hole.size += size;
            }
            Some(next) if addr == next.addr - size => {
                // block concatenate with the hole after
                hole.next = hole.next.as_mut().unwrap().next.take();
                size += next.size;
                continue;
            }
            Some(next) if next.addr <= addr => {
                // block after the hole after
                hole = move_helper(hole).next.as_mut().unwrap();
                continue;
            }
            _ => {
                // block between the two hole and does not concatenate
                let new_hole = Hole {
                    size: size,
                    next: hole.next.take(),
                };
                // write the new hole to the freed memory
                let ptr = addr as *mut Hole;
                unsafe { ptr.write(new_hole) };
                // add the F block as the next block of the X block
                hole.next = Some(unsafe { &mut *ptr });
            }
        }
        break;
    }
}

fn move_helper<T>(x: T) -> T {
    x
}