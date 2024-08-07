use crate::inode::Inode;
use crate::path::AbsPath;
use crate::{error::FsError, inode::FileKind};
use km_checker::AbstractState;
use km_command::fs::{FileMode, OpenFlags};
use multi_key_map::MultiKeyMap;
use std::sync::Arc;

/// File descriptor table entry.
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: AbsPath,
    pub flags: OpenFlags,
}

/// File descriptor table size.
pub const FD_TABLE_SIZE: usize = 256;

/// Special file descriptor representing the current working directory.
pub const FDCWD: isize = -100;

/// Abstract state of the file system.
#[derive(Debug)]
pub struct FileSystem {
    /// Current working directory.
    pub cwd: AbsPath,
    /// User ID.
    pub uid: u32,
    /// Group ID.
    pub gid: u32,
    /// Inodes.
    pub inodes: MultiKeyMap<AbsPath, Inode>,
    /// File descriptor table.
    pub fd_table: [Option<Arc<FileDescriptor>>; FD_TABLE_SIZE],
}

impl AbstractState for FileSystem {
    fn matches(&self, other: &Self) -> bool {
        self.cwd == other.cwd
            && self.uid == other.uid
            && self.gid == other.gid
            && self.inodes == other.inodes
    }
    fn update(&mut self, other: &Self) {
        self.cwd = other.cwd.clone();
        self.uid = other.uid;
        self.gid = other.gid;
        self.inodes = other.inodes.clone();
    }
}

impl FileSystem {
    /// Get root inode.
    pub fn root(&self) -> &Inode {
        self.inodes.get(&AbsPath::root()).unwrap()
    }

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

    /// Lookup the inode by path.
    pub fn lookup(&self, path: &AbsPath) -> Result<Inode, FsError> {
        self.inodes.get(path).cloned().ok_or(FsError::NotFound)
    }

    /// Create a file by path.
    pub fn create_file(&mut self, path: &AbsPath, mode: FileMode) -> Result<(), FsError> {
        if self.inodes.get(path).is_some() {
            return Err(FsError::AlreadyExists);
        }
        let inode = Inode::new_file(mode, self.uid, self.gid);
        self.inodes.insert(path.clone(), inode);
        Ok(())
    }

    /// Create a directory by path.
    pub fn create_dir(&mut self, path: &AbsPath, mode: FileMode) -> Result<(), FsError> {
        if self.inodes.get(path).is_some() {
            return Err(FsError::AlreadyExists);
        }
        let inode = Inode::new_dir(mode, self.uid, self.gid);
        self.inodes.insert(path.clone(), inode);
        // Alias "."
        self.inodes.alias(path, path.join("."));
        Ok(())
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
            Ok(AbsPath::from_abs_command_path(path))
        } else {
            if dirfd == FDCWD {
                Ok(self.cwd.from_rel_command_path(path))
            } else {
                let fd = self.fd_table[dirfd as usize]
                    .as_ref()
                    .ok_or(FsError::NotOpened)?;
                Ok(fd.path.from_rel_command_path(path))
            }
        }
    }
}
