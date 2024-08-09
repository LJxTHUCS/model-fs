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
    /// Invalid path.
    InvalidPath,
    /// Directory is not empty.
    DirectoryNotEmpty,
}

impl Into<isize> for FsError {
    fn into(self) -> isize {
        // TODO
        -1
    }
}
