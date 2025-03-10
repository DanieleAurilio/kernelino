use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use reqwest::{Client, Method};
use core::fmt;
use std::{
    collections::HashMap,
    env,
    io::{stdout, Error, Read},
};

pub fn is_unix_symbol(s: &str) -> bool {
    const PROTECTED_SYMBOL: [&str; 3] = ["/", ".", ".."];
    PROTECTED_SYMBOL.contains(&s)
}

pub enum TarArchive {
    Gz,
    Xz,
    Bz2
}

impl fmt::Display for TarArchive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TarArchive::Gz => write!(f, ".tar.gz"),
            TarArchive::Xz => write!(f, ".tar.xz"),
            TarArchive::Bz2 => write!(f, ".tar.bz2"),
        }
    }
}

pub fn clear_terminal() {
    stdout().execute(Clear(ClearType::All)).unwrap();
    stdout().execute(MoveTo(0, 0)).unwrap();
}

pub async fn http_async_get(
    url: &str,
    params: Option<HashMap<String, String>>,
    print_download_percentage: bool,
) -> Option<Vec<u8>> {
    let mut request = Client::new().request(Method::GET, url);
    if params.is_some() {
        request = request.query(&params.unwrap());
    }

    let mut response = match request.send().await {
        Ok(response) => response,
        Err(e) => {
            println!("Error: {}", e);
            return None;
        }
    };

    if response.status().is_client_error() || response.status().is_server_error() {
        println!("Url: {}, Error: {}", url, response.text().await.unwrap());
        return None;
    } else {
        let content_size = response.content_length().unwrap();
        let mut bytes_stream: Vec<u8> = Vec::new();
        while let Some(chunk) = response.chunk().await.unwrap() {
            bytes_stream.append(&mut chunk.to_vec());

            if print_download_percentage {
                let total_bytes_downloaded =
                    (bytes_stream.len() as f64 / content_size as f64) * 100.0;
                print!("Downloaded: {:.2}%\r", total_bytes_downloaded);
            }
        }

        return Some(bytes_stream);
    }
}

pub fn is_unix() -> Option<String> {
    match env::consts::OS {
        "linux" => return Some("linux".to_string()),
        "macos" => return Some("macos".to_string()),
        _ => {
            println!("KPM Error: Unsupported OS");
            return None;
        }
    }
}

pub fn deflate_tar(flate_bytes: Vec<u8>, ext: &str) -> Result<Vec<u8>, Error> {
    match ext {
        ".tar.gz" => defalte_gz(flate_bytes),
        ".tar.xz" => deflate_xz(flate_bytes),
        ".tar.bz2" => deflate_bz2(flate_bytes),
        _ => {
            println!("Error: Unsupported extension");
            Err(Error::from(std::io::ErrorKind::InvalidInput))
        }
    }
}

fn defalte_gz(flate_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut decoder = flate2::read::GzDecoder::new(&flate_bytes[..]);
    let mut buffer: Vec<u8> = Vec::new();
    match decoder.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}

fn deflate_xz(flate_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut decoder = xz::read::XzDecoder::new(&flate_bytes[..]);
    let mut buffer: Vec<u8> = Vec::new();
    match decoder.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}

fn deflate_bz2(flate_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut decoder = bzip2::read::BzDecoder::new(&flate_bytes[..]);
    let mut buffer: Vec<u8> = Vec::new();
    match decoder.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => {
            println!("Error: {}", e);
            Err(e)
        }
    }
}

pub fn fmt_package_path(basedir: &str, package_name: &str) -> String {
    if package_name.contains(".tar") {
        return format!("{}/{}", basedir, package_name.split(".tar").next().unwrap());
    }
    format!("{}/{}", basedir, package_name)
}