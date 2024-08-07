use crate::path::AbsPath;
use km_checker::{
    state::{Ignored, ValueSet},
    AbstractState,
};
use km_command::fs::{FileMode, OpenFlags};
use std::sync::Arc;

/// File kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// File status, the abstract state of a file.
#[derive(Debug, Clone, AbstractState)]
pub struct FileStatus {
    pub mode: u32,
    pub nlink: usize,
    pub uid: usize,
    pub gid: usize,
}

/// File system I-node type, regular file or directory.
#[derive(Debug, Clone, AbstractState)]
pub struct Inode {
    /// Relative path under direct parent directory.
    pub name: String,
    /// File kind.
    pub kind: FileKind,
    /// File status.
    pub status: FileStatus,
    /// `Some(entries)` for directory, `None` for regular file.
    pub entries: Option<ValueSet<Inode>>,
}

impl Inode {
    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }
    pub fn is_file(&self) -> bool {
        self.kind == FileKind::File
    }
}

/// File descriptor table entry.
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: AbsPath,
    pub oflags: OpenFlags,
}

pub const FD_TABLE_SIZE: usize = 256;

pub const FDCWD: isize = -100;

/// Abstract state of the file system.
#[derive(Debug, AbstractState)]
pub struct FileSystem {
    /// Root directory.
    pub root: Inode,
    /// Current working directory.
    pub cwd: AbsPath,
    /// File descriptor table.
    pub fd_table: Ignored<[Option<Arc<FileDescriptor>>; FD_TABLE_SIZE]>,
}

/// File system error.
#[derive(Debug, Clone, Copy)]
pub enum FsError {
    /// File not found.
    NotFound,
    /// Permission denied.
    PermissionDenied,
    /// File already exists.
    AlreadyExists,
    /// File is a directory.
    IsDirectory,
    /// File is not a directory.
    NotDirectory,
    /// File not opened.
    NotOpened,
    /// No available file descriptor.
    NoAvailableFd,
}

impl Into<isize> for FsError {
    fn into(self) -> isize {
        -1
    }
}

impl FileSystem {
    /// Get file descriptor by fd.
    pub fn get_fd(&self, fd: isize) -> Result<Arc<FileDescriptor>, FsError> {
        if fd < 0 || fd as usize >= self.fd_table.len() {
            return Err(FsError::NotOpened);
        } else {
            self.fd_table[fd as usize].clone().ok_or(FsError::NotOpened)
        }
    }

    /// Find the lowest available posistion in the fd table and write `fd` into it.
    pub fn alloc_fd(&mut self, fd: Arc<FileDescriptor>) -> Result<isize, FsError> {
        for (i, e) in self.fd_table.iter_mut().enumerate() {
            if e.is_none() {
                *e = Some(fd);
                return Ok(i as isize);
            }
        }
        Err(FsError::NoAvailableFd)
    }

    /// Free the file descriptor.
    pub fn free_fd(&mut self, fd: isize) -> Result<(), FsError> {
        if self.get_fd(fd).is_ok() {
            self.fd_table[fd as usize] = None;
            Ok(())
        } else {
            Err(FsError::NotOpened)
        }
    }

    /// Change the current working directory.
    pub fn chdir(&mut self, path: &AbsPath) -> Result<(), FsError> {
        let inode = self.lookup(path)?;
        if inode.is_dir() {
            self.cwd = path.clone();
            Ok(())
        } else {
            Err(FsError::NotDirectory)
        }
    }

    /// Parse `path` argument of fs syscall. For `openat`, `linkat`, `mkdirat` ...
    ///
    /// The dirfd argument is used in conjunction with the pathname
    /// argument as follows:
    ///
    /// -  If the pathname given in pathname is absolute, then dirfd is
    ///       ignored.
    /// -  If the pathname given in pathname is relative and dirfd is the
    ///       special value AT_FDCWD, then pathname is interpreted relative
    ///       to the current working directory of the calling process (like
    ///       open()).
    /// -  If the pathname given in pathname is relative, then it is
    ///       interpreted relative to the directory referred to by the file
    ///       descriptor dirfd (rather than relative to the current working
    ///       directory of the calling process, as is done by open() for a
    ///       relative pathname).  In this case, dirfd must be a directory
    ///       that was opened for reading (O_RDONLY) or using the O_PATH
    ///       flag.
    ///
    /// Ref: https://man7.org/linux/man-pages/man2/open.2.html
    pub fn parse_path(
        &self,
        dirfd: isize,
        path: &km_command::fs::Path,
    ) -> Result<AbsPath, FsError> {
        if path.absolute() {
            Ok(AbsPath::from_abs(path))
        } else {
            if dirfd == FDCWD {
                Ok(self.cwd.concat_rel(path))
            } else {
                let fd = self.fd_table[dirfd as usize]
                    .as_ref()
                    .ok_or(FsError::NotOpened)?;
                Ok(fd.path.concat_rel(path))
            }
        }
    }

    /// Lookup the inode by path.
    pub fn lookup(&self, path: &AbsPath) -> Result<&Inode, FsError> {
        let mut cur = &self.root;
        for name in path.iter() {
            if let Some(entries) = &cur.entries {
                cur = entries
                    .iter()
                    .find(|e| e.name == *name)
                    .ok_or(FsError::NotFound)?;
            } else {
                return Err(FsError::NotDirectory);
            }
        }
        Ok(&cur)
    }

    /// Create an inode by path.
    pub fn create(
        &mut self,
        path: &AbsPath,
        kind: FileKind,
        mode: FileMode,
    ) -> Result<(), FsError> {
        todo!()
    }
}
