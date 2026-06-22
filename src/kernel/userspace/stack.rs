use x86_64::VirtAddr;
use x86_64::structures::paging::{Mapper, Size4KiB, PageTableFlags, FrameAllocator, Page};

const USER_STACK_SIZE: usize = 4096 * 20;

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
