use km_command::linux_err;

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
        match self {
            FsError::NotFound => linux_err!(ENOENT),
            FsError::PermissionDenied => linux_err!(EACCES),
            FsError::AlreadyExists => linux_err!(EEXIST),
            FsError::IsDirectory => linux_err!(EISDIR),
            FsError::NotDirectory => linux_err!(ENOTDIR),
            FsError::NotOpened => linux_err!(EBADFD),
            FsError::NoAvailableFd => linux_err!(EBADFD),
            FsError::InvalidPath => linux_err!(EINVAL),
            FsError::DirectoryNotEmpty => linux_err!(ENOTEMPTY),
        }
    }
}
