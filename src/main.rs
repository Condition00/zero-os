#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(zero::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use zero::drivers::keyboard;
use zero::kernel::memory::allocator;
use zero::kernel::memory::memory;
use zero::kernel::memory::memory::BootInfoFrameAllocator;
use zero::kernel::task::{executor::Executor, Task};
use zero::println;
use zero::ui::shell;

entry_point!(kernel_main);
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;

    println!("                                 ZERO OS\n");
    zero::init();
    zero::arch::x86_64::gdt::test_user_segments();

    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&_boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap inititalization failed");
    println!("heap allocator initialized...");

    zero::kernel::fs::init();
    println!("ramfs initialized...\n");

    #[cfg(test)]
    test_main();

    //jumpin
    println!("[USERSPACE]: Jumping to Userspace...");
    let entry = zero::kernel::userspace::get_user_function_addr();
    let stack_top = zero::kernel::userspace::user_stack_top();
    unsafe { zero::kernel::userspace::jump_to_userspace(entry, stack_top) }

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(shell::shell()));
    executor.run();
}

// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    zero::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    zero::test_panic_handler(info)
}
