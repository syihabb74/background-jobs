use std::{
    sync::{Arc, Condvar, Mutex, mpsc},
    thread,
};

use background_jobs::{
    app_state::AppState,
    queue::Queue,
    signaling,
    smtp::smtp_config::SmtpConfig,
    thread_pool::{self},
    uds::UnixServer,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut smtp_config = SmtpConfig::new();
    let smtp_server = smtp_config.connect().unwrap();
    let mut tls_smtp_server = smtp_server.upgrade_tls()?;
    let (auth_mech, smtp_credential) = tls_smtp_server.check_auth_method()?;
    smtp_config.set_auth_mech(auth_mech);
    smtp_config.set_smtp_credentials(smtp_credential);
    let smtp_config = Arc::new(smtp_config);
    let smtp_config_clone_for_check = Arc::clone(&smtp_config);
    let smtp_config_clone_for_worker = Arc::clone(&smtp_config);
    tls_smtp_server.login(smtp_config_clone_for_check)?;

    let graceful_shutdown = signaling::graceful_shutdown();
    let state_app = Arc::new(Mutex::new(AppState::new()));
    let queue = Arc::new((Mutex::new(Queue::new()), Condvar::new()));

    let (tx, rx) = mpsc::channel();
    let cloned_stated_app = Arc::clone(&state_app);
    let queue_clone = Arc::clone(&queue);
    let queue_clone_rx = Arc::clone(&queue);
    let state_app_rx = Arc::clone(&state_app);

    let mut server = UnixServer::build(String::from("/tmp/server_bg_jobs.sock"));
    let _ = server.deploy_uds()?;
    thread::spawn(move || {
        server.listening(tx);
    });
    let dedicated_thread = Queue::dedicated_thread(queue_clone, rx, cloned_stated_app);
    let _thread_pool = thread_pool::thread_pool::ThreadPool::new(
        4,
        queue_clone_rx,
        state_app_rx,
        smtp_config_clone_for_worker,
    )?;

    graceful_shutdown.join().unwrap();
    dedicated_thread.join().unwrap();
    Ok(())
}
