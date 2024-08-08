use km_checker::AbstractState;
use km_command::fs::Path;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// Absolute file path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, AbstractState)]
pub struct AbsPath(String);

impl Deref for AbsPath {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AbsPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Path> for AbsPath {
    /// Convert a `km_command::fs::Path` to an `AbsPath`.
    ///
    /// Callers should ensure that the `Path` is absolute.
    fn from(path: Path) -> Self {
        Self(path.to_string())
    }
}

/// Relative file path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelPath(String);

impl Deref for RelPath {
    type Target = String;
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
        Self(path.to_string())
    }
}

impl AbsPath {
    /// Create a new absolute path.
    pub fn new(path: String) -> Self {
        Self(path)
    }

    /// Root absolute path.
    pub fn root() -> Self {
        Self("/".to_owned())
    }

    /// Concatenate a relative path to this absolute path.
    pub fn join(&self, rel_path: &RelPath) -> Self {
        let mut path = self.0.clone();
        path.push('/');
        path.push_str(&rel_path.0);
        Self(path)
    }

    /// Get the parent directory of this absolute path.
    pub fn parent(&self) -> Option<Self> {
        if self.0 == "/" {
            None
        } else {
            let mut components = self.split('/').collect::<Vec<_>>();
            components.pop();
            Some(Self(components.join("/")))
        }
    }
}

impl RelPath {
    /// Current directory.
    pub fn cur() -> Self {
        Self(".".to_owned())
    }

    /// Parent directory.
    pub fn parent() -> Self {
        Self("..".to_owned())
    }
}
