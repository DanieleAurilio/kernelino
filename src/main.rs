use tokio;
mod editor;
mod shell;
mod utils;
mod vfs;
mod vmm;
mod vpm;
mod lua;

#[tokio::main]
async fn main() {
    shell::run().await;
}
