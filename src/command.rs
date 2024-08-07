use crate::fs::{FileDescriptor, FileKind, FileSystem, FsError, FDCWD};
use crate::path::AbsPath;
use km_checker::model_command;
use km_command::fs::OpenFlags;
use std::sync::Arc;

model_command!(km_command::fs, Chdir, FileSystem, {
    (|| {
        let path = state!().parse_path(FDCWD, &get!(path))?;
        state!().chdir(&path)
    })()
    .map_or_else(|e| e.into(), |_| 0)
});

model_command!(km_command::fs, Openat, FileSystem, {
    (|| {
        let path = state!().parse_path(get!(dirfd), &get!(path))?;
        // Check file exists
        if let Err(e) = state!().lookup(&path) {
            if !get!(flags).contains(OpenFlags::CREAT) {
                return Err(e);
            } else {
                // Create file
                state!().create(&path, FileKind::File, get!(mode))?;
            }
        }
        // Find available file descriptor
        state!().alloc_fd(Arc::new(FileDescriptor {
            path,
            oflags: get!(flags),
        }))
    })()
    .map_or_else(|e| e.into(), |fd| fd)
});

model_command!(km_command::fs, Close, FileSystem, {
    (|| state!().free_fd(get!(fd)))().map_or_else(|e| e.into(), |_| 0)
});

model_command!(km_command::fs, Dup, FileSystem, {
    (|| {
        let oldfd = state!().get_fd(get!(oldfd))?;
        // Find available file descriptor
        state!().alloc_fd(oldfd)
    })()
    .map_or_else(|e| e.into(), |fd| fd)
});

model_command!(km_command::fs, Getdents, FileSystem, { 0 });
