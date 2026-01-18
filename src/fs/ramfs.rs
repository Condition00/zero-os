use super::vfs::{FileSystem, FileType, FsError, FsResult, INode, VFS};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Clone)]
struct FileData {
    content: Vec<u8>,
}

#[derive(Clone)]
struct DirData {
    entries: Vec<String>,
}

#[derive(Clone)]
enum NodeData {
    File(FileData),
    Directory(DirData),
}

#[derive(Clone)]
struct Node {
    name: String,
    file_type: FileType,
    data: NodeData,
}

impl Node {
    fn new_file(name: String) -> Self {
        Node {
            name,
            file_type: FileType::File,
            data: NodeData::File(FileData {
                content: Vec::new(),
            }),
        }
    }

    fn new_dir(name: String) -> Self {
        Node {
            name,
            file_type: FileType::Directory,
            data: NodeData::Directory(DirData {
                entries: Vec::new(),
            }),
        }
    }

    fn size(&self) -> usize {
        match &self.data {
            NodeData::File(file) => file.content.len(),
            NodeData::Directory(dir) => dir.entries.len(),
        }
    }
}

pub struct RamFs {
    nodes: Mutex<BTreeMap<String, Node>>,
}

impl RamFs {
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        
        // Create root directory
        nodes.insert("/".into(), Node::new_dir("/".into()));
        
        RamFs {
            nodes: Mutex::new(nodes),
        }
    }

    fn get_node(&self, path: &str) -> FsResult<Node> {
        let normalized = VFS::normalize_path(path);
        let nodes = self.nodes.lock();
        nodes.get(&normalized).cloned().ok_or(FsError::NotFound)
    }

    fn add_to_parent_dir(&self, path: &str, name: &str) -> FsResult<()> {
        if let Some(parent_path) = VFS::parent_path(path) {
            let mut nodes = self.nodes.lock();
            if let Some(parent) = nodes.get_mut(&parent_path) {
                if let NodeData::Directory(ref mut dir) = parent.data {
                    if !dir.entries.contains(&name.into()) {
                        dir.entries.push(name.into());
                    }
                    return Ok(());
                }
                return Err(FsError::NotADirectory);
            }
            return Err(FsError::NotFound);
        }
        Ok(())
    }

    fn remove_from_parent_dir(&self, path: &str, name: &str) -> FsResult<()> {
        if let Some(parent_path) = VFS::parent_path(path) {
            let mut nodes = self.nodes.lock();
            if let Some(parent) = nodes.get_mut(&parent_path) {
                if let NodeData::Directory(ref mut dir) = parent.data {
                    dir.entries.retain(|e| e != name);
                    return Ok(());
                }
                return Err(FsError::NotADirectory);
            }
            return Err(FsError::NotFound);
        }
        Ok(())
    }
}

impl FileSystem for RamFs {
    fn create_file(&self, path: &str) -> FsResult<()> {
        let normalized = VFS::normalize_path(path);
        
        // Check if already exists
        if self.exists(&normalized) {
            return Err(FsError::AlreadyExists);
        }

        // Check if parent directory exists
        if let Some(parent_path) = VFS::parent_path(&normalized) {
            let parent = self.get_node(&parent_path)?;
            if parent.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
        }

        let filename = VFS::filename(&normalized).ok_or(FsError::InvalidPath)?;
        let node = Node::new_file(filename.clone());
        
        self.nodes.lock().insert(normalized.clone(), node);
        self.add_to_parent_dir(&normalized, &filename)?;
        
        Ok(())
    }

    fn create_dir(&self, path: &str) -> FsResult<()> {
        let normalized = VFS::normalize_path(path);
        
        // Check if already exists
        if self.exists(&normalized) {
            return Err(FsError::AlreadyExists);
        }

        // Check if parent directory exists
        if let Some(parent_path) = VFS::parent_path(&normalized) {
            let parent = self.get_node(&parent_path)?;
            if parent.file_type != FileType::Directory {
                return Err(FsError::NotADirectory);
            }
        }

        let dirname = VFS::filename(&normalized).ok_or(FsError::InvalidPath)?;
        let node = Node::new_dir(dirname.clone());
        
        self.nodes.lock().insert(normalized.clone(), node);
        self.add_to_parent_dir(&normalized, &dirname)?;
        
        Ok(())
    }

    fn remove(&self, path: &str) -> FsResult<()> {
        let normalized = VFS::normalize_path(path);
        
        if normalized == "/" {
            return Err(FsError::PermissionDenied);
        }

        let node = self.get_node(&normalized)?;
        
        // Check if directory is empty
        if node.file_type == FileType::Directory {
            if let NodeData::Directory(dir) = &node.data {
                if !dir.entries.is_empty() {
                    return Err(FsError::PermissionDenied);
                }
            }
        }

        let filename = VFS::filename(&normalized).ok_or(FsError::InvalidPath)?;
        self.remove_from_parent_dir(&normalized, &filename)?;
        self.nodes.lock().remove(&normalized);
        
        Ok(())
    }

    fn read_file(&self, path: &str) -> FsResult<Vec<u8>> {
        let normalized = VFS::normalize_path(path);
        let node = self.get_node(&normalized)?;
        
        match node.data {
            NodeData::File(file) => Ok(file.content.clone()),
            NodeData::Directory(_) => Err(FsError::NotAFile),
        }
    }

    fn write_file(&self, path: &str, data: &[u8]) -> FsResult<()> {
        let normalized = VFS::normalize_path(path);
        let mut nodes = self.nodes.lock();
        
        if let Some(node) = nodes.get_mut(&normalized) {
            match &mut node.data {
                NodeData::File(file) => {
                    file.content = data.to_vec();
                    Ok(())
                }
                NodeData::Directory(_) => Err(FsError::NotAFile),
            }
        } else {
            Err(FsError::NotFound)
        }
    }

    fn list_dir(&self, path: &str) -> FsResult<Vec<INode>> {
        let normalized = VFS::normalize_path(path);
        let node = self.get_node(&normalized)?;
        
        match node.data {
            NodeData::Directory(dir) => {
                let nodes = self.nodes.lock();
                let mut result = Vec::new();
                
                for entry in dir.entries.iter() {
                    let child_path = if normalized == "/" {
                        alloc::format!("/{}", entry)
                    } else {
                        alloc::format!("{}/{}", normalized, entry)
                    };
                    
                    if let Some(child_node) = nodes.get(&child_path) {
                        result.push(INode {
                            name: entry.clone(),
                            file_type: child_node.file_type,
                            size: child_node.size(),
                        });
                    }
                }
                
                Ok(result)
            }
            NodeData::File(_) => Err(FsError::NotADirectory),
        }
    }

    fn stat(&self, path: &str) -> FsResult<INode> {
        let normalized = VFS::normalize_path(path);
        let node = self.get_node(&normalized)?;
        
        let name = node.name.clone();
        let file_type = node.file_type;
        let size = node.size();
        
        Ok(INode {
            name,
            file_type,
            size,
        })
    }

    fn exists(&self, path: &str) -> bool {
        let normalized = VFS::normalize_path(path);
        self.nodes.lock().contains_key(&normalized)
    }
}
