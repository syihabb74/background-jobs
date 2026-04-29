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
        cmd: &[u8],
        closure : Option<Closure>
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.write_cmd(cmd)?;
        self.read_response(closure.as_ref())?;
        Ok(())
    }

    pub fn read_response(
        &mut self, 
        closure : Option<&Closure>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(&mut self.stream);
          loop {
            let mut response = String::new();
            match reader.read_line(&mut response) {
                Ok(0) => break,
                Ok(_) => {
                    if let Some(closure) = closure {
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
        &mut self,
        cmd: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sending = self.stream.write_all(cmd)?;
        Ok(sending)
    }

    pub fn authenticating (
        &mut self,
        config : Arc<SmtpConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        if let Err(e) = self.communicating( b"EHLO \r\n", None) {
            println!("Error occured cause {:?}", e);
            return Err(e)
        }

        Ok(())


    }

    pub fn upgrade_tls(
        mut self,
        host: &str,
        buff_reader : BufReader<&mut T>
    ) -> Result<LiveSmtp<StreamOwned<ClientConnection, T>>, Box<dyn std::error::Error>> {

        let _ = self.communicating(b"STARTTLS \r\n", None);

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



// todo 
// check tls supported
// check starttls ready
// do auth
// connect smtp
// ready
// make 4 thread
// implement all of this to each thread