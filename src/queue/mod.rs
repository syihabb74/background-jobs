use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{app_state::AppState, email::Email};

#[derive(Debug)]
pub struct Queue {
    pub queue: VecDeque<Email>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn add_queue(&mut self, email: Email) {
        self.queue.push_back(email);
        println!("{:?}", self.queue)
    }

    pub fn get_total_work(&self) -> usize {
        self.queue.len()
    }

    pub fn remove_queue(&mut self) -> Option<Email> {
        let email = self.queue.pop_front();
        email
    }

    pub fn dedicated_thread(
        queue: Arc<(Mutex<Queue>, Condvar)>,
        rx: Receiver<Email>,
        state_app: Arc<Mutex<AppState>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(email) => {
                        let (lock, cvar) = &*queue;
                        let mut lock = lock.lock().unwrap();
                        lock.add_queue(email);
                        let mut state_app_lock = state_app.lock().unwrap();
                        state_app_lock.increase_task();
                        drop(state_app_lock);
                        cvar.notify_all();
                    }
                    _ => {}
                }
            }
        })
    }
}
