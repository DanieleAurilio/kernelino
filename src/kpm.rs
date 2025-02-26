/**
 * Kernelino Package Manager
 *
 */
use std::{
    collections::HashMap,
    sync::RwLockWriteGuard,
};

use serde_json::Value;

use crate::{
    utils::{self, TarArchive},
    vfs::Vfs,
};

pub struct KPM {
    pub packages: HashMap<String, Package>,
}

#[allow(dead_code)]
pub struct Package {
    name: String,
    version: String,
    path: String,
    size: u64,
    dependencies: Vec<String>,
}

const URL_REGISTRY: &str = "https://formulae.brew.sh/api/formula/";
const KPM_BIN: &str = "/bin";

impl KPM {
    pub fn init() -> Self {
        let packages = HashMap::new();

        KPM { packages }
    }

    pub async fn download(&self, package_name: &str) -> (Option<Vec<u8>>, Option<String>, Option<String>) {
        let unix = utils::is_unix();
        if unix.is_none() {
            return (None, None, None);
        }

        let url = format!("{}{}.json", URL_REGISTRY, package_name);
        let response_repository = utils::http_async_get(url.as_str(), None, false).await;
        if response_repository.is_none() {
            println!("Error downloading package");
            return (None, None, None);
        }

        let repository_json: Value =
            serde_json::from_slice(response_repository.unwrap().as_slice()).unwrap();
        let urls = repository_json.get("urls");
        if urls.is_none() {
            println!("Url not found");
            return (None, None, None);
        }

        let stable = urls.unwrap().get("stable");
        if stable.is_none() {
            println!("Stable version not found");
            return (None, None, None);
        }

        let versions = repository_json.get("versions");
        if versions.is_none() {
            println!("Versions not found");
            return (None, None, None);
        }

        let stable_version = versions.unwrap().get("stable");
        if stable_version.is_none() {
            println!("Version not found");
            return (None, None, None);
        }

        let stable_version_url = match stable.unwrap().get("url") {
            Some(url) => Some(url.as_str().unwrap()).unwrap(),
            None => {
                println!("Stable version url to download not found");
                return (None, None, None);
            }
        };


        println!("Downloading package from {}", stable_version_url);

        let filename = stable_version_url.split("/").last().unwrap();
        let pkg = utils::http_async_get(stable_version_url, None, true).await;
        if pkg.is_none() {
            println!("Error downloading package");
            return (None, None, None);
        } else {
            println!("Package downloaded successfully");
        }

        (Some(pkg.unwrap()), Some(filename.to_string()), Some(stable_version.unwrap().to_string()))
    }

    pub fn write_downloaded_file(
        &self,
        mut vfs: RwLockWriteGuard<Vfs>,
        bytes: &Vec<u8>,
        filename: &str,
        basedir: &str,
    ) {
        vfs.add_directory_recursive(basedir);
        vfs.change_dir(basedir);
        vfs.touch(filename);

        let package_dirpath = utils::fmt_package_path(basedir, filename);
        vfs.write_file(
            filename,
            Some(bytes.clone()),
            Some(package_dirpath.as_str()),
        );

        vfs.change_dir("/");
    }

    pub fn deflate_package(&self, bytes: &Vec<u8>, filename: &str) -> Option<Vec<u8>> {
        let filename_splitted = filename.split(".").collect::<Vec<&str>>();
        let ext = format!(
            ".{}.{}",
            filename_splitted[filename_splitted.len() - 2],
            filename_splitted[filename_splitted.len() - 1]
        );

        println!("Deflating package {}", filename);
        match utils::deflate_tar(bytes.clone(), ext.as_str()) {
            Ok(bytes) => {
                println!("Deflated successfully");
                return Some(bytes);
            }
            Err(e) => {
                println!("Error deflate file: {}", e);
                return None;
            }
        };
    }

    pub fn install(&mut self, vfs: RwLockWriteGuard<Vfs>, flate_bytes: &Vec<u8>, filename: &str, version: &str) {
        println!("Start installing package {}", filename);
        match self.deflate_package(flate_bytes, filename) {
            Some(deflate_bytes) => {
                let package_name = filename
                    .strip_suffix(TarArchive::Gz.to_string().as_str())
                    .or_else(|| filename.strip_suffix(TarArchive::Xz.to_string().as_str()))
                    .or_else(|| filename.strip_suffix(TarArchive::Bz2.to_string().as_str()))
                    .or_else(|| filename.strip_suffix(TarArchive::Lz.to_string().as_str()))
                    .unwrap();
                utils::to_archive(&deflate_bytes);
                self.write_downloaded_file(vfs, &deflate_bytes, package_name, "/bin");
                
                let package_name_splitted = package_name.split("-").next();
                let package_name_formatted = match package_name_splitted {
                    Some(name) => name,
                    None => package_name
                };

                self.packages.insert(package_name_formatted.to_string(), Package {
                    dependencies: vec![],
                    name: package_name_formatted.to_string(),
                    path: utils::fmt_package_path(KPM_BIN, package_name),
                    size: deflate_bytes.len() as u64,
                    version: version.to_string()
                });
            }
            None => {
                println!("Error installing package {}", filename);
            }
        }
    }

    pub fn list(&self) {
        for (name, package) in self.packages.iter() {
            println!("Name: {} Version: {} Path: {}", name, package.version, package.path);
        }
    }

    pub async fn execute<'a>(&self, mut vfs: RwLockWriteGuard<'a, Vfs>, package_name: &str) {
        self.packages.iter().for_each(|(name, package)| {
            println!("Name: {} Version: {} Path: {}", name, package.version, package.path);
        });
        let package = self.packages.get(package_name);
        if package.is_none() {
            println!("Package not found");
            return;
        }

        let package = package.unwrap();
        vfs.change_dir("/bin");

        let filename = package.path.split("/").last().unwrap();
        vfs.execute_file(filename).await;

        vfs.change_dir("/");
    }
}
