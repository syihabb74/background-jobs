use crate::{WILL_SHUTDOWN, app_state::AppState, queue::Queue};
use colored::Colorize;
use std::sync::atomic::Ordering::Relaxed;
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Worker {
    no: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(
        no: usize,
        queue: Arc<(Mutex<Queue>, Condvar)>,
        app_state: Arc<Mutex<AppState>>,
    ) -> Self {
        let thread = thread::spawn(move || {
            let (lock, cvar) = &*queue;

            loop {
                let mut guard = lock.lock().unwrap();

                while guard.queue.is_empty() && !WILL_SHUTDOWN.load(Relaxed) {
                    guard = cvar.wait(guard).unwrap();
                }

                if guard.queue.is_empty() && WILL_SHUTDOWN.load(Relaxed) {
                    break;
                }

                let job = guard.remove_queue();

                drop(guard);
                
                if let Some(_) = job {
                    let mut app_state_lock = app_state.lock().unwrap();
                    app_state_lock.decrease_task();
                    println!(
                        "{}",
                        format!("Jumlah Task {}", app_state_lock.total_task).red()
                    );
                    drop(app_state_lock);
                    println!("{}", format!("Worker {} memproses", no).red());
                }
            }

            println!("Queue clean up after SIGINT or SIGTERM signal Received")

        });

        Self { no, thread }
    }
}
