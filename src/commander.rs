use crate::command::{
    Chdir as ModelChdir, Close as ModelClose, Dup as ModelDup, Linkat as ModelLinkat,
    Mkdirat as ModelMkdirat, Openat as ModelOpenat, Unlinkat as ModelUnlinkat,
};
use crate::fs::{FileSystem, FDCWD};
use cmdgen::{Constant, DefaultOr, Generator, SwitchConstant, UniformCollection, UniformRange};
use km_checker::{Command, Commander, Error};
use km_command::fs::{Chdir, Close, Dup, FileMode, Linkat, Mkdirat, OpenFlags, Openat, Unlinkat};
use std::str::FromStr;

const NAMES: [&str; 7] = ["aaa", "bbb", "ccc", "ddd", "eee", "fff", "ggg"];

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
            0 => Box::new(ModelOpenat(Openat::new(
                fd_gen.generate(),
                translate_rel_path(&rel_path_gen.generate()),
                OpenFlags::from_bits_truncate(oflags_gen.generate()),
                FileMode::from_bits_truncate(fmode_gen.generate()),
            ))),
            1 => Box::new(ModelClose(Close::new(fd_gen.generate()))),
            2 => Box::new(ModelChdir(Chdir::new(translate_abs_path(
                &abs_path_gen.generate(),
            )))),
            3 => Box::new(ModelMkdirat(Mkdirat::new(
                fd_gen.generate(),
                translate_rel_path(&rel_path_gen.generate()),
                FileMode::from_bits_truncate(fmode_gen.generate()),
            ))),
            4 => Box::new(ModelUnlinkat(Unlinkat::new(
                fd_gen.generate(),
                translate_rel_path(&rel_path_gen.generate()),
            ))),
            5 => Box::new(ModelLinkat(Linkat::new(
                fd_gen.generate(),
                translate_rel_path(&rel_path_gen.generate()),
                fd_gen.generate(),
                translate_rel_path(&rel_path_gen.generate()),
            ))),
            6 => Box::new(ModelDup(Dup::new(fd_gen.generate()))),
            _ => unreachable!(),
        };
        Ok(cmd)
    }
}

fn translate_rel_path(path: &str) -> km_command::fs::Path {
    km_command::fs::Path(heapless::String::from_str(path).unwrap())
}

fn translate_abs_path(path: &str) -> km_command::fs::Path {
    km_command::fs::Path(heapless::String::from_str(&("/".to_owned() + path)).unwrap())
}
