use core::u64;

use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

use alloc::string::String;
use alloc::vec::Vec;

//some syscall numbers
const SYS_READ: u64 = 0;
const SYS_WRITE: u64 = 1;
const SYS_OPEN: u64 = 2;
const SYS_CLOSE: u64 = 3;
const SYS_STAT: u64 = 4;
const SYS_READDIR: u64 = 5;
const SYS_MKDIR: u64 = 6;
const SYS_TOUCH: u64 = 7;
const SYS_RM: u64 = 8;
const SYS_CLEAR: u64 = 9;
const SYS_REBOOT: u64 = 10;
const SYS_EXIT: u64 = 11;
const SYS_YIELD: u64 = 12;

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

        "mov rcx, rdx",      // arg3: rdx -> rcx
        "mov rdx, rsi",      // arg2: rsi -> rdx
        "mov rsi, rdi",      // arg1: rdi -> rsi
        "mov rdi, rax",
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
        SYS_READ => sys_read(arg1, arg2, arg3),
        SYS_WRITE => sys_write(arg1, arg2, arg3),
        SYS_EXIT => sys_exit(arg1),
        SYS_YIELD => sys_yield(),
        SYS_OPEN => sys_open(arg1, arg2),
        SYS_CLOSE => sys_close(arg1),
        SYS_READDIR => sys_readdir(arg1, arg2, arg3),
        SYS_STAT => sys_stat(arg1, arg2),
        SYS_MKDIR => sys_mkdir(arg1),
        SYS_TOUCH => sys_touch(arg1),
        SYS_RM => sys_rm(arg1),
        SYS_CLEAR => sys_clear(),
        SYS_REBOOT => sys_reboot(),
        _ => {
            crate::println!("[SYSCALL] Unknown syscall: {}", syscall_number);
            u64::MAX // Error: -1
        }
    }
}

//write text func
fn sys_read(fd: u64, _buffer_ptr: u64, length: u64) -> u64 {
    if fd != 0 || length == 0 {
        // Only stdin supported
        return 0;
    }

    // For now, stub - will implement with keyboard driver integration
    crate::println!("[SYSCALL] sys_read called - not yet implemented");
    0
}

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
    // For now, we can't properly return to kernel from userspace
    // The best we can do is halt the process
    // TODO: Implement proper process termination and return to kernel
    crate::println!("[SYSCALL] Halting - will implement proper process exit later");
    crate::hlt_loop();
}

fn sys_yield() -> u64 {
    crate::println!("[SYSCALL] Yield Called");
    0
}

fn sys_open(path_ptr: u64, _flags: u64) -> u64 {
    unsafe {
        let path_bytes = read_string_from_user(path_ptr);
        if path_bytes.is_empty() {
            return u64::MAX;
        }
        if let Ok(path) = core::str::from_utf8(&path_bytes) {
            // Check if file exists in filesystem
            if let Some(fs) = crate::kernel::fs::root() {
                match fs.stat(path) {
                    Ok(_) => {
                        // File exists - return a fake fd for now
                        // Later: maintain fd table per process
                        return 3; // Fake fd
                    }
                    Err(_) => return u64::MAX, // File not found
                }
            }
        }
        u64::MAX
    }
}

//file close
fn sys_close(_fd: u64) -> u64 {
    // For now, always succeed
    0
}

// Read directory contents
fn sys_readdir(path_ptr: u64, buffer_ptr: u64, buffer_size: u64) -> u64 {
    unsafe {
        let path_bytes = read_string_from_user(path_ptr);
        if path_bytes.is_empty() {
            return u64::MAX;
        }

        if let Ok(path) = core::str::from_utf8(&path_bytes) {
            if let Some(fs) = crate::kernel::fs::root() {
                match fs.list_dir(path) {
                    Ok(entries) => {
                        use alloc::format;
                        let mut output = String::new();
                        for entry in entries {
                            let type_char = match entry.file_type {
                                crate::kernel::fs::FileType::Directory => 'd',
                                crate::kernel::fs::FileType::File => 'f',
                            };
                            output.push_str(&format!(
                                "{} {:8} {}\n",
                                type_char, entry.size, entry.name
                            ));
                        }

                        let bytes = output.as_bytes();
                        let copy_len = bytes.len().min(buffer_size as usize);
                        copy_to_user(buffer_ptr, &bytes[..copy_len]);
                        return copy_len as u64;
                    }
                    Err(_) => return u64::MAX,
                }
            }
        }
        u64::MAX
    }
}

// Get file/directory stats
fn sys_stat(path_ptr: u64, statbuf_ptr: u64) -> u64 {
    unsafe {
        let path_bytes = read_string_from_user(path_ptr);
        if path_bytes.is_empty() {
            return u64::MAX;
        }

        if let Ok(path) = core::str::from_utf8(&path_bytes) {
            if let Some(fs) = crate::kernel::fs::root() {
                match fs.stat(path) {
                    Ok(inode) => {
                        // Write stat info to user buffer
                        // Format: [file_type (1 byte), size (8 bytes)]
                        let file_type = match inode.file_type {
                            crate::kernel::fs::FileType::Directory => 1u8,
                            crate::kernel::fs::FileType::File => 0u8,
                        };

                        let stat_data = [
                            file_type,
                            (inode.size & 0xFF) as u8,
                            ((inode.size >> 8) & 0xFF) as u8,
                            ((inode.size >> 16) & 0xFF) as u8,
                            ((inode.size >> 24) & 0xFF) as u8,
                            ((inode.size >> 32) & 0xFF) as u8,
                            ((inode.size >> 40) & 0xFF) as u8,
                            ((inode.size >> 48) & 0xFF) as u8,
                            ((inode.size >> 56) & 0xFF) as u8,
                        ];

                        copy_to_user(statbuf_ptr, &stat_data);
                        return 0; // Success
                    }
                    Err(_) => return u64::MAX,
                }
            }
        }
        u64::MAX
    }
}

// Create directory
fn sys_mkdir(path_ptr: u64) -> u64 {
    unsafe {
        let path_bytes = read_string_from_user(path_ptr);
        if path_bytes.is_empty() {
            return u64::MAX;
        }

        if let Ok(path) = core::str::from_utf8(&path_bytes) {
            if let Some(fs) = crate::kernel::fs::root() {
                return match fs.create_dir(path) {
                    Ok(_) => 0,
                    Err(_) => u64::MAX,
                };
            }
        }
        u64::MAX
    }
}

// Create empty file
fn sys_touch(path_ptr: u64) -> u64 {
    unsafe {
        let path_bytes = read_string_from_user(path_ptr);
        if path_bytes.is_empty() {
            return u64::MAX;
        }

        if let Ok(path) = core::str::from_utf8(&path_bytes) {
            if let Some(fs) = crate::kernel::fs::root() {
                return match fs.create_file(path) {
                    Ok(_) => 0,
                    Err(_) => u64::MAX,
                };
            }
        }
        u64::MAX
    }
}

// Remove file or directory
fn sys_rm(path_ptr: u64) -> u64 {
    unsafe {
        let path_bytes = read_string_from_user(path_ptr);
        if path_bytes.is_empty() {
            return u64::MAX;
        }

        if let Ok(path) = core::str::from_utf8(&path_bytes) {
            if let Some(fs) = crate::kernel::fs::root() {
                return match fs.remove(path) {
                    Ok(_) => 0,
                    Err(_) => u64::MAX,
                };
            }
        }
        u64::MAX
    }
}

// Clear terminal screen
fn sys_clear() -> u64 {
    crate::ui::terminal::clear();
    0
}

// Reboot system
fn sys_reboot() -> u64 {
    crate::arch::x86_64::cpu::reboot();
}

// Helper functions for userspace memory access
unsafe fn read_string_from_user(ptr: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut offset = 0;
    loop {
        let byte = *((ptr + offset) as *const u8);
        if byte == 0 {
            break;
        }
        bytes.push(byte);
        offset += 1;
        if offset > 4096 {
            break; // Safety limit
        }
    }
    bytes
}

unsafe fn copy_to_user(ptr: u64, data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        *((ptr + i as u64) as *mut u8) = byte;
    }
}
