use std::{
    io::{BufRead, BufReader, Read, Write},
    sync::Arc,
};

use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;

type Closure = Box<dyn 'static + Fn(&str) -> Result<(), Box<dyn std::error::Error>>>;

pub struct SmtpConfig {
    host: &'static str,
    username : String, 
    password : String
}

impl SmtpConfig {
    pub fn new(
    host: &'static str,
    username : String,
    password : String) -> Self {
        Self { host, username, password }
    }

    pub fn connect<T>(&self, stream: T) -> LiveSmtp<T>
    where
        T: Read + Write,
    {
        LiveSmtp { stream }
    }
}

pub struct LiveSmtp<T: Read + Write> {
    stream: T,
}

impl<T: Read + Write> LiveSmtp<T> {
    pub fn communicating(
        &mut self,
        buff_reader : &mut BufReader<&mut T>,
        cmd: &[u8],
        closure : Option<Closure>
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::write_cmd(&mut self.stream, cmd)?;
        loop {
            let mut response = String::new();
            match buff_reader.read_line(&mut response) {
                Ok(0) => break,
                Ok(_) => {
                    if let Some(ref closure) = closure {
                        closure(&response)?;
                    }
                    if response.len() >= 4 && response.chars().nth(3) == Some(' ') {
                        break;
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(())
    }

    pub fn write_cmd(
        stream: &mut impl Write,
        cmd: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sending = stream.write_all(cmd)?;
        Ok(sending)
    }

    pub fn authenticating (
        &mut self,
        config : Arc<SmtpConfig>,
    ) {
        
        if let Err(e) = Self::write_cmd(&mut self.stream, b"EHLO \r\n") {
            println!("Error occured cause {:?}", e)
        }

    }

    pub fn upgrade_tls(
        mut self,
        host: &str,
        buff_reader : BufReader<&mut T>
    ) -> Result<LiveSmtp<StreamOwned<ClientConnection, T>>, Box<dyn std::error::Error>> {
        let mut move_buff_reader = buff_reader;

        let _ = self.communicating(&mut move_buff_reader, b"STARTTLS \r\n", None);

        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        );

        let server_name = ServerName::try_from(host)?.to_owned();

        let conn = ClientConnection::new(config, server_name)?;

        Ok(LiveSmtp {
            stream: StreamOwned::new(conn, self.stream),
        })
    }
}
