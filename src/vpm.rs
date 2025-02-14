/**
 * Virtual Process Manager
 */
use crossterm::{
    cursor::MoveTo,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};

use crate::vmm::Vmm;
use core::time;
use std::{
    io::stdout,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

static NEXT_PID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Vpm {
    pub pid: u32,
    pub vmm: Arc<Mutex<Vmm>>,
    time: Arc<Mutex<u64>>,
    children: Vec<Vpm>,
}

impl Vpm {
    pub fn new(vmm: Arc<Mutex<Vmm>>) -> Self {
        let time = Arc::new(Mutex::new(0));
        let process = Self {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            vmm,
            time: Arc::clone(&time),
            children: Vec::new(),
        };

        Vpm::start_time(Arc::clone(&time));
        process
    }

    fn start_time(time: Arc<Mutex<u64>>) {
        std::thread::spawn(move || loop {
            let mut time_mutex = time.lock().unwrap();
            *time_mutex += 1;
            drop(time_mutex);
            std::thread::sleep(Duration::from_secs(1));
        });
    }

    pub fn fork(&mut self) -> Vpm {
        let main_process = self.clone();
        Vpm::new(main_process.vmm)
    }

    #[allow(dead_code)]
    pub fn execute_child<F>(&mut self, func: F)
    where
        F: FnOnce(&Self) + Send + 'static,
    {
        let child_process = self.fork();
        self.children.push(child_process.clone());
        std::thread::spawn(move || {
            func(&child_process);
        });
    }

    pub fn execute<F>(&mut self, func: F)
    where
        F: FnOnce(&Self),
    {
        let child_process = self.fork();
        func(&child_process);
    }

    pub fn show_processes(&self) {
        loop {
            enable_raw_mode().unwrap();

            // Capture the event
            let exist_event = crossterm::event::poll(time::Duration::from_millis(100)).unwrap();
            if exist_event {
                let event: crossterm::event::Event = crossterm::event::read().unwrap();
                match event {
                    crossterm::event::Event::Key(key_event) => {
                        if key_event.code == crossterm::event::KeyCode::Esc {
                            disable_raw_mode().unwrap();
                            break;
                        }
                    }
                    _ => {}
                }
            }

            disable_raw_mode().unwrap();

            // Clear the screen
            stdout().execute(Clear(ClearType::All)).unwrap();
            stdout().execute(MoveTo(0, 0)).unwrap();

            let time = self.time.lock().unwrap();
            println!("Press ESC to stop showing processes");
            println!("PID: {} TIME: {}", self.pid, time);
            self.children.iter().for_each(|child| {
                println!("PID: {} Time: {}", child.pid, time);
            });
        }
    }
}
