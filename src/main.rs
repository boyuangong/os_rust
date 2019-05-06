#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use os_rust::println;
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB


#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os_rust::interrupts::PICS;

    unsafe {
        os_rust::HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    println!("Hello World{}", "!");

    os_rust::gdt::init();
    os_rust::interrupts::init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();


    println!("It did not crash!");
    os_rust::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os_rust::hlt_loop();
}
