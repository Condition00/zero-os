pub mod ramfs;
pub mod vfs;

pub use ramfs::RamFs;
pub use vfs::{FileSystem, FileType, INode, OpenOptions, VFS};

use alloc::sync::Arc;
use spin::Mutex;

static ROOT_FS: Mutex<Option<Arc<dyn FileSystem + Send + Sync>>> = Mutex::new(None);

pub fn init() {
    let ramfs = Arc::new(RamFs::new());
    *ROOT_FS.lock() = Some(ramfs.clone());

    if let Some(fs) = ROOT_FS.lock().as_ref() {
        let _ = fs.create_dir("/home");
        let _ = fs.create_dir("/tmp");
        let _ = fs.create_dir("/bin");
    }
}

pub fn root() -> Option<Arc<dyn FileSystem + Send + Sync>> {
    ROOT_FS.lock().as_ref().map(Arc::clone)
}
