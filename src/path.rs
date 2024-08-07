use km_checker::AbstractState;
use km_command::fs::Path;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// Absolute file path.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AbsPath(Vec<String>);

impl Debug for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.0.join("/"))
    }
}

impl Deref for AbsPath {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AbsPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AbstractState for AbsPath {
    fn matches(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn update(&mut self, other: &Self) {
        self.0 = other.0.clone();
    }
}

impl From<Path> for AbsPath {
    /// Convert a `km_command::fs::Path` to an `AbsPath`.
    ///
    /// Callers should ensure that the `Path` is absolute.
    fn from(path: Path) -> Self {
        Self(
            path.strip_prefix("/")
                .unwrap()
                .split("/")
                .map(|s| s.to_owned())
                .collect(),
        )
    }
}

/// Relative file path.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RelPath(Vec<String>);

impl Debug for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("/"))
    }
}

impl Deref for RelPath {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Path> for RelPath {
    /// Convert a `km_command::fs::Path` to a `RelPath`.
    ///
    /// Callers should ensure that the `Path` is relative.
    fn from(path: Path) -> Self {
        Self(path.split("/").map(|s| s.to_owned()).collect())
    }
}

impl AbsPath {
    /// Create a new absolute path.
    pub fn new(path: Vec<String>) -> Self {
        Self(path)
    }

    /// Root absolute path.
    pub fn root() -> Self {
        Self(vec![])
    }

    /// Concatenate a relative path to this absolute path.
    pub fn join(&self, rel_path: &RelPath) -> Self {
        Self(
            self.0
                .clone()
                .into_iter()
                .chain(rel_path.0.clone())
                .collect(),
        )
    }

    /// Get the parent directory of this absolute path.
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            Some(Self(self[..self.len() - 1].to_vec()))
        }
    }
}

impl RelPath {
    /// Current directory.
    pub fn cur() -> Self {
        Self(vec![".".to_owned()])
    }
    /// Parent directory.
    pub fn parent() -> Self {
        Self(vec!["..".to_owned()])
    }
}
