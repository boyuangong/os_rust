#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use os_rust::println;
use core::panic::PanicInfo;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use os_rust::interrupts::PICS;

    println!("Hello World{}", "!");

    os_rust::gdt::init();
    os_rust::interrupts::init_idt();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

//    // invoke a breakpoint exception
//    x86_64::instructions::int3();

//    fn stack_overflow() {
//        stack_overflow(); // for each recursion, the return address is pushed
//    }
//
//    // trigger a stack overflow
//    stack_overflow();

    println!("It did not crush!");

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
