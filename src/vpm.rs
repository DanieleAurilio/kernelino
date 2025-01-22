/**
 * Virtual Process Manager
 */
use crate::vmm::Vmm;
use std::sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex};

static NEXT_PID: AtomicU32 = AtomicU32::new(1);
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Vpm {
    pub pid: u32,
    pub vmm: Arc<Mutex<Vmm>>,
}

impl Vpm {
    pub fn new(vmm: Arc<Mutex<Vmm>>) -> Self {
        Self {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            vmm,
        }
    }

    pub fn fork(&mut self) -> Vpm {
        let main_process = self.clone();
        Vpm::new(main_process.vmm)
    }

    pub fn execute<F>(&mut self, func: F)
    where
        F: FnOnce(&Self),
    {
        let child_process = self.fork();
        func(&child_process);
    }
}
