use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
};

use crate::{app_state::AppState, email::Email};

use colored::Colorize;

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
        println!("{:#?}", self.queue)
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

            while let Ok(email) = rx.recv() {
                println!("{}", "Receiving".green());
                let (lock, cvar) = &*queue;
                let mut lock = lock.lock().unwrap();
                println!("{}", "Adding".green());
                lock.add_queue(email);
                let mut state_app_lock = state_app.lock().unwrap();
                println!("{}", format!("Jumlah Task {}", state_app_lock.total_task).green());
                state_app_lock.increase_task();
                drop(lock);
                drop(state_app_lock);
                cvar.notify_one();
            }

            println!("Queue clean up after SIGINT or SIGTERM signal Received")
            
        })
    }
}
