use std::{sync::{Arc, Mutex, mpsc::{self, Receiver}}, thread::{self, JoinHandle}, time::Duration};

use crate::{WILL_SHUTDOWN, app_state::AppState, email::Email};



pub fn worker (receiver : Receiver<Email>, app_state : Arc<Mutex<AppState>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(email) => {
                    let state_clone = Arc::clone(&app_state);
                    thread::spawn(move || {
                        {
                            let mut state = state_clone.lock().unwrap();
                            state.enqueue(email);
                        }
                        thread::sleep(Duration::from_millis(3000));
                        {
                            let mut state = state_clone.lock().unwrap();
                            state.dequeue();
                        }
                    });
                }
                Err(_) => {
                    println!("Nyangkut di worker")
                }
            }

            if WILL_SHUTDOWN.load(std::sync::atomic::Ordering::Relaxed) {
                let state = app_state.lock().unwrap();
                if state.total_works == 0 && !state.has_works {
                    break;
                }
            }
        }
    })
}

