use crate::{lexer::Lexer, vfs::Vfs};

pub struct Lua;

impl Lua {
    pub fn new() -> Self {
        println!("Lua interpreter initialized");
        Self {}
    }

    pub fn run(&self, vfs: &Vfs, filepath: &str) {
        let root = vfs.root.clone();

        if !filepath.contains(".lua") {
            println!("File extension is not .lua, {}", filepath);
            return;
        }

        let file_mux = match Vfs::search_file_recursive(vfs, filepath, &root) {
            Some(file_mux) => file_mux,
            None => {
                println!("File not found: {}", filepath);
                return;
            }
        };

        let file = file_mux.lock().unwrap();
        let file_clone = &file.clone();
        drop(file);

        match Vfs::read_file_content_bytes_to_utf8(vfs, file_clone) {
            Some(content) => {
               let mut lexer =  Lexer::new(content);
               lexer.read_input();
            },
            None => {
                return;
            }
        }
    }
}
