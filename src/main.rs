#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]
#![feature(alloc)]
use os_rust::println;
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;
use os_rust::memory;
#[macro_use]
extern crate alloc;
entry_point!(kernel_main);

pub const HEAP_START: usize = 0o_000_000_000_000_0000;
pub const HEAP_SIZE: usize = 1000 * 1024; // 100 KiB


#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os_rust::interrupts::PICS;

    println!("Hello World{}", "!");

    os_rust::gdt::init();
    os_rust::interrupts::init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let mut recursive_page_table = unsafe { memory::init(boot_info.p4_table_addr as usize) };

    println!("p4 table address at {:#x}",boot_info.p4_table_addr);

    unsafe{
        os_rust::HEAP_ALLOCATOR.lock().init(boot_info.p4_table_addr as usize + 0x10, HEAP_SIZE);
    }


    println!("first hole of the allocator at {:?}", os_rust::HEAP_ALLOCATOR.lock().first_hole());
    println!("bottom of the allocator at {:#x}", os_rust::HEAP_ALLOCATOR.lock().bottom());

    println!("Test small size layout");
    let mut vec_test1 = vec![1];
    for i in &vec_test1 {
        println!("{} ", i);
    }

    println!("Test a vector");
    let mut vec_test = vec![1i32, 2, 3, 4, 5];
    for i in &vec_test {
        println!("{} ", i);
    }


    println!("Change of first hole");
    println!("first hole of the allocator at {:?}", os_rust::HEAP_ALLOCATOR.lock().first_hole());


    println!("Test push into a vector");
    for i in 10..20{
        vec_test.push(i);
    }
    println!("{:?}", vec_test);

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
