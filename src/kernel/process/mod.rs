use alloc::vec::Vec;
use spin::Mutex;

pub mod context;
pub mod process;
pub mod state;
pub mod pid;

pub static PROCESS_TABLE: Mutex<Vec<Process>> = Mutex::new(Vec::new());

pub fn add_process(process: Process) {
	PROCESS_TABLE.lock().push(process);
}

pub use context::*;
pub use pid::*;
pub use process::Process;
pub use state::*;
