use x86_64::VirtAddr; 

pub fn jump_to_userspace(entry_point: VirtAddr, user_stack: VirtAddr) -> ! {
    let code_selector = crate::arch::x86_64::gdt::user_code_selector();
    let data_selector = crate::arch::x86_64::gdt::user_data_selector();

    unsafe {
        core::arch::asm!(
            "mov ax, {0:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",

            //iretq stack frame
            "push {0:r}",            // SS (stack segment)
            "push {1:r}",            // RSP (user stack pointer)
            "push 0x200",            // RFLAGS (interrupt enable)
            "push {2:r}",            // CS (code segment)
            "push {3:r}",            // RIP (entry point)

            "iretq",

            in(reg) data_selector.0,
            in(reg) user_stack.as_u64(),
            in(reg) code_selector.0,
            in(reg) entry_point.as_u64(),
            options(noreturn)
        );
    }
}

