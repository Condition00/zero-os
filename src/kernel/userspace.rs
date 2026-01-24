use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

const USER_STACK_SIZE: usize = 4096 * 20;

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

//allocates fresh phy frames and maps them user virtual address
pub fn allocate_user_stack(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<VirtAddr, &'static str> {
    let stack_start = VirtAddr::new(0x0000_7000_0000_0000);
    let stack_end = stack_start + USER_STACK_SIZE;

    let flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    // Map each page in the stack range
    for addr in (stack_start.as_u64()..stack_end.as_u64()).step_by(4096) {
        let page = Page::containing_address(VirtAddr::new(addr));

        // Allocate a fresh physical frame
        let frame = frame_allocator
            .allocate_frame()
            .ok_or("Failed to allocate frame for user stack")?;

        // Map the page to the frame
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| "Failed to map user stack")?
                .flush();
        }
    }

    // Return stack END (remember: stacks grow DOWN)
    Ok(stack_end)
}

//fn to take machine code and laod them into user memory

pub fn load_user_program(
    code: &[u8],
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<VirtAddr, &'static str> {
    let code_start = VirtAddr::new(0x0000_4000_0000_0000);
    let code_size = code.len();
    let pages_needed = (code_size + 4095) / 4096; // Round up

    let flags =
        PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE; // TODO: remove WRITABLE later for security

    // Allocate and map pages for the code
    for i in 0..pages_needed {
        let offset = (i * 4096) as u64;
        let addr = code_start + offset;
        let page = Page::containing_address(addr);

        let frame = frame_allocator
            .allocate_frame()
            .ok_or("Failed to allocate frame for user code")?;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| "Failed to map user code")?
                .flush();
        }
    }

    // Copy code to the mapped user memory
    unsafe {
        let dest = code_start.as_u64() as *mut u8;
        core::ptr::copy_nonoverlapping(code.as_ptr(), dest, code_size);
    }

    Ok(code_start)
}
