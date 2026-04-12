use std::{sync::mpsc, thread};

use background_jobs::{
    app_state::{self, AppState},
    email::Email,
    uds::UnixServer,
};

fn main() {
    let mut state_app = AppState::default();
    let (tx, rx) = mpsc::channel();

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
    let server = thread::spawn(move || {
        server.listening(tx);
    });

    loop {
        if let Ok(email) = rx.recv() {
            state_app.add_work(email);
        }
        // break;
    }

    server.join().unwrap()


}
