use core::sync::atomic::{AtomicUsize, Ordering};

static NEXT_PID: AtomicUsize = AtomicUsize::new(1);


pub fn alloc_pid() -> usize {
    NEXT_PID.fetch_add(1, Ordering::Relaxed)
}
