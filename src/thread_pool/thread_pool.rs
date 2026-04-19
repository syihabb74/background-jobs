use std::{os::macos::raw::stat, sync::{Arc, Condvar, Mutex}};

use crate::{app_state::AppState, queue::Queue, thread_pool::worker::Worker};

pub struct ThreadPool {
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(
        size: usize,
        queue_state: Arc<(Mutex<Queue>, Condvar)>,
        app_state: Arc<Mutex<AppState>>,
    ) -> Self {
        let mut workers_vec = Vec::with_capacity(size);
        for i in 1..=size {
            let state_app = Arc::clone(&app_state);
            let state_thread = Arc::clone(&queue_state);
            let worker = Worker::new(i, state_thread, state_app);
            workers_vec.push(worker);
        }
        Self {
            workers: workers_vec,
        }
    }
}
