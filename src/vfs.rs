use std::{borrow::{Borrow, BorrowMut}, collections::HashMap, path::PathBuf};
use crate::utils;

const SEPARATOR: &str = "/";

#[derive(Debug, Clone)]
struct File {
    name: String,
    path: String,
    content: String,
}

#[derive(Debug, Clone)]
struct Directory {
    name: String,
    parent: Option<Box<Directory>>,
    files: HashMap<String, File>,
    subdirectories: HashMap<String, Directory>,
    path: PathBuf,
}

impl Directory {
    pub fn new(name: &str, path: PathBuf, parent: Option<Box<Directory>>) -> Self {
        Self {
            name: String::from(name),
            parent,
            files: HashMap::new(),
            subdirectories: HashMap::new(),
            path,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vfs {
    root: Directory,
    cwd: PathBuf,
}

impl Vfs {
    pub fn new() -> Self {
        let root_path = PathBuf::from(SEPARATOR);
        let root = Directory::new("/", PathBuf::from("/"), None);
        Self {
            root,
            cwd: root_path,
        }
    }

    pub fn list(&mut self) {
        let current_dir = self.get_dir_in_vfs(
            self.cwd
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
                .as_str(),
        );
        if let Some(dir) = current_dir {
            if dir.files.is_empty() {
                println!("No files found");
            } else {
                for file in dir.files.values() {
                    println!("{}", file.name);
                }
            }

            if dir.subdirectories.is_empty() {
                println!("No directories found");
            } else {
                for subdir in dir.subdirectories.values() {
                    println!("{}/", subdir.name);
                }
            }
            
        }
    }

    pub fn pwd(self) {
        println!(
            "{}",
            self.cwd
                .clone()
                .into_os_string()
                .into_string()
                .unwrap()
        );
    }

    pub fn add_directory_recursive(&mut self, dirnames: &str) {
        let mut current_path = self.cwd.clone();
        let mut current_dir = match self.get_dir_in_vfs(current_path.to_str().unwrap()) {
            Some(dir) => dir,
            None => return,
        };
        
        if utils::is_unix_symbol(dirnames) {
            println!("Invalid directory name");
            return;
        }

        if dirnames.split(SEPARATOR).count() > 1 {
            let dirs = dirnames.split(SEPARATOR);
            for dir in dirs {
                if !current_dir.subdirectories.contains_key(dir) {
                    current_dir.subdirectories.insert(String::from(dir), Directory::new(dir, current_path.join(dir), Some(Box::new(current_dir.clone()))));
                } else {
                    let curr = current_dir.subdirectories.get_mut(dir).unwrap();
                    current_path = curr.path.clone();
                    current_dir = curr;
                }
            }
        } else {
            if !current_dir.subdirectories.contains_key(dirnames) {
                current_dir.subdirectories.insert(String::from(dirnames), Directory::new(dirnames, current_path.join(dirnames), Some(Box::new(current_dir.clone()))));
            } else {
                println!("Directory already exists");
            }
        }
    }

    pub fn change_dir(&mut self, dir: &str) {
        if dir == "." {
            return;
        }

        if dir == "/" {
            self.cwd = PathBuf::from(SEPARATOR);
            return;
        }

        if dir == ".." {
            self.cwd = self.cwd.parent().unwrap().to_path_buf();
            return;
        }


        let dir_to = self.get_dir_in_vfs(self.cwd.join(dir).to_str().unwrap());
        if let Some(dir) = dir_to {
            self.cwd = dir.path.clone();
        } else {
            println!("Directory not found");
        }
    }

    fn get_dir_in_vfs(&mut self, path: &str) -> Option<&mut Directory> {
        let mut current_dir = &mut self.root;
        for dir in path.split(SEPARATOR) {
            if dir.is_empty() {
                continue;
            }
            if let Some(new_dir) = current_dir.subdirectories.get_mut(dir) {
                current_dir = new_dir;
            } else {
                return None;
            }
        }
        Some(current_dir)
    }
}

pub fn init_vfs() -> Vfs {
    Vfs::new()
}