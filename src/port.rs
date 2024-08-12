use crate::{
    command::{
        Close as ModelClose, Fstat as ModelFstat, Getcwd as ModelGetcwd, Getdents as ModelGetdents,
        Openat as ModelOpenat,
    },
    inode::Inode,
    path::AbsPath,
    FileSystem,
};
use core::str;
use km_checker::{
    Command, CommandChannel, Error, MemCommandChannel, QemuMem, StateChannel, TestPort,
};
use km_command::fs::{
    Close, DirEntry, FileKind, FileMode, FileStat, Fstat, Getcwd, Getdents, OpenFlags, Openat,
    Path, MAX_PATH_LEN,
};
use multi_key_map::MultiKeyMap;
use std::{collections::HashMap, mem::size_of, str::FromStr};

/// Execution step of `FsTestPort`.
enum Step {
    /// Opening an inode.
    Open,
    /// Reading directory entries.
    Getdents,
    /// Reading inode metadata.
    Fstat,
    /// Closing an inode.
    Close,
    /// Get current working directory.
    Getcwd,
}

/// Test port to communicate with target kernel.
///
/// - Send file system command to target kernel and receive return value.
/// - Get target file system state by DFS traversal.
///
/// `FsTestPort` uses constant FS commands to get target file system state.
///
/// - `getdents` to get directory structure.
/// - `fstat` to get inode metadata.
pub struct FsTestPort {
    /// Command channel to send command to target kernel.
    cmd_chan: MemCommandChannel<QemuMem, QemuMem>,
    /// Current working directory.
    cwd: AbsPath,
    /// Fs directory structure.
    fs: MultiKeyMap<AbsPath, Inode>,
    /// DFS stack of opened inodes, (fd, name).
    stack: Vec<(isize, String)>,
    /// Seen inode_id set, need to resolve hard links.
    seen_inodes: HashMap<usize, AbsPath>,
    /// Execution step.
    step: Step,
}

impl FsTestPort {
    /// Get the stack top inode.
    fn top(&self) -> &(isize, String) {
        self.stack.last().unwrap()
    }

    /// Get the mutable reference to the stack top inode.
    fn top_mut(&mut self) -> &mut (isize, String) {
        self.stack.last_mut().unwrap()
    }

    /// Get the absolute path of the stack top inode.
    fn top_path(&self) -> AbsPath {
        AbsPath::new(
            &self
                .stack
                .iter()
                .cloned()
                .map(|(_, name)| name)
                .collect::<Vec<_>>()
                .join("/"),
        )
        .unwrap()
    }

    /// Open inode `name` relative to the stack top directory.
    /// Send `openat` command to target kernel.
    fn openat_command(&mut self, name: &str) -> Result<(), Error> {
        self.send_command(&ModelOpenat::from(Openat::new(
            self.top().0,
            Path(heapless::String::from_str(name).unwrap()),
            OpenFlags::RDONLY,
            FileMode::empty(),
        )))
    }

    /// Get the newly opened fd from target kernel.
    fn openat_result(&mut self) -> Result<isize, Error> {
        if self.receive_retv() >= 0 {
            Ok(self.receive_retv())
        } else {
            Err(Error::Io)
        }
    }

    /// Read a directory entry from the stack top directory.
    /// Send `getdents` command to target kernel.
    fn getdents_command(&mut self) -> Result<(), Error> {
        self.send_command(&ModelGetdents::from(Getdents::new(self.top().0, 1)))
    }

    /// Get the newly read directory entry from target kernel.
    fn getdents_result(&mut self) -> Result<Option<DirEntry>, Error> {
        let retv = self.receive_retv();
        if retv > 0 {
            let data = self.receive_extra_data(size_of::<DirEntry>()).unwrap();
            Ok(Some(unsafe { *(data.as_ptr() as *const DirEntry) }))
        } else if retv == 0 {
            Ok(None)
        } else {
            Err(Error::Io)
        }
    }

    /// Get the file status of the stack top inode.
    /// Send `fstat` command to target kernel.
    fn fstat_command(&mut self) -> Result<(), Error> {
        self.send_command(&ModelFstat::from(Fstat::new(self.top().0)))
    }

    /// Get the newly read file status from target kernel.
    fn fstat_result(&mut self) -> Result<FileStat, Error> {
        if self.receive_retv() >= 0 {
            let data = self.receive_extra_data(size_of::<FileStat>()).unwrap();
            Ok(unsafe { *(data.as_ptr() as *const FileStat) })
        } else {
            Err(Error::Io)
        }
    }

    /// Close the stack top inode.
    /// Send `close` command to target kernel.
    fn close_command(&mut self) -> Result<(), Error> {
        self.send_command(&ModelClose::from(Close::new(self.top().0)))
    }

    /// Get close result from target kernel.
    fn close_result(&mut self) -> Result<(), Error> {
        if self.receive_retv() >= 0 {
            Ok(())
        } else {
            Err(Error::Io)
        }
    }

    /// Get current working directory.
    /// Send `getcwd` command to target kernel.
    fn getcwd_command(&mut self) -> Result<(), Error> {
        self.send_command(&ModelGetcwd::from(Getcwd::new()))
    }

    /// Get current working directory from target kernel.
    fn getcwd_result(&mut self) -> Result<AbsPath, Error> {
        if self.receive_retv() >= 0 {
            let data = self.receive_extra_data(MAX_PATH_LEN).unwrap();
            // 2 + n format
            let len = u16::from_le_bytes(data[0..2].try_into().unwrap());
            let path = unsafe { str::from_utf8_unchecked(&data[2..2 + len as usize]) };
            Ok(AbsPath::new(path).unwrap())
        } else {
            Err(Error::Io)
        }
    }
}

impl CommandChannel<FileSystem> for FsTestPort {
    fn send_command(&mut self, command: &dyn Command<FileSystem>) -> Result<(), Error> {
        self.cmd_chan.send_command(command)
    }
    fn receive_retv(&mut self) -> isize {
        <MemCommandChannel<QemuMem, QemuMem> as CommandChannel<FileSystem>>::receive_retv(
            &mut self.cmd_chan,
        )
    }
    fn receive_extra_data(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        <MemCommandChannel<QemuMem, QemuMem> as CommandChannel<FileSystem>>::receive_extra_data(
            &mut self.cmd_chan,
            len,
        )
    }
}

impl StateChannel<FileSystem> for FsTestPort {
    fn start_state_retrieval(&mut self) -> Result<(), Error> {
        // Clear collections
        self.stack.clear();
        self.seen_inodes.clear();
        self.fs.clear();
        // Open root directory
        self.stack.push((-1, String::new()));
        self.openat_command("/")?;
        self.step = Step::Open;
        Ok(())
    }

    /// State retrieval process can be regarded as a finite state machine.
    ///
    /// This function is the state transition function.
    fn retrieve_state_data(&mut self) -> Result<bool, Error> {
        match self.step {
            Step::Open => {
                let fd = self.openat_result()?;
                // `top` is pushed at `Getdents` step.
                self.top_mut().0 = fd;
                self.fstat_command()?;
                self.step = Step::Fstat;
                Ok(false)
            }
            Step::Fstat => {
                let stat = self.fstat_result()?;
                if let Some(path) = self.seen_inodes.get(&stat.ino) {
                    // The inode is already been visited i.e. a hard link.
                    // Create an alias in the filesystem.
                    self.fs.insert_alias(path, self.top_path());
                } else {
                    self.fs.insert(self.top_path(), Inode::from_stat(&stat));
                }
                match stat.kind {
                    FileKind::File => {
                        // The inode is a file, close it.
                        self.close_command()?;
                        self.step = Step::Close;
                    }
                    FileKind::Directory => {
                        // The inode is a directory, get its entries.
                        self.getdents_command()?;
                        self.step = Step::Getdents;
                    }
                }
                Ok(false)
            }
            Step::Close => {
                self.close_result()?;
                self.stack.pop();
                if self.stack.is_empty() {
                    // No more directories to visit, get cwd.
                    self.getcwd_command()?;
                    self.step = Step::Getcwd;
                } else {
                    // Go back to the parent directory.
                    self.getdents_command()?;
                    self.step = Step::Getdents;
                }
                Ok(false)
            }
            Step::Getdents => {
                let dent = self.getdents_result()?;
                if let Some(dent) = dent {
                    if dent.name() == "." || dent.name() == ".." {
                        // Ignore "." and "..".
                        self.getdents_command()?;
                        self.step = Step::Getdents;
                    } else {
                        // Push to stack, fd will be updated later.
                        self.stack.push((-1, dent.name().to_owned()));
                        self.openat_command(dent.name())?;
                        self.step = Step::Open;
                    }
                } else {
                    // No more entries, close the directory.
                    self.close_command()?;
                    self.step = Step::Close;
                }
                Ok(false)
            }
            Step::Getcwd => {
                self.cwd = self.getcwd_result()?;
                Ok(true)
            }
        }
    }

    fn finish_state_retrieval(&mut self) -> Result<FileSystem, Error> {
        Ok(FileSystem::new(self.fs.clone(), self.cwd.clone(), 0, 0))
    }
}

impl TestPort<FileSystem> for FsTestPort {}
