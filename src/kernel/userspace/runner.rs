use crate::kernel::process::{Process, ProcessState};

use super::ring3::jump_to_userspace;

pub fn run_process(mut process: Process) -> ! {
    process.state = ProcessState::Running;

    jump_to_userspace(
        x86_64::VirtAddr::new(process.context.rip),
        x86_64::VirtAddr::new(process.context.rsp),
    )
}
