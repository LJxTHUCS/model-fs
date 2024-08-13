use crate::command::{
    Chdir as ModelChdir, Close as ModelClose, Dup as ModelDup, Linkat as ModelLinkat,
    Mkdirat as ModelMkdirat, Openat as ModelOpenat, Unlinkat as ModelUnlinkat,
};
use crate::fs::{FileSystem, FDCWD};
use cmdgen::{Constant, DefaultOr, Generator, RandomFlags, SwitchConstant, UniformCollection};
use km_checker::{Command, Commander, Error};
use km_command::fs::{
    Chdir, Close, Dup, FileMode, Linkat, Mkdirat, OpenFlags, Openat, Path, Unlinkat,
};
use std::str::FromStr;

/// Command type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandType {
    Openat,
    Mkdirat,
    Linkat,
    Unlinkat,
    Dup,
    Close,
    Chdir,
}

/// All available file names.
// const NAMES: [&str; 7] = ["aaa", "bbb", "ccc", "ddd", "eee", "fff", "ggg"];
const NAMES: [&str; 7] = ["a", "aa", "aaa", "aaaa", "aaaaa", "aaaaaa", "aaaaaaa"];

#[cfg(not(feature = "fat"))]
/// All available commands.
const COMMANDS: [CommandType; 7] = [
    CommandType::Openat,
    CommandType::Mkdirat,
    CommandType::Linkat,
    CommandType::Unlinkat,
    CommandType::Dup,
    CommandType::Close,
    CommandType::Chdir,
];

#[cfg(feature = "fat")]
/// All available commands. FAT filesystem does not support linkat.
const COMMANDS: [CommandType; 6] = [
    CommandType::Openat,
    CommandType::Mkdirat,
    CommandType::Unlinkat,
    CommandType::Dup,
    CommandType::Close,
    CommandType::Chdir,
];

pub struct FsCommander;

impl Commander<FileSystem> for FsCommander {
    fn command(&mut self, state: &FileSystem) -> Result<Box<dyn Command<FileSystem>>, Error> {
        // Generators
        let mut cmd_gen = UniformCollection::new(COMMANDS.to_vec());
        let mut fd_gen = DefaultOr::new(
            FDCWD,
            SwitchConstant::new(
                Constant::new(FDCWD),
                UniformCollection::new(
                    state
                        .all_fds()
                        .into_iter()
                        .filter(|fd| ![0, 1, 2].contains(fd))
                        .collect(),
                ),
                0.2,
            ),
        );
        let mut abs_path_gen = UniformCollection::new(
            state
                .inodes
                .keys()
                .into_iter()
                .map(|k| {
                    Path(heapless::String::from_str(&("/".to_owned() + &k.to_string())).unwrap())
                })
                .collect(),
        );
        let mut rel_path_gen = UniformCollection::new(
            NAMES
                .iter()
                .map(|name| Path(heapless::String::from_str(name).unwrap()))
                .collect(),
        );
        let mut oflags_gen = RandomFlags::new(0.5);
        oflags_gen.exclude(OpenFlags::DIRECTORY);
        let mut fmode_gen = RandomFlags::new(0.4);
        fmode_gen.include(FileMode::USER_READ);
        let mut unlinkat_flags_gen = RandomFlags::new(0.3);

        // Generate
        let cmd: Box<dyn Command<FileSystem>> = match cmd_gen.generate() {
            CommandType::Openat => Box::new(ModelOpenat(Openat::new(
                fd_gen.generate(),
                rel_path_gen.generate(),
                oflags_gen.generate(),
                fmode_gen.generate(),
            ))),
            CommandType::Close => Box::new(ModelClose(Close::new(fd_gen.generate()))),
            CommandType::Chdir => Box::new(ModelChdir(Chdir::new(abs_path_gen.generate()))),
            CommandType::Mkdirat => Box::new(ModelMkdirat(Mkdirat::new(
                fd_gen.generate(),
                rel_path_gen.generate(),
                fmode_gen.generate(),
            ))),
            CommandType::Unlinkat => Box::new(ModelUnlinkat(Unlinkat::new(
                fd_gen.generate(),
                rel_path_gen.generate(),
                unlinkat_flags_gen.generate(),
            ))),
            CommandType::Linkat => Box::new(ModelLinkat(Linkat::new(
                fd_gen.generate(),
                rel_path_gen.generate(),
                fd_gen.generate(),
                rel_path_gen.generate(),
            ))),
            CommandType::Dup => Box::new(ModelDup(Dup::new(fd_gen.generate()))),
        };
        Ok(cmd)
    }
}
