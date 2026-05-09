use crate::email::Email;
use crate::{WILL_SHUTDOWN, app_state::AppState, queue::Queue};
use colored::Colorize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::Sender;
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

pub struct WorkerQueue {
    _no: usize,
    _worker: JoinHandle<()>,
    _sender_email: Arc<Sender<Email>>,
}

impl WorkerQueue {
    pub fn new(
        no: usize,
        queue: Arc<(Mutex<Queue>, Condvar)>,
        app_state: Arc<Mutex<AppState>>,
        sender: Arc<Sender<Email>>,
    ) -> Self {
        let tx_clone = Arc::clone(&sender);

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

                if let Some(email) = job {
                    let mut app_state_lock = app_state.lock().unwrap();
                    app_state_lock.decrease_task();
                    tx_clone.send(email).unwrap();
                    println!(
                        "{}",
                        format!("Jumlah Task {}", app_state_lock.total_task).red()
                    );
                    drop(app_state_lock);

                    println!("{}", format!("Worker {} memproses", no).red());
                }
            }
        });

        let _no = no;
        let _worker = thread;
        let _sender_email = sender;

        Self {
            _no,
            _worker,
            _sender_email,
        }
    }


    pub fn return_thread(self) -> JoinHandle<()> {
        self._worker
    }

}

