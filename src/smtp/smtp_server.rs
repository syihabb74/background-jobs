use std::{
    io::{Read, Write},
    sync::Arc,
};

use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;

use crate::{
    Closure,
    cli::{cli_auth_credentials, cli_auth_smtp},
    smtp::{
        auth_mechanism::AuthMechanism,
        smtp_config::SmtpConfig,
        smtp_credential::SmtpCredential,
        tcp_com::{read_response, write_cmd},
    },
};

#[derive(Debug)]
pub struct LiveSmtp<T: Read + Write> {
    pub stream: T,
    pub server_name: String,
}

impl<T: Read + Write> LiveSmtp<T> {
    pub fn communicating(
        &mut self,
        cmd: &[u8],
        closure: Option<&Closure>,
        response_result: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_cmd(&mut self.stream, cmd)?;
        read_response(&mut self.stream, closure, response_result)?;
        Ok(())
    }

    pub fn parse_auth(bucket_response: &Vec<String>) -> Vec<AuthMechanism> {
        let mut v = Vec::new();

        for line in bucket_response {
            if let Some(mechs) = line.trim().strip_prefix("250-AUTH ") {
                for mech in mechs.split_whitespace() {
                    let mechanism = AuthMechanism::new(mech);
                    v.push(mechanism);
                }
            }
        }

        v
    }

    pub fn check_auth_method(
        &mut self,
    ) -> Result<(AuthMechanism, SmtpCredential), Box<dyn std::error::Error>> {
        let closure: Option<Closure> = Some(Box::new(
            |response_result: &mut Vec<String>, response: String| {
                response_result.push(response);
            },
        ));

        let mut response_result: Vec<String> = Vec::new();
        let _ = self.communicating(
            b"EHLO mylocalhost\r\n",
            closure.as_ref(),
            &mut response_result,
        );
        let auth_mechs = Self::parse_auth(&response_result);
        let auth_mech = cli_auth_smtp(auth_mechs)?;
        let credentials = cli_auth_credentials(&auth_mech)?;
        return Ok((auth_mech, credentials));
    }

    pub fn login(
        &mut self,
        smtp_config: &Arc<SmtpConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let smtp_config_clone = Arc::clone(&*smtp_config);
        let mut response_result: Vec<String> = Vec::new();
        let credentials = smtp_config_clone.credentials.as_ref();
        let auth_mechanism = smtp_config_clone.auth_mechanism.as_ref();
        println!("{:?} {:?}", credentials, auth_mechanism);
        if credentials.is_some() && auth_mechanism.is_some() {
            let _ = self.communicating(b"EHLO mylocalhost\r\n", None, &mut response_result);
            response_result.clear();
            let c = credentials.unwrap();
            let a_m = auth_mechanism.unwrap();
            match a_m {
                AuthMechanism::Login => {
                    let format_login = format!("{}\r\n", a_m.auth_command().unwrap());
                    let _ =
                        self.communicating(format_login.as_bytes(), None, &mut response_result)?;
                    let encode = c.encode_auth(a_m)?;
                    let (email, password) = encode
                        .split_once(',')
                        .map(|(u, p)| (u.to_string(), p.to_string()))
                        .ok_or("Invalid encode format")?;
                    let _ = self.communicating(
                        format!("{}\r\n", email).as_bytes(),
                        None,
                        &mut response_result,
                    );
                    let _ = self.communicating(
                        format!("{}\r\n", password).as_bytes(),
                        None,
                        &mut response_result,
                    );
                }
                _ => {
                    let encoded = credentials.unwrap().encode_auth(a_m)?;
                    let auth_command = a_m.auth_command().unwrap();
                    let auth_format = format!("{auth_command} {encoded}\r\n");
                    self.communicating(auth_format.as_bytes(), None, &mut response_result)?;
                }
            }

            for response_server in response_result.iter() {
                if let Some(_) = response_server.trim().strip_prefix("535") {
                    return Err("Failed credentials".into());
                }

                if let Some(_) = response_server.trim().strip_prefix("235") {
                    println!("Login soccessfully");
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn upgrade_tls(
        mut self,
    ) -> Result<LiveSmtp<StreamOwned<ClientConnection, T>>, Box<dyn std::error::Error>> {
        let mut response_result: Vec<String> = Vec::new();
        let closure: Option<Closure> = Some(Box::new(
            |response_result: &mut Vec<String>, response: String| {
                response_result.push(response);
            },
        ));
        self.communicating(
            b"EHLO mylocalhost\r\n",
            closure.as_ref(),
            &mut response_result,
        )?;
        let is_tls_supported = response_result.iter().any(|response| {
            response.starts_with("250-STARTTLS") || response.starts_with("250 STARTTLS")
        });

        if !is_tls_supported {
            return Err("STARTTLS not supported".into());
        }

        self.communicating(b"STARTTLS\r\n", closure.as_ref(), &mut response_result)?;
        let is_tls_ready = &response_result[response_result.len() - 1];

        if !is_tls_ready.starts_with("220") {
            return Err("TLS is not ready".into());
        }

        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );

        let server_name = ServerName::try_from(self.server_name.clone())?.to_owned();

        let conn = ClientConnection::new(config, server_name)?;

        Ok(LiveSmtp {
            stream: StreamOwned::new(conn, self.stream),
            server_name: self.server_name,
        })
    }
}
