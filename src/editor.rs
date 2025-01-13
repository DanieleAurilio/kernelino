use std::io;
/**
 * Minimal file editor
 * "wq" -> save and quit
 * "q" -> quit
 */
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::vfs::File;

#[warn(dead_code)]
pub struct Editor {}

impl Editor {
    pub fn write(file_mux: Arc<Mutex<File>>) {
        let mut file = file_mux.lock().unwrap();
        let mut new_content = Vec::<u8>::new();
        let mut buffer = String::new();
        print!(
            "Welcome to the editor! Type 'wq' to save and quit or 'q' to quit without saving.\n"
        );
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            buffer.push_str(&input);

            match input.trim() {
                "wq" => {
                    new_content = buffer.as_bytes().to_vec();
                    file.content = new_content.clone();
                    file.size = file.content.len() as u64;
                    println!("File saved successfully!");
                    return;
                }
                "q" => {
                    println!("Exit without save");
                    return;
                }
                _ => {
                    new_content.extend_from_slice(input.as_bytes());
                    new_content.push(b'\n');
                }
            }
        }
    }
}
