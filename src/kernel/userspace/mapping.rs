use x86_64::VirtAddr;
use x86_64::structures::paging::{Mapper, Size4KiB, PageTableFlags, FrameAllocator, Page};

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
