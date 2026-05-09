use std::sync::atomic::Ordering::Relaxed;
use std::{
    io::{ErrorKind, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

use crate::{WILL_SHUTDOWN, email::Email};

#[derive(Debug)]
pub struct UnixServer {
    path: String,
    listener: Option<UnixListener>,
}

impl UnixServer {
    pub fn build(path: String) -> Self {
        Self {
            path,
            listener: None,
        }
    }

    pub fn deploy_uds(&mut self) -> Result<(), String> {
        if !self.path.contains(".sock") {
            return Err(String::from("Invalid format file it should be .sock"));
        }

        let _ = std::fs::remove_file(&self.path);

        match UnixListener::bind(&self.path) {
            Ok(uds) => {
                println!("Unix domain socket already listener on file {}", self.path);
                self.listener = Some(uds);
                Ok(())
            }
            Err(e) => match e.kind() {
                ErrorKind::AddrInUse => Err("File path being use".to_string()),
                ErrorKind::AlreadyExists => {
                    Err("Enum already exist file path being use".to_string())
                }
                _ => Err("Unknown error".to_string()),
            },
        }
    }

    pub fn listening(&mut self, tx: Sender<Email>) {
        let unix_listener = self.listener.as_ref();
        match unix_listener {
            Some(listener) => {
                let non_blocking = listener.set_nonblocking(true);
                if let Err(e) = non_blocking {
                    println!("{}", e);
                }
                loop {
                    let sender = tx.clone();
                    let connection = listener.accept();
                    match connection {
                        Ok((mut stream, _)) => {
                            if WILL_SHUTDOWN.load(Relaxed) {
                                let _ = stream.write_all(
                                    b"Connected successfully but server will be shutdown",
                                );
                                let _ = stream.flush().ok();
                                break;
                            }
                            thread::spawn(move || {
                                handle_client(stream, sender);
                            });
                        }
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {
                            if WILL_SHUTDOWN.load(Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(20));
                        }
                        _ => {
                            println!("Unexpected error happening at UDS")
                        }
                    }
                }
                println!("Successfully exit in uds")
            }
            _ => {
                println!("No connection");
            }
        }
    }

    fn disconnected(&mut self) {
        if let Some(_listener) = self.listener.take() {
            println!("Listener closed");
        }

        match std::fs::remove_file(&self.path) {
            Ok(_) => {
                println!("File deleted sucessfully")
            }
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    println!("File not found")
                }
                _ => {
                    println!("Another error")
                }
            },
        }
    }
}

impl Drop for UnixServer {
    fn drop(&mut self) {
        println!("Shutdown...");
        self.disconnected();
    }
}

fn handle_client(mut stream: UnixStream, sender: Sender<Email>) {
    let mut buffer = [0u8; 1024];
    loop {
        if WILL_SHUTDOWN.load(Relaxed) {
            stream.write_all(b"Server will be shutdown").unwrap();
            break;
        }
        match stream.read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                if WILL_SHUTDOWN.load(std::sync::atomic::Ordering::Relaxed) {
                    stream.write_all(b"Server will shutdown").ok();
                    stream.flush().ok();
                } else {
                    if let Ok(email) = Email::to_struct_single(&mut buffer, n) {
                        sender.send(email).unwrap();
                    } else if let Ok(vec_email) = Email::to_struct_batches(&mut buffer, n) {
                        for email in vec_email.into_iter() {
                            sender.send(email).unwrap()
                        }
                    };

                    stream
                        .write_all(b"OK: Email received processing background jobs\n")
                        .ok();
                    stream.flush().ok();
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
                if WILL_SHUTDOWN.load(Relaxed) {
                    println!("WIll break");
                    break;
                }
            }
            Err(e) => {
                println!(
                    "Unexpected Error happenned at handle_client fn {:#?}",
                    e.kind()
                )
            }
        }
    }
}
