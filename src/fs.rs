use crate::inode::Inode;
use crate::path::{AbsPath, RelPath};
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
    /// Inodes. An inode may have multiple absolutes paths (hard links).
    /// Each key is corresponding to an absolute path.
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
    /// Check if `path` exists.
    pub fn exists(&self, path: &AbsPath) -> Result<(), FsError> {
        if self.inodes.get(path).is_some() {
            Ok(())
        } else {
            Err(FsError::NotFound)
        }
    }

    /// Check if `path` is a valid directory.
    pub fn is_dir(&self, path: &AbsPath) -> Result<(), FsError> {
        match self.inodes.get(&path) {
            Some(p) => {
                if p.is_dir() {
                    Ok(())
                } else {
                    Err(FsError::NotDirectory)
                }
            }
            None => Err(FsError::NotFound),
        }
    }

    /// Lookup the inode by path.
    pub fn lookup(&self, path: &AbsPath) -> Result<Inode, FsError> {
        self.inodes.get(path).cloned().ok_or(FsError::NotFound)
    }

    /// Link an inode.
    pub fn link(&mut self, oldpath: &AbsPath, newpath: AbsPath) -> Result<(), FsError> {
        // Check if the old path exists.
        self.exists(oldpath)?;
        // Check if the new parent exists.
        self.is_dir(&newpath.parent().unwrap())?;
        // Link the inode.
        self.inodes.alias(oldpath, newpath);
        Ok(())
    }

    /// Unlink an inode.
    pub fn unlink(&mut self, path: &AbsPath) -> Result<(), FsError> {
        // Check if the path exists.
        self.exists(path)?;
        // Unlink the inode.
        self.inodes.remove_alias(path);
        Ok(())
    }

    /// Create an inode by path.
    pub fn create(&mut self, path: AbsPath, kind: FileKind, mode: FileMode) -> Result<(), FsError> {
        // Check if the file already exists.
        self.exists(&path)?;
        // Check if the parent directory exists.
        self.is_dir(&path.parent().unwrap())?;
        // Create the inode.
        let inode = Inode::new(mode, self.uid, self.gid, kind);
        self.inodes.insert(path.clone(), inode);
        // If `inode` is a directory, then create a `.` and `..` link.
        if kind == FileKind::Directory {
            self.link(&path, path.join(&RelPath::cur()))?;
            self.link(&path.parent().unwrap(), path.join(&RelPath::parent()))?;
        }
        Ok(())
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
    pub fn parse_path(&self, dirfd: isize, path: km_command::fs::Path) -> Result<AbsPath, FsError> {
        if path.absolute() {
            Ok(AbsPath::from(path))
        } else {
            if dirfd == FDCWD {
                Ok(self.cwd.join(&RelPath::from(path)))
            } else {
                let fd = self.fd_table[dirfd as usize]
                    .as_ref()
                    .ok_or(FsError::NotOpened)?;
                Ok(fd.path.join(&RelPath::from(path)))
            }
        }
    }
}
