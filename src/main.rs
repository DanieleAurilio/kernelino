use tokio;
mod editor;
mod shell;
mod utils;
mod vfs;
mod vmm;
mod vpm;
mod lua;
mod lexer;
mod ast;
mod parser;

#[tokio::main]
async fn main() {
    shell::run().await;
}
