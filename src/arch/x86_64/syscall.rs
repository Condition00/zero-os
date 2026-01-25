use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

//some syscall numbers
const SYS_WRITE: u64 = 0;
const SYS_EXIT: u64 = 1;
const SYS_YIELD: u64 = 2;

//kernel stack for he syscalls
const SYSCALL_STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
struct SyscallStack {
    data: [u8; SYSCALL_STACK_SIZE],
}

static mut SYSCALL_STACK: SyscallStack = SyscallStack {
    data: [0; SYSCALL_STACK_SIZE],
};

//syscall support

pub fn init() {
    // FUU forgot  ----- without this syscalls will not be enabled
    unsafe {
        Efer::update(|flags| {
            *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        });
    }

    //kernel stack for syscalls
    unsafe {
        let stack_top = VirtAddr::from_ptr(&raw const SYSCALL_STACK.data) + SYSCALL_STACK_SIZE;
        KERNEL_RSP = stack_top.as_u64();
    }

    LStar::write(VirtAddr::new(syscall_entry as u64));
    // Set segment selectors for syscall/sysret
    // Lower 32 bits: kernel CS/SS for syscall
    // Upper 32 bits: user CS/SS for sysret
    Star::write(
        crate::arch::x86_64::gdt::user_code_selector(),
        crate::arch::x86_64::gdt::user_data_selector(),
        crate::arch::x86_64::gdt::selectors().kernel_code_selector,
        crate::arch::x86_64::gdt::selectors().kernel_data_selector,
    )
    .expect("Failed to write STAR MSR");

    //masking interrupts during syscall
    SFMask::write(RFlags::INTERRUPT_FLAG);
}

//assembly syscall entry point
#[unsafe(naked)]
extern "C" fn syscall_entry() {
    core::arch::naked_asm!(
        // Save user stack pointer
        "mov [rip + USER_RSP], rsp",

        // Switch to kernel stack
        "mov rsp, [rip + KERNEL_RSP]",

        // Align stack to 16 bytes (required by System V ABI)
        "and rsp, ~0xf",
        "sub rsp, 8",        // Make it 8-byte misaligned (call will push 8 bytes, making it aligned)

        // Save user registers that we need to preserve
        "push rcx",          // User RIP (saved by CPU)
        "push r11",          // User RFLAGS (saved by CPU)

        // Call the Rust syscall handler
        // rax = syscall number, rdi = arg1, rsi = arg2, rdx = arg3
        "call {handler}",
        // Return value is in rax

        // Restore user registers
        "pop r11",           // User RFLAGS
        "pop rcx",           // User RIP

        // Restore user stack
        "mov rsp, [rip + USER_RSP]",

        // Return to userspace
        "sysretq",

        handler = sym syscall_handler,
    );
}

// Storage for stack pointers during syscall
#[no_mangle]
static mut USER_RSP: u64 = 0;
#[no_mangle]
static mut KERNEL_RSP: u64 = 0;

// Rust syscall handler
extern "C" fn syscall_handler(syscall_number: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match syscall_number {
        SYS_WRITE => sys_write(arg1, arg2, arg3),
        SYS_EXIT => sys_exit(arg1),
        SYS_YIELD => sys_yield(),
        _ => {
            crate::println!("[SYSCALL] Unknown syscall: {}", syscall_number);
            u64::MAX // Error: -1
        }
    }
}

//write text func

fn sys_write(fd: u64, buffer_ptr: u64, length: u64) -> u64 {
    //only supporting fd=1, stdput

    if fd != 1 {
        return u64::MAX; // Error
    }

    unsafe {
        let buffer = core::slice::from_raw_parts(buffer_ptr as *const u8, length as usize);

        // converting to string and print
        if let Ok(s) = core::str::from_utf8(buffer) {
            crate::print!("{}", s);
            length // Return bytes written
        } else {
            0 // Error: invalid UTF-8
        }
    }
}

fn sys_exit(exit_code: u64) -> u64 {
    crate::println!("[SYSCALL] User program exited with code: {}", exit_code);
    //halting the process for now we will exit the program later
    crate::hlt_loop();
}

fn sys_yield() -> u64 {
    crate::println!("[SYSCALL] Yield Called");
    0
}
