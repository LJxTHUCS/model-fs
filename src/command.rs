use crate::fs::{FileDescriptor, FileKind, FileSystem, FsError};
use crate::path::AbsPath;
use km_checker::model_command;
use km_command::fs::OpenFlags;
use km_command::linux_err;

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
        state!().alloc_fd(FileDescriptor {
            path,
            oflags: get!(flags),
        })
    })()
    .map_or(-1, |e| e)
});

model_command!(km_command::fs, Dup, FileSystem, {
    (|| {
        let oldfd = state!().get_fd(get!(oldfd))?;
        // Find available file descriptor
        state!().alloc_fd(FileDescriptor {
            path: oldfd.path.clone(),
            oflags: oldfd.oflags,
        })
    })()
    .map_or(-1, |e| e)
});

model_command!(km_command::fs, Getdents, FileSystem, { -1 });
