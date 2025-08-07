use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use reqwest::{Client, Method};
use std::{
    collections::HashMap,
    io::{stdout},
};

pub fn is_unix_symbol(s: &str) -> bool {
    const PROTECTED_SYMBOL: [&str; 3] = ["/", ".", ".."];
    PROTECTED_SYMBOL.contains(&s)
}


pub fn clear_terminal() {
    stdout().execute(Clear(ClearType::All)).unwrap();
    stdout().execute(MoveTo(0, 0)).unwrap();
}

#[allow(dead_code)]
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