use crate::error::FsError;
use km_checker::AbstractState;
use km_command::fs::Path;
use std::{fmt::Debug, vec};

/// Normalized absolute file path.
///
/// - Cannot contain "." or "..".
/// - Cannot start or end with "/".
#[derive(Clone, PartialEq, Eq, Hash, AbstractState, PartialOrd, Ord)]
pub struct AbsPath(String);

impl Debug for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("/{}", &self.0))
    }
}

impl TryFrom<Path> for AbsPath {
    type Error = FsError;
    fn try_from(value: Path) -> Result<Self, Self::Error> {
        if !value.absolute() {
            return Err(FsError::InvalidPath);
        }
        Self::normalize(&value.0)
    }
}

impl ToString for AbsPath {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl AbsPath {
    /// Create a new absolute path.
    pub fn new(path: &str) -> Result<Self, FsError> {
        Self::normalize(path)
    }

    /// Root absolute path.
    pub fn root() -> Self {
        Self("".to_owned())
    }

    /// Check if this path is root.
    pub fn is_root(&self) -> bool {
        self.0 == ""
    }

    /// Check if this path is an ancestor of another path.
    pub fn is_ancestor(&self, other: &Self) -> bool {
        other.0.starts_with(&format!("{}/", self.0))
    }

    /// Get the parent directory of this absolute path.
    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            None
        } else {
            let mut components = self.0.split('/').collect::<Vec<_>>();
            components.pop();
            Some(Self(components.join("/")))
        }
    }

    /// Concatenate a relative path to this absolute path.
    pub fn join(&self, rel_path: &RelPath) -> Result<Self, FsError> {
        let mut path = self.0.clone();
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(&rel_path.0);
        Self::normalize(&path)
    }

    /// Normalize a `path` string, then create an `AbsPath`.
    ///
    /// - Remove leading and trailing "/".
    /// - Remove "." and ".." components.
    fn normalize(path: &str) -> Result<Self, FsError> {
        // Possibly remove leading and trailing "/".
        let path = path.strip_prefix("/").unwrap_or(path);
        let path = path.strip_suffix("/").unwrap_or(path);
        // Empty path is root.
        if path.is_empty() {
            return Ok(Self::root());
        }
        // Split nonempty path into components.
        let mut normalized = vec![];
        for component in path.split("/") {
            match component {
                // Empty component, error.
                "" => return Err(FsError::InvalidPath),
                // Current directory, do nothing.
                "." => (),
                // Parent directory, remove last component if possible.
                ".." => {
                    if !normalized.is_empty() {
                        normalized.pop();
                    }
                }
                // Normal component, add to normalized path.
                _ => normalized.push(component),
            }
        }
        if normalized.is_empty() {
            Ok(Self::root())
        } else {
            Ok(Self(normalized.join("/")))
        }
    }
}

/// Relative file path. Must not start with "/".
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RelPath(String);

impl Debug for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<Path> for RelPath {
    type Error = FsError;
    fn try_from(value: Path) -> Result<Self, Self::Error> {
        if !value.relative() {
            return Err(FsError::InvalidPath);
        }
        Ok(Self(value.to_string()))
    }
}

impl ToString for RelPath {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl RelPath {
    /// Create a new relative path.
    pub fn new(path: String) -> Self {
        Self(path)
    }

    /// Current directory.
    pub fn cur() -> Self {
        Self(".".to_owned())
    }

    /// Parent directory.
    pub fn parent() -> Self {
        Self("..".to_owned())
    }
}
