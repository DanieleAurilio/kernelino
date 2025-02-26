use tokio;
mod editor;
mod kpm;
mod shell;
mod utils;
mod vfs;
mod vmm;
mod vpm;
mod libx;
mod elf;

#[tokio::main]
async fn main() {
    shell::run().await;
}
