use crate::error::FsError;
use crate::inode::Inode;
use crate::path::AbsPath;
use km_checker::AbstractState;
use km_command::fs::{FileKind, FileMode, OpenFlags, Path};
use multi_key_map::MultiKeyMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::usize;

/// File descriptor reference file type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FdRefType {
    /// An existing file, noted by an absolute path.
    Existing(AbsPath),
    /// A temporary file, noted by an index in the temporary inode list.
    Temporary(usize),
}

/// File descriptor table entry.
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    fref: FdRefType,
    flags: OpenFlags,
}

impl FileDescriptor {
    /// Create a file descriptor, which refers to an existing file.
    pub fn new_path(path: AbsPath, flags: OpenFlags) -> Self {
        Self {
            fref: FdRefType::Existing(path),
            flags,
        }
    }
    /// Create a file descriptor, which refers to a temporary file.
    pub fn new_tmp(idx: usize, flags: OpenFlags) -> Self {
        Self {
            fref: FdRefType::Temporary(idx),
            flags,
        }
    }
}

/// File descriptor table size.
pub const FD_TABLE_SIZE: usize = 256;

/// Special file descriptor representing the current working directory.
pub const FDCWD: isize = -100;

/// Abstract state of the file system.
#[derive(Clone)]
pub struct FileSystem {
    /// User ID.
    uid: u32,
    /// Group ID.
    gid: u32,
    /// Inodes. An inode may have multiple absolutes paths (hard links).
    /// Each key is corresponding to an absolute path.
    inodes: MultiKeyMap<AbsPath, Inode>,
    /// Current working directory.
    cwd: AbsPath,
    /// File descriptor table.
    fd_table: [Option<Rc<RefCell<FileDescriptor>>>; FD_TABLE_SIZE],
    /// Temporary inodes. Inodes deleted but still referenced by file descriptors
    /// will be stored here.
    tmp_inodes: HashMap<usize, Inode>,
    /// Next temporary inode index.
    tmp_idx: usize,
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

impl Debug for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("File System:\n")?;
        f.write_fmt(format_args!("  cwd: {:?}\n", self.cwd))?;
        f.write_fmt(format_args!("  uid: {}\n", self.uid))?;
        f.write_fmt(format_args!("  gid: {}\n", self.gid))?;
        f.write_str("Directory structure:\n")?;
        let mut paths = self.inodes.keys();
        paths.sort();
        for path in paths {
            f.write_fmt(format_args!(
                "{:?}\t {:?}\n",
                path,
                self.inodes.get(&path).unwrap()
            ))?;
        }
        f.write_str("<Not Checked> File descriptor table:\n")?;
        for (i, e) in self.fd_table.iter().enumerate() {
            if let Some(fd) = e {
                f.write_fmt(format_args!("[{}]\t {:?}\n", i, fd))?
            }
        }
        f.write_str("<Not Checked> Temporary inodes:\n")?;
        for (i, inode) in self.tmp_inodes.iter() {
            f.write_fmt(format_args!("[{}]\t {:?}\n", i, inode))?
        }
        Ok(())
    }
}

impl FileSystem {
    /// Create a file system with given inodes.
    pub fn new(inodes: MultiKeyMap<AbsPath, Inode>, cwd: AbsPath, uid: u32, gid: u32) -> Self {
        const NONE_FD: Option<Rc<RefCell<FileDescriptor>>> = None;
        Self {
            uid,
            gid,
            inodes,
            cwd,
            fd_table: [NONE_FD; FD_TABLE_SIZE],
            tmp_inodes: HashMap::new(),
            tmp_idx: 0,
        }
    }

    /// Create an empty file system, initializing the root directory.
    pub fn new_root(uid: u32, gid: u32) -> Self {
        // Set the current working directory to root.
        let cwd = AbsPath::root();
        // Initialize the file descriptor table.
        const NONE_FD: Option<Rc<RefCell<FileDescriptor>>> = None;
        let fd_table = [NONE_FD; FD_TABLE_SIZE];
        // Create fs.
        let mut fs = Self {
            uid,
            gid,
            inodes: MultiKeyMap::new(),
            cwd,
            fd_table,
            tmp_inodes: HashMap::new(),
            tmp_idx: 0,
        };
        // Initialize root directory. The `nlink` of the root directory is 2
        // ("." and ".."), which also matches the initialization of the inode.
        fs.inodes.insert(
            AbsPath::root(),
            Inode::new(FileMode::all(), uid, gid, FileKind::Directory),
        );
        fs
    }

    /// Open `stdin`, `stdout` and `stderr`.
    pub fn open_stdio(&mut self) {
        for i in 0..3 {
            self.fd_table[i] = Some(Rc::new(RefCell::new(FileDescriptor::new_tmp(
                usize::MAX,
                OpenFlags::empty(),
            ))))
        }
    }

    /// Check if `path` exists.
    pub fn exists(&self, path: &AbsPath) -> bool {
        self.inodes.contains_key(path)
    }

    /// Check if `path` exists and is a valid directory.
    pub fn is_dir(&self, path: &AbsPath) -> bool {
        self.inodes
            .get(&path)
            .map(|inode| inode.is_dir())
            .unwrap_or(false)
    }

    /// Check if `path` exists and is an empty directory.
    pub fn is_empty_dir(&self, path: &AbsPath) -> bool {
        self.is_dir(path) && self.inodes.keys().iter().all(|k| !path.is_ancestor(k))
    }

    /// Get all valid paths in the file system.
    pub fn paths(&self) -> Vec<AbsPath> {
        self.inodes.keys()
    }

    /// Lookup the inode by path.
    pub fn lookup(&self, path: &AbsPath) -> Result<Inode, FsError> {
        self.inodes.get(path).cloned().ok_or(FsError::NotFound)
    }

    /// Makr a new name for an inode.
    pub fn link(&mut self, oldpath: &AbsPath, newpath: AbsPath) -> Result<(), FsError> {
        if !self.exists(oldpath) {
            return Err(FsError::NotFound);
        }
        if self.is_dir(oldpath) {
            return Err(FsError::IsDirectory);
        }
        if self.exists(&newpath) {
            return Err(FsError::AlreadyExists);
        }
        if !self.exists(&newpath.parent().unwrap()) {
            return Err(FsError::NotFound);
        }
        if !self.is_dir(&newpath.parent().unwrap()) {
            return Err(FsError::NotDirectory);
        }
        // Link the inode.
        self.inodes.insert_alias(oldpath, newpath);
        self.increase_nlink(oldpath)
    }

    /// Delete a name and possibly the inode it refer to
    pub fn unlink(&mut self, path: &AbsPath, rmdir: bool) -> Result<(), FsError> {
        if path.is_root() {
            return Err(FsError::InvalidPath);
        }
        if !self.exists(path) {
            return Err(FsError::NotFound);
        }
        if self.is_dir(path) {
            if !rmdir {
                return Err(FsError::IsDirectory);
            }
            if !self.is_empty_dir(path) {
                return Err(FsError::DirectoryNotEmpty);
            }
            // If inode is a directory, update parent link count
            self.decrease_nlink(&path.parent().unwrap())?;
        } else {
            if rmdir {
                return Err(FsError::NotDirectory);
            }
        }
        // Unlink the inode.
        // Get all fds referring to the inode.
        let related_fds = self.all_fds_ref_same_inode(&FdRefType::Existing(path.clone()));
        let aliases = self.inodes.aliases(path).unwrap();
        if aliases.len() == 1 {
            // The inode will be removed. If there are fd pointing to it,
            // the inode will be collected in `tmp_inodes`.
            let inode = self.inodes.remove(path).unwrap();
            if !related_fds.is_empty() {
                // Some fds pointing to the inode, update their fref.
                for fd in related_fds {
                    self.fd_table[fd as usize]
                        .as_mut()
                        .unwrap()
                        .borrow_mut()
                        .fref = FdRefType::Temporary(self.tmp_idx);
                }
                // Collect the inode in `tmp_inodes`.
                self.tmp_inodes.insert(self.tmp_idx, inode);
                self.tmp_idx += 1;
            }
        } else {
            // The inode is still referenced by other paths, just remove the alias.
            self.inodes.remove_alias(path).unwrap();
            self.decrease_nlink(path)?;
            if !related_fds.is_empty() {
                // Some fds pointing to the inode, update their fref.
                let another_path = aliases
                    .into_iter()
                    .find(|p| self.inodes.contains_key(p))
                    .unwrap();
                for fd in related_fds {
                    self.fd_table[fd as usize]
                        .as_mut()
                        .unwrap()
                        .borrow_mut()
                        .fref = FdRefType::Existing(another_path.clone());
                }
            }
        }
        Ok(())
    }

    /// Create an inode by path.
    pub fn create(&mut self, path: AbsPath, kind: FileKind, mode: FileMode) -> Result<(), FsError> {
        if self.exists(&path) {
            return Err(FsError::AlreadyExists);
        }
        if !self.exists(&path.parent().unwrap()) {
            return Err(FsError::NotFound);
        }
        if !self.is_dir(&path.parent().unwrap()) {
            return Err(FsError::NotDirectory);
        }
        // Create the inode.
        let inode = Inode::new(mode, self.uid, self.gid, kind);
        self.inodes.insert(path.clone(), inode);
        // If `inode` is a directory, update parent link count
        if kind == FileKind::Directory {
            self.increase_nlink(&path.parent().unwrap())?;
        }
        Ok(())
    }

    /// Change the current working directory.
    pub fn chdir(&mut self, path: AbsPath) -> Result<(), FsError> {
        if !self.exists(&path) {
            return Err(FsError::NotFound);
        }
        if !self.is_dir(&path) {
            return Err(FsError::NotDirectory);
        }
        self.cwd = path;
        Ok(())
    }

    /// Get all allocated file descriptors.
    pub fn all_fds(&self) -> Vec<isize> {
        self.fd_table
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_some())
            .map(|(i, _)| i as isize)
            .collect()
    }

    /// Get file descriptor by fd.
    pub fn get_fd(&self, fd: isize) -> Result<Rc<RefCell<FileDescriptor>>, FsError> {
        if fd < 0 || fd as usize >= self.fd_table.len() {
            return Err(FsError::FdOutOfRange);
        } else {
            self.fd_table[fd as usize].clone().ok_or(FsError::NotOpened)
        }
    }

    /// Find the lowest available posistion in the fd table and write `fd` into it.
    pub fn alloc_fd(&mut self, fd: Rc<RefCell<FileDescriptor>>) -> Result<isize, FsError> {
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
            let fd = self.fd_table[fd as usize].take().unwrap();
            let fref = &fd.borrow().fref;
            if let FdRefType::Temporary(idx) = fref {
                // If the file descriptor refers to a temporary file, and
                // there is no other file descriptor referring to the same
                // inode, then remove the inode.
                let related_fds = self.all_fds_ref_same_inode(fref);
                if related_fds.len() == 1 {
                    self.tmp_inodes.remove(idx);
                }
            }
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
    pub fn parse_path(&self, dirfd: isize, path: Path) -> Result<AbsPath, FsError> {
        if path.absolute() {
            path.try_into()
        } else {
            if dirfd == FDCWD {
                Ok(self.cwd.join(&path.try_into()?)?)
            } else {
                let fd = self.get_fd(dirfd)?;
                let fref = &fd.borrow().fref;
                if let FdRefType::Existing(p) = fref {
                    if !self.exists(&p) {
                        return Err(FsError::NotFound);
                    }
                    if !self.is_dir(&p) {
                        return Err(FsError::NotDirectory);
                    }
                    Ok(p.join(&path.try_into()?)?)
                } else {
                    Err(FsError::NotFound)
                }
            }
        }
    }

    /// Increase link count of an inode
    fn increase_nlink(&mut self, path: &AbsPath) -> Result<(), FsError> {
        let inode = self.inodes.get_mut(path).ok_or(FsError::NotFound)?;
        inode.nlink += 1;
        Ok(())
    }

    /// Decrease link count of an inode
    fn decrease_nlink(&mut self, path: &AbsPath) -> Result<(), FsError> {
        let inode = self.inodes.get_mut(path).ok_or(FsError::NotFound)?;
        inode.nlink -= 1;
        Ok(())
    }

    /// Check if 2 frefs refer to the same inode.
    fn ref_same_inode(&self, a: &FdRefType, b: &FdRefType) -> bool {
        match (a, b) {
            (FdRefType::Existing(a), FdRefType::Existing(b)) => self.inodes.are_aliases(a, b),
            (FdRefType::Temporary(a), FdRefType::Temporary(b)) => a == b,
            _ => false,
        }
    }

    /// Get all file descriptors referring to the same inode as `fref`
    fn all_fds_ref_same_inode(&self, fref: &FdRefType) -> Vec<isize> {
        self.all_fds()
            .into_iter()
            .filter(|&fd| {
                self.ref_same_inode(
                    &self.fd_table[fd as usize].as_ref().unwrap().borrow().fref,
                    fref,
                )
            })
            .collect()
    }
}
