/**
 * Virtual Process Manager
 *
 * Keeps the process table. For now the only process is the shell;
 * Lua scripts will run here as scheduled processes.
 */
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    started: Instant,
}

impl Process {
    pub fn uptime(&self) -> Duration {
        self.started.elapsed()
    }
}

#[derive(Debug)]
pub struct Vpm {
    processes: Vec<Process>,
}

impl Vpm {
    pub fn new() -> Self {
        Self {
            processes: vec![Process {
                pid: 1,
                name: String::from("shell"),
                started: Instant::now(),
            }],
        }
    }

    pub fn processes(&self) -> &[Process] {
        &self.processes
    }
}
