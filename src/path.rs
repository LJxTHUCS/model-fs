use crate::error::FsError;
use km_checker::AbstractState;
use km_command::fs::Path;
use std::fmt::Debug;

/// Absolute file path. Cannot contain "." or "..".
#[derive(Clone, PartialEq, Eq, Hash, AbstractState, PartialOrd, Ord)]
pub struct AbsPath(String);

impl Debug for AbsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == "" {
            f.write_str("/")
        } else {
            f.write_fmt(format_args!("{}", &self.0))
        }
    }
}

impl TryFrom<Path> for AbsPath {
    type Error = FsError;
    fn try_from(value: Path) -> Result<Self, Self::Error> {
        if !value.absolute() {
            return Err(FsError::InvalidPath);
        }
        Ok(Self(split_and_join(&value.strip_prefix("/").unwrap())?))
    }
}

impl ToString for AbsPath {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl AbsPath {
    /// Create a new absolute path.
    pub fn new(path: String) -> Self {
        Self(path)
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
        let pref = self.0.clone() + "/";
        other.0.starts_with(&pref)
    }

    /// Get the parent directory of this absolute path.
    pub fn parent(&self) -> Option<Self> {
        let path = self.0.clone();
        if self.0 == "" {
            None
        } else {
            let mut components = path.split('/').collect::<Vec<_>>();
            components.pop();
            Some(Self(components.join("/")))
        }
    }

    /// Concatenate a relative path to this absolute path.
    pub fn join(&self, rel_path: &RelPath) -> Self {
        let mut path = self.0.clone();
        path.push('/');
        path.push_str(&rel_path.0);
        Self(path)
    }
}

/// Relative file path.
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
        Ok(Self(split_and_join(&value)?))
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

/// Split a path into components, removing "." and ".." components,
/// then join them back together.
fn split_and_join(path: &str) -> Result<String, FsError> {
    if !path.contains("/") {
        return Ok(path.to_owned());
    }
    let mut components = Vec::new();
    for comp in path.split("/") {
        match comp {
            "" => return Err(FsError::InvalidPath),
            "." => continue,
            ".." => {
                if components.is_empty() {
                    continue;
                }
                components.pop();
            }
            _ => components.push(comp),
        }
    }
    Ok(components.join("/"))
}
