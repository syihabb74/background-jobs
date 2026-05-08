use std::{io, net::TcpStream};

use crate::{
    cli::prompt,
    smtp::{
        auth_mechanism::AuthMechanism, smtp_credential::SmtpCredential, smtp_server::LiveSmtp,
        tcp_com::read_response,
    },
};

#[derive(Debug)]
pub struct SmtpConfig {
    pub auth_mechanism: Option<AuthMechanism>,
    pub credentials: Option<SmtpCredential>,
    pub host: String,
}

impl SmtpConfig {
    pub fn new() -> Self {
        let mut host = String::new();
        let _ = prompt(
            "Insert your relay here \nExample smtp.gmail.com:587",
            &mut host,
        );
        Self {
            host,
            credentials: None,
            auth_mechanism: None,
        }
    }

    pub fn connect(&self) -> Result<LiveSmtp<TcpStream>, io::Error> {
        let mut stream = TcpStream::connect(&self.host)?;
        let mut banner = Vec::new();
        let _ = read_response(&mut stream, None, &mut banner);
        let server_name = self.host_name();
        Ok(LiveSmtp {
            stream,
            server_name,
        })
    }

    fn host_name(&self) -> String {
        let host_name = self
            .host
            .split_once(":")
            .map(|(host, _)| host.to_string())
            .unwrap();
        println!("{}", host_name);
        return host_name;
    }

    // fn port(&self) -> String {
    //     let port = self.host_name()
    //     .split_once(":")
    //     .map(|(_, port)| {
    //         port.to_string()
    //     })
    //     .unwrap();
    //     return port
    // }

    pub fn set_auth_mech(&mut self, auth_mech: AuthMechanism) {
        self.auth_mechanism = Some(auth_mech)
    }

    pub fn set_smtp_credentials(&mut self, smtp_credentials: SmtpCredential) {
        self.credentials = Some(smtp_credentials)
    }
}
