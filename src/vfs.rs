use crate::editor::Editor;
use crate::utils;
use crate::vmm::Vmm;
use crate::vpm::Vpm;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

const SEPARATOR: &str = "/";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct File {
    name: String,
    path: PathBuf,
    pub vmm_address: Vec<u64>,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct Directory {
    name: String,
    parent: Option<Box<Directory>>,
    files: HashMap<String, Arc<Mutex<File>>>,
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
    pub root: Directory,
    pub cwd: PathBuf,
    pub vpm: Vpm,
}

impl Vfs {
    pub fn new(vpm: Vpm) -> Self {
        let root_path = PathBuf::from(SEPARATOR);
        let root = Directory::new("/", PathBuf::from("/"), None);
        Self {
            root,
            cwd: root_path,
            vpm,
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
                    let file = file.as_ref().lock().unwrap();
                    println!("{} {}", file.name, file.size);
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
            self.cwd.clone().into_os_string().into_string().unwrap()
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
                    current_dir.subdirectories.insert(
                        String::from(dir),
                        Directory::new(
                            dir,
                            current_path.join(dir),
                            Some(Box::new(current_dir.clone())),
                        ),
                    );
                } else {
                    let curr = current_dir.subdirectories.get_mut(dir).unwrap();
                    current_path = curr.path.clone();
                    current_dir = curr;
                }
            }
        } else {
            if !current_dir.subdirectories.contains_key(dirnames) {
                current_dir.subdirectories.insert(
                    String::from(dirnames),
                    Directory::new(
                        dirnames,
                        current_path.join(dirnames),
                        Some(Box::new(current_dir.clone())),
                    ),
                );
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
            match self.cwd.parent() {
                Some(parent) => {
                    self.cwd = parent.to_path_buf();
                    return;
                }
                None => return,
            }
        }

        let dir_to = self.get_dir_in_vfs(self.cwd.join(dir).to_str().unwrap());
        if let Some(dir) = dir_to {
            self.cwd = dir.path.clone();
        } else {
            println!("Directory not found");
        }
    }

    pub fn remove(&mut self, files_path: &str) {
        let file = files_path.split(SEPARATOR).last();
        if file.unwrap().contains(".") {
            let dir_path = &files_path
                .split(SEPARATOR)
                .take(files_path.split(SEPARATOR).count() - 1)
                .collect::<Vec<&str>>()
                .join(SEPARATOR);
            let current_dir = self.get_dir_in_vfs(self.cwd.join(dir_path).to_str().unwrap());
            if let Some(dir) = current_dir {
                if dir.files.contains_key(file.unwrap()) {
                    let file_address_to_deallocate = dir
                        .files
                        .get(file.unwrap())
                        .unwrap()
                        .lock()
                        .unwrap()
                        .vmm_address
                        .clone();
                    dir.files.remove(file.unwrap());
                    let mut vmm = self.vpm.vmm.lock().unwrap();
                    vmm.deallocate_page(file_address_to_deallocate);
                    drop(vmm);
                } else {
                    println!("File {} not found.", file.unwrap());
                }
            } else {
                println!("Directory {} not found.", dir_path);
            }
        } else {
            let dir_to_remove = self.get_dir_in_vfs(self.cwd.join(files_path).to_str().unwrap());
            if let Some(dir) = dir_to_remove {
                dir.parent
                    .as_mut()
                    .unwrap()
                    .subdirectories
                    .remove(&dir.name);
            } else {
                println!("Directory {} not found.", files_path)
            }
        }
    }

    pub fn touch(&mut self, filename: &str) {
        if filename.contains(SEPARATOR) {
            println!("Please provide only filename with extension {}", filename);
            return;
        }

        let mut vmm = self.vpm.vmm.lock().unwrap();
        let (vmm_address, _) = vmm.allocate_page();
        drop(vmm);

        let cwd = self.cwd.clone();
        let current_dir = self.get_dir_in_vfs(cwd.to_str().unwrap()).unwrap();
        if !current_dir.files.contains_key(filename) {
            let new_file = File {
                vmm_address: vec![vmm_address],
                name: filename.to_string(),
                path: PathBuf::from(cwd.join(filename)),
                size: 0,
            };
            current_dir
                .files
                .insert(filename.to_string(), Arc::new(Mutex::new(new_file)));
        } else {
            println!("File {} already exists.", filename)
        }
    }

    /**
     * Allow to write file content, if no bytes are provided, it will open the editor
     */
    pub fn write_file(
        &mut self,
        filename: &str,
        bytes_to_write: Option<Vec<u8>>,
        filepath: Option<&str>,
    ) {
        if let Some(file) = self.get_file_in_cwd(filename) {
            let vmm_clone = Arc::clone(&self.vpm.vmm);
            if bytes_to_write.is_none() {
                self.vpm.execute(move |_| {
                    Editor::write(file, vmm_clone);
                });
            } else {
                self.write_file_bytes(vmm_clone, file, bytes_to_write.unwrap(), filepath);
            }
        }
    }

    fn write_file_bytes(
        &mut self,
        vmm: Arc<Mutex<Vmm>>,
        file: Arc<Mutex<File>>,
        bytes: Vec<u8>,
        filepath: Option<&str>,
    ) {
        if filepath.is_none() {
            println!("Please provide a file path");
            return;
        }

        self.vpm.execute(move |_| {
            let mut vmm = vmm.lock().unwrap();
            let virtual_addresses = vmm.allocate_bytes(bytes.clone());
            let mut file = file.lock().unwrap();
            file.size = bytes.len() as u64;
            file.vmm_address = virtual_addresses;
            file.path = PathBuf::from(filepath.unwrap());
        });
    }

    pub fn read_file(&mut self, filename: &str) {
        if let Some(file) = self.get_file_in_cwd(filename) {
            let vmm_clone = Arc::clone(&self.vpm.vmm);
            self.vpm.execute(move |_| {
                Editor::read(file, vmm_clone);
            });
        }
    }

    pub fn get_file_in_cwd(&mut self, filename: &str) -> Option<Arc<Mutex<File>>> {
        let cwd = self.cwd.clone();
        if let Some(current_dir) = self.get_dir_in_vfs(cwd.to_str().unwrap()) {
            if current_dir.files.contains_key(filename) {
                return Some(current_dir.files.get(filename).unwrap().clone());
            } else {
                println!("File {} not found.", filename);
                return None;
            }
        } else {
            println!("Directory {} not found.", cwd.to_str().unwrap());
            None
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

    pub fn search_file_recursive(
        &self,
        filepath: &str,
        current_dir: &Directory,
    ) -> Option<Arc<Mutex<File>>> {
        if current_dir.files.contains_key(filepath) {
           return Some(current_dir.files.get(filepath).unwrap().clone());
        }

        if filepath.contains(SEPARATOR) {
            let dirs = filepath.split(SEPARATOR);
            for dir in dirs {
                if current_dir.subdirectories.contains_key(dir) {
                    self.search_file_recursive(
                        filepath,
                        current_dir.subdirectories.get(dir).unwrap(),
                    );
                }
            }
        }

        return None;
    }

    pub fn read_file_content_bytes_to_utf8(&self, file: &File) -> Option<String> {
        let vmm_mux = self.vpm.vmm.lock().unwrap();
        let file_content_bytes = vmm_mux.get_bytes(file.vmm_address.clone(), 4096);
        match String::from_utf8(file_content_bytes) {
            Ok(content) => Some(content),
            Err(_) => {
                println!("Error reading file {}", file.name);
                None
            }
        }
    }
}

pub fn init_vfs() -> Vfs {
    Vfs::new(Vpm::new(Arc::new(Mutex::new(Vmm::new(
        1024 * 1024 * 1024 * 4,
    )))))
}
