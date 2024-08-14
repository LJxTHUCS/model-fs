use km_command::fs::{FileKind, FileMode, FileStat};

/// File system I-node type, regular file or directory.
#[derive(Debug, Clone)]
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

#[cfg(feature = "fat")]
impl PartialEq for Inode {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid && self.gid == other.gid && self.kind == other.kind
    }
}

#[cfg(not(feature = "fat"))]
impl PartialEq for Inode {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode
            && self.uid == other.uid
            && self.gid == other.gid
            && self.nlink == other.nlink
            && self.kind == other.kind
    }
}

impl Eq for Inode {}

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
    /// Create an inode file file stat.
    pub fn from_stat(stat: &FileStat) -> Self {
        Self {
            mode: stat.mode,
            uid: stat.uid,
            gid: stat.gid,
            nlink: stat.nlink,
            kind: stat.kind,
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
