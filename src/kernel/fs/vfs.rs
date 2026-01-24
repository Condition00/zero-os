use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug)]
pub enum FsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    InvalidPath,
    NoSpace,
    PermissionDenied,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FsError::NotFound => write!(f, "No such file or directory"),
            FsError::AlreadyExists => write!(f, "File or directory already exists"),
            FsError::NotADirectory => write!(f, "Not a directory"),
            FsError::NotAFile => write!(f, "Not a file"),
            FsError::InvalidPath => write!(f, "Invalid path"),
            FsError::NoSpace => write!(f, "No space left"),
            FsError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

pub type FsResult<T> = Result<T, FsError>;

#[derive(Clone)]
pub struct INode {
    pub name: String,
    pub file_type: FileType,
    pub size: usize,
}

#[derive(Default)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub truncate: bool,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }

    pub fn write(mut self, write: bool) -> Self {
        self.write = write;
        self
    }

    pub fn create(mut self, create: bool) -> Self {
        self.create = create;
        self
    }

    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }
}

pub trait FileSystem {
    fn create_file(&self, path: &str) -> FsResult<()>;
    fn create_dir(&self, path: &str) -> FsResult<()>;
    fn remove(&self, path: &str) -> FsResult<()>;
    fn read_file(&self, path: &str) -> FsResult<Vec<u8>>;
    fn write_file(&self, path: &str, data: &[u8]) -> FsResult<()>;
    fn list_dir(&self, path: &str) -> FsResult<Vec<INode>>;
    fn stat(&self, path: &str) -> FsResult<INode>;
    fn exists(&self, path: &str) -> bool;
}

pub struct VFS;

impl VFS {
    pub fn normalize_path(path: &str) -> String {
        if path.is_empty() || !path.starts_with('/') {
            return "/".into();
        }
        
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return "/".into();
        }
        
        let mut normalized = String::from("/");
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                normalized.push('/');
            }
            normalized.push_str(part);
        }
        normalized
    }

    pub fn parent_path(path: &str) -> Option<String> {
        if path == "/" {
            return None;
        }
        
        let normalized = Self::normalize_path(path);
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Some("/".into());
        }
        
        if parts.len() == 1 {
            return Some("/".into());
        }
        
        let mut parent = String::from("/");
        for (i, part) in parts[..parts.len() - 1].iter().enumerate() {
            if i > 0 {
                parent.push('/');
            }
            parent.push_str(part);
        }
        Some(parent)
    }

    pub fn filename(path: &str) -> Option<String> {
        let normalized = Self::normalize_path(path);
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        parts.last().map(|s| String::from(*s))
    }
}
