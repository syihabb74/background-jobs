use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use crate::{app_state::AppState, queue::Queue};

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

                while guard.queue.is_empty() {
                    guard = cvar.wait(guard).unwrap();
                }

                let job = guard.remove_queue();
                println!("thread no {} {:?}", no, guard.queue);

                let mut app_state_lock = app_state.lock().unwrap();
                app_state_lock.decrease_task();
                drop(app_state_lock);
                drop(guard);

                if let Some(job) = job {
                    println!("Worker {} memproses job: {:?}", no, job);
                }
            }
        });

        Self { no, thread }
    }
}
