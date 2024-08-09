use crate::fs::{FileSystem, FDCWD};
use crate::path::{AbsPath, RelPath};
use cmdgen::{ConstOr, Constant, Generator, UniformRange, UniformResource};
use km_checker::{Command, Commander, Error};
use km_command::fs::{Chdir, Close, FileMode, Linkat, Mkdirat, OpenFlags, Openat, Unlinkat};
use std::str::FromStr;

const NAMES: [&str; 9] = [
    "foo",
    "bar",
    "baz",
    "foobar",
    "foobarbaz",
    "bbb",
    "a",
    "aa",
    "aaa",
];

pub struct FsCommander;

impl Commander<FileSystem> for FsCommander {
    fn command(&mut self, state: &FileSystem) -> Result<Box<dyn Command<FileSystem>>, Error> {
        let cmd = UniformRange::new(0, 6).generate();
        // Resources
        let all_fds = state.all_fds();
        let fd_resource = if !all_fds.is_empty() {
            UniformResource::new(state.all_fds())
        } else {
            UniformResource::new(vec![FDCWD])
        };
        let abs_path_resource = UniformResource::new(state.inodes.keys());
        let rel_path_resource =
            UniformResource::new(NAMES.map(|name| RelPath::new(name.to_owned())).to_vec());

        // Generators
        let mut fd_gen = ConstOr::new(Constant(FDCWD), fd_resource);
        let mut abs_path_gen = abs_path_resource;
        let mut rel_path_gen = rel_path_resource;
        let mut oflags_gen = UniformRange::new(0, 7);
        let mut fmode_gen = UniformRange::new(0, 7);

        // Generate
        let cmd: Box<dyn Command<FileSystem>> = match cmd {
            0 => Box::new(crate::command::Openat::from(Openat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
                OpenFlags::from_bits_truncate(oflags_gen.generate()),
                FileMode::from_bits_truncate(fmode_gen.generate()),
            ))),
            1 => Box::new(crate::command::Close::from(Close::new(fd_gen.generate()))),
            2 => Box::new(crate::command::Chdir::from(Chdir::new(translate_path(
                &abs_path_gen.generate(),
            )))),
            3 => Box::new(crate::command::Mkdirat::from(Mkdirat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
                FileMode::from_bits_truncate(fmode_gen.generate()),
            ))),
            4 => Box::new(crate::command::Unlinkat::from(Unlinkat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
            ))),
            5 => Box::new(crate::command::Linkat::from(Linkat::new(
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
                fd_gen.generate(),
                translate_path(&rel_path_gen.generate()),
            ))),
            _ => unreachable!(),
        };
        Ok(cmd)
    }
}

fn translate_path(path: &str) -> km_command::fs::Path {
    km_command::fs::Path(heapless::String::from_str(path).unwrap())
}
