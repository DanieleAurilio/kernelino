use tokio;
mod editor;
mod elf;
mod kpm;
mod libx;
mod shell;
mod utils;
mod vfs;
mod vmm;
mod vpm;

#[tokio::main]
async fn main() {
    shell::run().await;
}
