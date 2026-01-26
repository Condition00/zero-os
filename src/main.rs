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

    // Optional userspace test - comment out to skip straight to shell
    {
        println!("[USERSPACE]: Jumping to Userspace test...");

        let user_code: &[u8] = &[
            0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, // mov rax, 1 (SYS_WRITE)
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1 (stdout)
            0x48, 0x8d, 0x35, 0x19, 0x00, 0x00, 0x00, // lea rsi, [rip+25]
            0x48, 0xc7, 0xc2, 0x0f, 0x00, 0x00, 0x00, // mov rdx, 15
            0x0f, 0x05, // syscall
            // Exit with code 42
            0x48, 0xc7, 0xc0, 0x0b, 0x00, 0x00, 0x00, // mov rax, 11 (SYS_EXIT)
            0x48, 0xc7, 0xc7, 0x2a, 0x00, 0x00, 0x00, // mov rdi, 42
            0x0f, 0x05, // syscall
            // Message data: "Hello from R3!\n"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x66, 0x72, // "Hello fr"
            0x6f, 0x6d, 0x20, 0x52, 0x33, 0x21, 0x0a, // "om R3!\n"
        ];

        //loading user program into memory space:
        let entry = zero::kernel::userspace::load_user_program(
            user_code,
            &mut mapper,
            &mut frame_allocator,
        )
        .expect("failed to load user program");
        //allocating the user stack
        let user_stack =
            zero::kernel::userspace::allocate_user_stack(&mut mapper, &mut frame_allocator)
                .expect("failed to allocate user stack");

        unsafe {
            zero::kernel::userspace::jump_to_userspace(entry, user_stack);
        }
    }

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
