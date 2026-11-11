/**
 * Virtual File System
 *
 * An in-memory tree of directories and files, navigated by path.
 * File contents live in the VMM: a file only knows the virtual pages
 * that hold its bytes.
 */
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};

const SEPARATOR: &str = "/";

#[derive(Debug, Default)]
pub struct File {
    pub size: u64,
    pub pages: Vec<u64>,
}

#[derive(Debug, Default)]
pub struct Directory {
    entries: BTreeMap<String, Node>,
}

#[derive(Debug)]
pub enum Node {
    File(File),
    Directory(Directory),
}

impl Node {
    /// Every page used by this node, including the ones of nested files.
    pub fn pages(&self) -> Vec<u64> {
        match self {
            Self::File(file) => file.pages.clone(),
            Self::Directory(dir) => dir.entries.values().flat_map(Node::pages).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VfsError {
    NotFound(PathBuf),
    AlreadyExists(PathBuf),
    NotADirectory(PathBuf),
    IsADirectory(PathBuf),
    InUse(PathBuf),
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "{}: no such file or directory", path.display()),
            Self::AlreadyExists(path) => write!(f, "{}: already exists", path.display()),
            Self::NotADirectory(path) => write!(f, "{}: not a directory", path.display()),
            Self::IsADirectory(path) => write!(f, "{}: is a directory", path.display()),
            Self::InUse(path) => write!(
                f,
                "{}: is the current directory or one of its parents",
                path.display()
            ),
        }
    }
}

impl std::error::Error for VfsError {}

#[derive(Debug)]
pub struct Vfs {
    root: Directory,
    cwd: PathBuf,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            root: Directory::default(),
            cwd: PathBuf::from(SEPARATOR),
        }
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Turns `path` into a normalized absolute path, relative to the cwd.
    pub fn resolve(&self, path: &str) -> PathBuf {
        let mut resolved = if path.starts_with(SEPARATOR) {
            PathBuf::from(SEPARATOR)
        } else {
            self.cwd.clone()
        };

        for name in path.split(SEPARATOR) {
            match name {
                "" | "." => {}
                ".." => {
                    resolved.pop();
                }
                name => resolved.push(name),
            }
        }
        resolved
    }

    pub fn change_dir(&mut self, path: &str) -> Result<(), VfsError> {
        let path = self.resolve(path);
        self.dir(&path)?;
        self.cwd = path;
        Ok(())
    }

    /// Creates the directory and any missing parent, like `mkdir -p`.
    pub fn create_dir_all(&mut self, path: &str) -> Result<(), VfsError> {
        let path = self.resolve(path);
        let mut created = false;
        let mut dir = &mut self.root;

        for name in names(&path) {
            let node = dir.entries.entry(name.to_string()).or_insert_with(|| {
                created = true;
                Node::Directory(Directory::default())
            });
            dir = match node {
                Node::Directory(dir) => dir,
                Node::File(_) => return Err(VfsError::NotADirectory(path.clone())),
            };
        }

        if created {
            Ok(())
        } else {
            Err(VfsError::AlreadyExists(path))
        }
    }

    pub fn create_file(&mut self, path: &str) -> Result<(), VfsError> {
        let path = self.resolve(path);
        let (parent, name) = split(&path)?;
        let dir = self.dir_mut(parent)?;

        if dir.entries.contains_key(name) {
            return Err(VfsError::AlreadyExists(path.clone()));
        }
        dir.entries
            .insert(name.to_string(), Node::File(File::default()));
        Ok(())
    }

    /// Detaches the node from the tree. The caller owns its pages.
    pub fn remove(&mut self, path: &str) -> Result<Node, VfsError> {
        let path = self.resolve(path);
        if self.cwd.starts_with(&path) {
            return Err(VfsError::InUse(path));
        }

        let (parent, name) = split(&path)?;
        self.dir_mut(parent)?
            .entries
            .remove(name)
            .ok_or_else(|| VfsError::NotFound(path.clone()))
    }

    pub fn list(&self, path: &str) -> Result<impl Iterator<Item = (&str, &Node)>, VfsError> {
        let dir = self.dir(&self.resolve(path))?;
        Ok(dir.entries.iter().map(|(name, node)| (name.as_str(), node)))
    }

    pub fn file(&self, path: &str) -> Result<&File, VfsError> {
        let path = self.resolve(path);
        let (parent, name) = split(&path)?;
        match self.dir(parent)?.entries.get(name) {
            Some(Node::File(file)) => Ok(file),
            Some(Node::Directory(_)) => Err(VfsError::IsADirectory(path.clone())),
            None => Err(VfsError::NotFound(path.clone())),
        }
    }

    pub fn file_mut(&mut self, path: &str) -> Result<&mut File, VfsError> {
        let path = self.resolve(path);
        let (parent, name) = split(&path)?;
        match self.dir_mut(parent)?.entries.get_mut(name) {
            Some(Node::File(file)) => Ok(file),
            Some(Node::Directory(_)) => Err(VfsError::IsADirectory(path.clone())),
            None => Err(VfsError::NotFound(path.clone())),
        }
    }

    fn dir(&self, path: &Path) -> Result<&Directory, VfsError> {
        let mut dir = &self.root;
        for name in names(path) {
            dir = match dir.entries.get(name) {
                Some(Node::Directory(dir)) => dir,
                Some(Node::File(_)) => return Err(VfsError::NotADirectory(path.to_path_buf())),
                None => return Err(VfsError::NotFound(path.to_path_buf())),
            };
        }
        Ok(dir)
    }

    fn dir_mut(&mut self, path: &Path) -> Result<&mut Directory, VfsError> {
        let mut dir = &mut self.root;
        for name in names(path) {
            dir = match dir.entries.get_mut(name) {
                Some(Node::Directory(dir)) => dir,
                Some(Node::File(_)) => return Err(VfsError::NotADirectory(path.to_path_buf())),
                None => return Err(VfsError::NotFound(path.to_path_buf())),
            };
        }
        Ok(dir)
    }
}

fn names(path: &Path) -> impl Iterator<Item = &str> {
    path.components().filter_map(|component| match component {
        Component::Normal(name) => name.to_str(),
        _ => None,
    })
}

/// Splits an absolute path into its parent directory and its last name.
fn split(path: &Path) -> Result<(&Path, &str), VfsError> {
    match (
        path.parent(),
        path.file_name().and_then(|name| name.to_str()),
    ) {
        (Some(parent), Some(name)) => Ok((parent, name)),
        _ => Err(VfsError::IsADirectory(path.to_path_buf())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_in(vfs: &Vfs, path: &str) -> Vec<String> {
        vfs.list(path)
            .unwrap()
            .map(|(name, _)| name.to_string())
            .collect()
    }

    #[test]
    fn resolve_normalizes_paths() {
        let mut vfs = Vfs::new();
        vfs.create_dir_all("a/b").unwrap();
        vfs.change_dir("a/b").unwrap();
        assert_eq!(vfs.resolve("../c/./d"), PathBuf::from("/a/c/d"));
        assert_eq!(vfs.resolve("/x"), PathBuf::from("/x"));
        assert_eq!(vfs.resolve("../../../.."), PathBuf::from("/"));
    }

    #[test]
    fn mkdir_nested_creates_hierarchy() {
        let mut vfs = Vfs::new();
        vfs.create_dir_all("a/b/c").unwrap();
        assert_eq!(names_in(&vfs, "/"), ["a"]);
        assert_eq!(names_in(&vfs, "/a"), ["b"]);
        assert_eq!(names_in(&vfs, "/a/b"), ["c"]);
        assert_eq!(
            vfs.create_dir_all("a/b"),
            Err(VfsError::AlreadyExists(PathBuf::from("/a/b")))
        );
    }

    #[test]
    fn remove_directory_detaches_it() {
        let mut vfs = Vfs::new();
        vfs.create_dir_all("foo").unwrap();
        vfs.remove("foo").unwrap();
        assert!(names_in(&vfs, "/").is_empty());
    }

    #[test]
    fn names_with_dots_keep_their_kind() {
        let mut vfs = Vfs::new();
        vfs.create_dir_all("my.dir").unwrap();
        vfs.create_file(".env").unwrap();
        assert!(matches!(vfs.remove("my.dir"), Ok(Node::Directory(_))));
        assert!(matches!(vfs.remove(".env"), Ok(Node::File(_))));
    }

    #[test]
    fn cannot_remove_cwd_or_its_parents() {
        let mut vfs = Vfs::new();
        vfs.create_dir_all("a/b").unwrap();
        vfs.change_dir("a/b").unwrap();
        assert!(matches!(vfs.remove("/a"), Err(VfsError::InUse(_))));
        assert!(matches!(vfs.remove("/"), Err(VfsError::InUse(_))));
    }

    #[test]
    fn files_are_not_directories() {
        let mut vfs = Vfs::new();
        vfs.create_file("f.txt").unwrap();
        assert!(matches!(
            vfs.change_dir("f.txt"),
            Err(VfsError::NotADirectory(_))
        ));
        assert!(matches!(
            vfs.create_file("f.txt"),
            Err(VfsError::AlreadyExists(_))
        ));
        assert!(matches!(
            vfs.create_file("f.txt/x"),
            Err(VfsError::NotADirectory(_))
        ));
    }

    #[test]
    fn files_are_found_by_relative_and_absolute_path() {
        let mut vfs = Vfs::new();
        vfs.create_dir_all("d").unwrap();
        vfs.create_file("d/x.lua").unwrap();
        assert!(vfs.file("/d/x.lua").is_ok());
        assert!(vfs.file("d/x.lua").is_ok());
        assert!(matches!(vfs.file("d"), Err(VfsError::IsADirectory(_))));
    }
}
