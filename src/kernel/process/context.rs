#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub rip: u64,
    pub rsp: u64,

    pub rbx: u64,
    pub rbp: u64,

    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}