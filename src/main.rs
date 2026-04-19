use std::sync::{Arc, Condvar, Mutex, mpsc};

use background_jobs::{app_state::AppState, queue::Queue, signaling, thread_pool::{self}, uds::UnixServer};

fn main() {

    let graceful_shutdown = signaling::graceful_shutdown();
    let state_app = Arc::new(Mutex::new(AppState::new()));
    let queue = Arc::new((Mutex::new(Queue::new()), Condvar::new()));


    let (tx, rx) = mpsc::channel();
    let cloned_stated_app = Arc::clone(&state_app);
    let queue_clone = Arc::clone(&queue);
    let queue_clone_rx = Arc::clone(&queue);
    let state_app_rx = Arc::clone(&state_app);
    
    let mut server = UnixServer::build(String::from("/tmp/server_bg_jobs.sock"));
    let run = server.deploy_uds();
    match run {
        Ok(_) => {
            println!("Running");
        }
        Err(e) => {
            println!("{}", e)
        }
    }
    server.listening(tx);
    let dedicated_thread = Queue::dedicated_thread(queue_clone, rx, cloned_stated_app);
    let x = thread_pool::thread_pool::ThreadPool::new(4, queue_clone_rx, state_app_rx);

    graceful_shutdown.join().unwrap();
    dedicated_thread.join().unwrap();
}
