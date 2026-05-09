use std::{
    ops::Deref, sync::{Arc, mpsc::Receiver}, thread::{self, JoinHandle}
};

use crate::{email::Email, smtp::smtp_config::SmtpConfig};

pub struct WorkerSmtp {
    _no: usize,
    _worker: JoinHandle<()>,
}

impl WorkerSmtp {
    pub fn new(
        no: usize,
        receiver: Receiver<Email>,
        smtp_config: Arc<SmtpConfig>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let smtp_server = smtp_config.connect()?;
        let mut smtp_server_upgrade = smtp_server.upgrade_tls().unwrap();
        smtp_server_upgrade.login(smtp_config)?;

        let _worker = thread::spawn(move || {
            for email in receiver {
                let sending = smtp_server_upgrade.send_email("Syihabb74@gmail.com", email);
                if let Err(e) = sending {
                    println!("{:?}", e);
                    panic!()
                }
            }
        });

        Ok(Self { _no: no, _worker })
    }

    pub fn return_thread (self) -> JoinHandle<()> {
        self._worker
    }

}

