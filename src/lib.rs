#![cfg_attr(not(test), no_std)]
#![feature(abi_x86_interrupt)]
#![feature(const_fn)]
#![feature(global_allocator)]

//following for the alloc
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]


#![feature(lang_items)]

extern crate alloc;
#[cfg(feature = "use_spin")]
extern crate spin;
use alloc::alloc::{Layout};

pub mod gdt;
pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod memory;
pub mod hole;
pub mod heap_allocator;

use heap_allocator::LockedHeap;


pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {

    loop {}
}


#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
