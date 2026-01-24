use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

const KERNEL_STACK_SIZE: usize = 4096 * 5;

#[repr(align(16))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

//one kernel stack for ring3 -> ring0
static mut KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();


        // Ring 0 privilige stack
        let start_stack = VirtAddr::from_ptr(unsafe {
          &raw const KERNEL_STACK.data
        });
        let stack_end = start_stack + KERNEL_STACK_SIZE;
        tss.privilege_stack_table[0] = stack_end;

        //double fault IST
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kernel_code_selector,
                tss_selector,
                user_data_selector,
                user_code_selector,
            },
        )
    };
}

pub struct Selectors {
    kernel_code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernel_code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub fn selectors() -> &'static Selectors {
    &GDT.1
}

//testing

pub fn test_user_segments() {
    let selectors = selectors();
    crate::println!("User code selector: {:?}", selectors.user_code_selector);
    crate::println!("User Data selector: {:?}", selectors.user_data_selector);
}
