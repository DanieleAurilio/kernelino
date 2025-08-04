use crate::vfs::{self, Vfs};

#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    Number(i64),
    StringLiteral(String),
    
    Assign,
    Plus,
    Eof
}


pub struct Lua;

impl Lua {
    pub fn new () -> Self {
        println!("Lua interpreter initialized");
        Self {}
    }

    pub fn run(&self, vfs: &Vfs,  filepath: &str) {
        let root = vfs.root.clone();
        let file_mux = match Vfs::search_file_recursive(vfs, filepath,&root) {
            Some(file_mux) => file_mux,
            None => {
                println!("File not found: {}", filepath);
                return;
            }
        };

        let file = file_mux.lock().unwrap();
        let file_clone = &file.clone();
        drop(file);
        
        match Vfs::read_file_content_bytes_to_utf8(vfs,file_clone) {
            Some(content) => println!("File content {}", content),
            None => {
                return;
            }
        }
    }

}

