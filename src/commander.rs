use crate::command::{
    Chdir as ModelChdir, Close as ModelClose, Dup as ModelDup, Linkat as ModelLinkat,
    Mkdirat as ModelMkdirat, Openat as ModelOpenat, Unlinkat as ModelUnlinkat,
};
use crate::fs::{FileSystem, FDCWD};
use cmdgen::{Constant, DefaultOr, Generator, SwitchConstant, UniformCollection, UniformRange};
use km_checker::{Command, Commander, Error};
use km_command::fs::{Chdir, Close, Dup, FileMode, Linkat, Mkdirat, OpenFlags, Openat, Unlinkat};
use std::str::FromStr;

const NAMES: [&str; 7] = ["foo", "bar", "baz", "foobar", "bbb", "a", "aa"];

pub struct FsCommander;

impl Commander<FileSystem> for FsCommander {
    fn command(&mut self, state: &FileSystem) -> Result<Box<dyn Command<FileSystem>>, Error> {
        // Generators
        let mut cmd_gen = UniformRange::new(0, 7);
        let mut fd_gen = DefaultOr::new(
            FDCWD,
            SwitchConstant::new(
                Constant::new(FDCWD),
                UniformCollection::new(state.all_fds()),
                0.2,
            ),
        );
        let mut abs_path_gen = UniformCollection::new(
            state
                .inodes
                .keys()
                .into_iter()
                .map(|k| k.to_string())
                .collect(),
        );
        let mut rel_path_gen = UniformCollection::new(NAMES.to_vec());
        let mut oflags_gen = UniformRange::new(0, 7);
        let mut fmode_gen = UniformRange::new(0, 7);

        // Generate
        let cmd: Box<dyn Command<FileSystem>> = match cmd_gen.generate() {
            0 => Box::new(ModelOpenat::from(Openat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
                OpenFlags::from_bits_truncate(oflags_gen.generate()),
                FileMode::from_bits_truncate(fmode_gen.generate()),
            ))),
            1 => Box::new(ModelClose::from(Close::new(fd_gen.generate()))),
            2 => Box::new(ModelChdir::from(Chdir::new(translate_path(
                &abs_path_gen.generate(),
            )))),
            3 => Box::new(ModelMkdirat::from(Mkdirat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
                FileMode::from_bits_truncate(fmode_gen.generate()),
            ))),
            4 => Box::new(ModelUnlinkat::from(Unlinkat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
            ))),
            5 => Box::new(ModelLinkat::from(Linkat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
            ))),
            6 => Box::new(ModelDup::from(Dup::new(fd_gen.generate()))),
            _ => unreachable!(),
        };
        Ok(cmd)
    }
}

fn translate_path(path: &str) -> km_command::fs::Path {
    km_command::fs::Path(heapless::String::from_str(path).unwrap())
}
