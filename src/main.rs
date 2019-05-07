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

    println!("p4 address at {}",boot_info.p4_table_addr);

    unsafe{
        os_rust::HEAP_ALLOCATOR.lock().init(boot_info.p4_table_addr as usize + 0x10, HEAP_SIZE);
    }

//    for i in 0..10000 {
//        println!("Some String");
//    }

    println!("{:?}", os_rust::HEAP_ALLOCATOR.lock().first_hole());

    let mut vec_test1 = vec![1];
    for i in &vec_test1 {
        println!("{} ", i);
    }

    let mut vec_test = vec![1i32, 2, 3, 4, 5];
    for i in &vec_test {
        println!("{} ", i);
    }

    println!("{:?}", os_rust::HEAP_ALLOCATOR.lock().first_hole());

//    for i in 10..20{
//        vec_test.push(i);
//    }

    println!("{:?}", os_rust::HEAP_ALLOCATOR.lock().first_hole());

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
