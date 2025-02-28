/**
 * Kernelino Package Manager
 *
 */
use std::{collections::HashMap, io::Read, sync::RwLockWriteGuard};

use serde_json::Value;

use crate::{
    utils::{self, TarArchive},
    vfs::Vfs,
};

pub struct Kpm {
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
const KPM_BIN: &str = "bin";

impl Kpm {
    pub fn init() -> Self {
        if !utils::is_make_installed() {
            println!("Make is not installed");
            std::process::exit(1);
        }
        let packages = HashMap::new();

        Kpm { packages }
    }

    pub async fn download(
        &self,
        package_name: &str,
    ) -> (Option<Vec<u8>>, Option<String>, Option<String>) {
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

        let original_url = match stable.unwrap().get("url") {
            Some(url) => Some(url.as_str().unwrap()).unwrap(),
            None => {
                println!("Stable version url to download not found");
                return (None, None, None);
            }
        };

        if utils::is_package_lz(original_url) {
            println!("Package is lz, not supported. Try to download package in .tar.gz format");
        }

        let replaced_url = utils::replace_lz(original_url);
        let url = replaced_url.as_str();

        println!("Downloading package from {}", url);

        let filename = url.split("/").last().unwrap();
        let pkg = utils::http_async_get(url, None, true).await;
        if pkg.is_none() {
            println!("Error downloading package");
            return (None, None, None);
        } else {
            println!("Package downloaded successfully");
        }

        (
            Some(pkg.unwrap()),
            Some(filename.to_string()),
            Some(stable_version.unwrap().to_string()),
        )
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

    pub fn install(
        &mut self,
        mut vfs: RwLockWriteGuard<Vfs>,
        flate_bytes: &Vec<u8>,
        filename: &str,
        version: &str,
    ) {
        println!("Start installing package {}", filename);
        let deflate_bytes = match self.deflate_package(flate_bytes, filename) {
            Some(deflate_bytes) => deflate_bytes,
            None => {
                println!("Error deflating package {}", filename);
                return;
            }
        };

        let package_name = filename
            .strip_suffix(TarArchive::Gz.to_string().as_str())
            .or_else(|| filename.strip_suffix(TarArchive::Xz.to_string().as_str()))
            .or_else(|| filename.strip_suffix(TarArchive::Bz2.to_string().as_str()))
            .unwrap();

        let mut archive = utils::to_archive(&deflate_bytes);

        let entries = match archive.entries() {
            Ok(entries) => entries,
            Err(e) => {
                println!("Error reading entries: {}", e);
                return;
            }
        };

        for entry in entries {
            let mut entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    println!("Error reading entry: {}", e);
                    return;
                }
            };

            let entry_ref = entry.by_ref();
            let entry_heder = entry_ref.header();
            let is_dir = entry_heder.entry_type().is_dir();

            if is_dir {
                let path = entry_ref.path().unwrap();
                let full_path = format!("{}/{}", KPM_BIN, path.to_str().unwrap());
                vfs.add_directory_recursive(&full_path);
                continue;
            }

            let bytes: Vec<u8> = entry_ref.bytes().filter_map(Result::ok).collect();
            let path = entry_ref.path().unwrap();
            let file_name: &str = path.file_name().unwrap().to_str().unwrap();
            let mut path_split = path.to_str().unwrap().split(file_name);
            let dir_path = path_split.next().unwrap();

            let dir_path_formatted = format!("{}/{}", KPM_BIN, dir_path);

            vfs.write_downloaded_file(&bytes, file_name, &dir_path_formatted);
        }

        let package_name_splitted = package_name.split("-").next();
        let package_name_formatted = match package_name_splitted {
            Some(name) => name,
            None => package_name,
        };

        self.packages.insert(
            package_name_formatted.to_string(),
            Package {
                dependencies: vec![],
                name: package_name_formatted.to_string(),
                path: utils::fmt_package_path(KPM_BIN, package_name),
                size: deflate_bytes.len() as u64,
                version: version.to_string(),
            },
        );
    }

    pub fn list(&self) {
        for (name, package) in self.packages.iter() {
            println!(
                "Name: {} Version: {} Path: {}",
                name, package.version, package.path
            );
        }
    }

    pub async fn execute<'a>(&self, mut vfs: RwLockWriteGuard<'a, Vfs>, package_name: &str) {
        self.packages.iter().for_each(|(name, package)| {
            println!(
                "Name: {} Version: {} Path: {}",
                name, package.version, package.path
            );
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
