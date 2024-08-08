use km_checker::AbstractState;
use km_command::fs::FileMode;

/// File kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    File,
    Directory,
}

impl AbstractState for FileKind {
    fn matches(&self, other: &Self) -> bool {
        self == other
    }
    fn update(&mut self, other: &Self) {
        *self = *other;
    }
}

/// File system I-node type, regular file or directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inode {
    /// File model.
    pub mode: FileMode,
    /// User ID.
    pub uid: u32,
    /// Group ID.
    pub gid: u32,
    /// Link count.
    pub nlink: usize,
    /// File kind.
    pub kind: FileKind,
}

impl Inode {
    /// Create a new inode.
    /// 
    /// Set link count to 1 for regular file, 2 for directory.
    pub fn new(mode: FileMode, uid: u32, gid: u32, kind: FileKind) -> Self {
        let nlink = if kind == FileKind::Directory { 2 } else { 1 };
        Self {
            mode,
            uid,
            gid,
            nlink,
            kind,
        }
    }
    /// Check if the file is a directory.
    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }
    /// Check if the file is a regular file.
    pub fn is_file(&self) -> bool {
        self.kind == FileKind::File
    }
}
