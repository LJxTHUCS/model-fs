use crate::path::AbsPath;
use bitflags::bitflags;
use km_checker::{
    state::{Ignored, ValueSet},
    AbstractState,
};
use km_command::fs::OpenFlags;

bitflags! {
    /// File permission mode.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FileMode: u32 {
        /// User readable.
        const USER_READ = 0o400;
        /// User writable.
        const USER_WRITE = 0o200;
        /// User executable.
        const USER_EXEC = 0o100;
        /// Group readable.
        const GROUP_READ = 0o040;
        /// Group writable.
        const GROUP_WRITE = 0o020;
        /// Group executable.
        const GROUP_EXEC = 0o010;
        /// Other readable.
        const OTHER_READ = 0o004;
        /// Other writable.
        const OTHER_WRITE = 0o002;
        /// Other executable.
        const OTHER_EXEC = 0o001;
    }
}

impl AbstractState for FileMode {
    fn matches(&self, other: &Self) -> bool {
        self == other
    }
    fn update(&mut self, other: &Self) {
        *self = *other;
    }
}

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
    pub mode: FileMode,
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

/// File descriptor table entry.
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: AbsPath,
    pub oflags: OpenFlags,
}

pub const FD_TABLE_SIZE: usize = 256;

pub const AT_FDCWD: isize = -100;

/// File system control info.
#[derive(Debug)]
pub struct FsControlBlock {
    pub fd_table: [Option<FileDescriptor>; FD_TABLE_SIZE],
}

/// Abstract state of the file system.
#[derive(Debug, AbstractState)]
pub struct FileSystem {
    /// Root directory.
    pub root: Inode,
    /// Current working directory.
    pub cwd: AbsPath,
    /// Control info.
    pub control: Ignored<FsControlBlock>,
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
}

impl FileSystem {
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
            if dirfd == AT_FDCWD {
                Ok(self.cwd.concat_rel(path))
            } else {
                let fd = self.control.fd_table[dirfd as usize]
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
