use std::sync::{Arc, Condvar, Mutex, mpsc};

use crate::{
    app_state::AppState,
    email::Email,
    queue::Queue,
    smtp::smtp_config::SmtpConfig,
    thread_pool::{worker_queue::WorkerQueue, worker_smtp::WorkerSmtp},
};

pub struct ThreadPool {
    _workers_queue: Vec<WorkerQueue>,
    _workers_smtp: Vec<WorkerSmtp>,
}

impl ThreadPool {
    pub fn new(
        size: usize,
        queue_state: Arc<(Mutex<Queue>, Condvar)>,
        app_state: Arc<Mutex<AppState>>,
        smtp_config: Arc<SmtpConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if size < 1 {
            panic!("0 or negative is not allowed");
        }

        let mut workers_queue = Vec::with_capacity(size);
        let mut workers_smtp = Vec::with_capacity(size);
        for i in 1..=size {
            let smtp_config = Arc::clone(&smtp_config);
            let (tx, rx) = mpsc::channel::<Email>();
            let tx = Arc::new(tx);
            let state_app = Arc::clone(&app_state);
            let state_thread = Arc::clone(&queue_state);
            let worker_queue = WorkerQueue::new(i, state_thread, state_app, tx);
            let worker_smtp = WorkerSmtp::new(i, rx, smtp_config)?;
            workers_queue.push(worker_queue);
            workers_smtp.push(worker_smtp);
        }
        Ok(Self {
            _workers_queue: workers_queue,
            _workers_smtp: workers_smtp,
        })
    }
}


impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker_queue in self._workers_queue.drain(..) {
            worker_queue.return_thread().join().unwrap();
        }

        for worker_smtp in self._workers_smtp.drain(..) {
            worker_smtp.return_thread().join().unwrap();
        }

    }
}