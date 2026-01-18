#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(zero::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use zero::println;
use zero::task::{executor::Executor, keyboard, Task};

entry_point!(kernel_main);
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use zero::allocator;
    use zero::memory;
    use zero::memory::BootInfoFrameAllocator;

    use zero::{input, terminal};

    println!("ZERO OS\n");
    zero::init();

    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&_boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap inititalization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(async {
        loop {
            terminal::write("zero-os> ");
            terminal::mark_input_start();
            let line = input::read_line().await;
            terminal::write("You typed: ");
            terminal::write(&line);
            terminal::write("\n");
        }
    }));
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
