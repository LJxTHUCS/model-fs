use km_checker::AbstractState;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// Absolute file path.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AbsPath(Vec<String>);

impl AbstractState for AbsPath {
    fn matches(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn update(&mut self, other: &Self) {
        self.0 = other.0.clone();
    }
}

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

impl AbsPath {
    /// Translate an absolute `km_command::fs::Path` to an `AbsPath`.
    ///
    /// Caller must ensure `path` is an absolute path.
    pub fn from_abs_command_path(abs_path: &km_command::fs::Path) -> Self {
        Self(abs_path.split("/").map(|s| s.to_string()).collect())
    }

    /// Concatenate a relative `km_command::fs::Path` to this `AbsPath`.
    ///
    /// Caller must ensure `path` is a relative path.
    pub fn from_rel_command_path(&self, rel_path: &km_command::fs::Path) -> Self {
        let mut new_path = self.clone();
        new_path.extend(rel_path.split("/").map(|s| s.to_string()));
        Self(new_path.0)
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
    ///
    /// `rel_path` format must be `a/b/c`.
    pub fn join(&self, rel_path: &str) -> Self {
        let mut new_path = self.clone();
        new_path.extend(rel_path.split("/").map(|s| s.to_string()));
        Self(new_path.0)
    }
}
