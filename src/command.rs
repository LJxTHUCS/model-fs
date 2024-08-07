use crate::{
    fs::{FileDescriptor, FileSystem, FDCWD},
    inode::FileKind,
};
use km_checker::model_command;
use km_command::fs::OpenFlags;
use std::sync::Arc;

model_command!(km_command::fs, Chdir, FileSystem, {
    (|| {
        let path = state!().parse_path(FDCWD, get!(path).clone())?;
        state!().chdir(&path)
    })()
    .map_or_else(|e| e.into(), |_| 0)
});

model_command!(km_command::fs, Openat, FileSystem, {
    (|| {
        let path = state!().parse_path(get!(dirfd), get!(path).clone())?;
        // Check file exists
        if let Err(e) = state!().lookup(&path) {
            if !get!(flags).contains(OpenFlags::CREAT) {
                return Err(e);
            } else {
                // Create file
                state!().create(path.clone(), FileKind::File, get!(mode))?;
            }
        }
        // Find available file descriptor
        state!().alloc_fd(Arc::new(FileDescriptor {
            path,
            flags: get!(flags),
        }))
    })()
    .map_or_else(|e| e.into(), |fd| fd)
});

model_command!(km_command::fs, Close, FileSystem, {
    (|| state!().free_fd(get!(fd)))().map_or_else(|e| e.into(), |_| 0)
});

model_command!(km_command::fs, Linkat, FileSystem, {
    (|| {
        // Parse paths
        let old_path = state!().parse_path(get!(olddirfd), get!(oldpath).clone())?;
        let new_path = state!().parse_path(get!(newdirfd), get!(newpath).clone())?;
        // Link file
        state!().link(&old_path, new_path)
    })()
    .map_or_else(|e| e.into(), |_| 0)
});

model_command!(km_command::fs, Unlinkat, FileSystem, {
    (|| {
        // Parse paths
        let path = state!().parse_path(get!(dirfd), get!(path).clone())?;
        // Link file
        state!().unlink(&path)
    })()
    .map_or_else(|e| e.into(), |_| 0)
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
