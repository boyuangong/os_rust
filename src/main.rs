#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use os_rust::println;
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;


entry_point!(kernel_main);

#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os_rust::interrupts::PICS;
    use os_rust::memory::{self, create_example_mapping, EmptyFrameAllocator};
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{
        FrameAllocator, Mapper, Page, PageTable, PhysFrame, RecursivePageTable, Size4KiB,
    };

    println!("Hello World{}", "!");

    os_rust::gdt::init();
    os_rust::interrupts::init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let mut recursive_page_table = unsafe { memory::init(boot_info.p4_table_addr as usize) };
    let mut frame_allocator = memory::init_frame_allocator(&boot_info.memory_map);

    for _i in 1..100 {
        println!("{:?}", frame_allocator.allocate_frame());
    }



    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x20010a,
        // some stack page
        0x57ac_001f_fe48,
        // virtual address mapped to physical address 0
    ];

    for &address in &addresses {
        let page: Page = Page::containing_address(VirtAddr::new(address));
        // new: use the `mapper.translate_addr` method
        let phys = recursive_page_table.translate_page(page);
        println!("{:?} -> {:?}", page, phys);
    }


    create_example_mapping(&mut recursive_page_table, &mut frame_allocator);
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e) };


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
