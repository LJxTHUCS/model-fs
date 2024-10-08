use crate::fs::{FileDescriptor, FileSystem};
use km_checker::model_command;
use km_command::fs::{FileKind, OpenFlags, UnlinkatFlags};
use std::cell::RefCell;
use std::rc::Rc;

model_command!(km_command::fs, Chdir, FileSystem, {
    (|| state!().chdir(get!(path).clone().try_into()?))().map_or_else(|e| e.into(), |_| 0)
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
        state!().alloc_fd(Rc::new(RefCell::new(FileDescriptor::new_perm(
            path,
            get!(flags),
        ))))
    })()
    .map_or_else(|e| e.into(), |fd| fd)
});

model_command!(km_command::fs, Close, FileSystem, {
    (|| state!().free_fd(get!(fd)))().map_or_else(|e| e.into(), |_| 0)
});

model_command!(km_command::fs, Mkdirat, FileSystem, {
    (|| {
        let path = state!().parse_path(get!(dirfd), get!(path).clone())?;
        state!().create(path, FileKind::Directory, get!(mode))
    })()
    .map_or_else(|e| e.into(), |_| 0)
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
        let rmdir = get!(flags).contains(UnlinkatFlags::REMOVEDIR);
        // Link file
        state!().unlink(&path, rmdir)
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

// Constant FS commands.
//
// These commands don't change the state of the file system. They
// are not generated by commander, only used by test port to get
// the state of the file system.

model_command!(km_command::fs, Fstat, FileSystem, { 0 });

model_command!(km_command::fs, Getdents, FileSystem, { 0 });

model_command!(km_command::fs, Getcwd, FileSystem, { 0 });

model_command!(km_command, Nop, FileSystem, { 0 });
