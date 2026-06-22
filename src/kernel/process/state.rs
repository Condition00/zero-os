#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Zombie,
}
