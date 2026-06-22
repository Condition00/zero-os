use x86_64::VirtAddr;

use super::{alloc_pid, Context, ProcessState};

#[derive(Debug)]
pub struct Process {
    pub pid: usize,

    pub context: Context,

    pub state: ProcessState,
}

impl Process {
    pub fn new(entry: VirtAddr, stack: VirtAddr) -> Self {
        Self {
            pid: alloc_pid(),
            state: ProcessState::Ready,
            context: Context {
                rip: entry.as_u64(),
                rsp: stack.as_u64(),
                rbx: 0,
                rbp: 0,
                r12: 0,
                r13: 0,
                r14: 0,
                r15: 0,
            },
        }
    }
}
