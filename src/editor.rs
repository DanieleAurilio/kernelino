use std::io;
/**
 * Minimal file editor
 * "wq" -> save and quit
 * "q" -> quit
 */
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::vfs::File;
use crate::vmm::Vmm;

#[warn(dead_code)]
pub struct Editor {}

impl Editor {
    pub fn write(file_mux: Arc<Mutex<File>>, vmm: Arc<Mutex<Vmm>>) {
        let mut vmm_mutex = vmm.lock().unwrap();
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

            match input.trim() {
                "wq" => {
                    new_content = buffer.as_bytes().to_vec().clone();
                    file.size = new_content.len() as u64;
                    file.vmm_address = vmm_mutex.allocate_bytes(new_content);
                    println!("File saved successfully!");
                    return;
                }
                "q" => {
                    println!("Exit without save");
                    return;
                }
                _ => {
                    buffer.push_str(&input);
                    new_content.extend_from_slice(input.as_bytes());
                    new_content.push(b'\n');
                }
            }
        }
    }

    pub fn read(file_mux: Arc<Mutex<File>>, vmm: Arc<Mutex<Vmm>>) {
        let file = file_mux.lock().unwrap();
        let vmm_mutex = vmm.lock().unwrap();
        let content = vmm_mutex.get_bytes(file.vmm_address.clone(), file.size);
        if content.is_empty() {
            //println!("File is empty");
            return;
        }
        let content = String::from_utf8(content).unwrap();
        content
            .lines()
            .filter(|line| !line.is_empty())
            .for_each(|line| {
                println!("{}", line);
            });
    }
}
