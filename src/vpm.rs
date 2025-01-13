/**
 * Virtual Process Manager
 */
use crate::vmm::Vmm;
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_PID: AtomicU32 = AtomicU32::new(1);
#[derive(Debug, Clone)]
pub struct Vpm {
    pid: u32,
    vmm: Vmm,
}

impl Vpm {
    pub fn new(vmm: Vmm) -> Self {
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
