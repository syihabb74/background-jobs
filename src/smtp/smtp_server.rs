use std::{
    io::{BufRead, BufReader, Read, Write},
    sync::Arc,
};

use base64::{Engine, engine::general_purpose};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;

use crate::cli::cli_smtp;

type Closure =
    Box<dyn 'static + Fn(&mut Vec<String>, String)>;

pub struct SmtpConfig {
    host: &'static str,
    credentials : SmtpCredential,
}

pub enum AuthMechanism {
    Plain,
    Login,
    XOAuth,
    XOAuth2,
    OAuthBearer,
    PlainClientToken,
    Unknown(String)
}

// impl AuthMechanism {
//     pub fn input_credential(&self) -> SmtpCredential {
//         match self {
//             AuthMechanism::Login |
//             AuthMechanism::Plain |
//             AuthMechanism::PlainClientToken => {

//             },
//             AuthMechanism::XOAuth |
//             AuthMechanism::XOAuth2 => {

//             },
//             AuthMechanism::OAuthBearer => {

//             },
//             AuthMechanism::Unknown(s) => {

//             }
//         }
//     }
// }

pub enum SmtpCredential {

    EmailPassword {
        email: String,
        password: String,
    },

   
    OAuth {
        email: String,
        access_token: String,
    },

    OAuthBearer {
        bearer_token : String
    }
}

impl SmtpCredential {
    pub fn new_email_password(email : String, password : String) -> Self {
        SmtpCredential::EmailPassword { email, password }
    }

    pub fn new_oauth(email : String, access_token : String) -> Self {
        SmtpCredential::OAuth { email, access_token }
    }

    pub fn new_oauth_bearer(bearer_token : String) -> Self {
        SmtpCredential::OAuthBearer { bearer_token }
    }

}

impl SmtpConfig {

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

    fn host_name(&self) -> String {
        let host_name = self.host_name()
        .split_once(":")
        .map(|(host, _)| {
            host.to_string()
        })
        .unwrap();
        return host_name
    }

    fn port(&self) -> String {
        let port = self.host_name()
        .split_once(":")
        .map(|(_, port)| {
            port.to_string()
        })
        .unwrap();
        return port
    }

    pub fn communicating(
        &mut self,
        cmd: &[u8],
        closure: Option<&Closure>,
        response_resullt: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.write_cmd(cmd)?;
        self.read_response(closure, response_resullt)?;
        Ok(())
    }

    pub fn read_response(
        &mut self,
        closure: Option<&Closure>,
        response_result: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(&mut self.stream);
        loop {
            let mut response = String::new();
            match reader.read_line(&mut response) {
                Ok(0) => break,
                Ok(_) => {
                    let is_last = response.as_bytes().get(3) == Some(&b' ');

                    if let Some(closure) = closure {
                        closure(response_result, response);
                    }

                    if is_last {
                        break;
                    }

                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }

    pub fn write_cmd(&mut self, cmd: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let sending = self.stream.write_all(cmd)?;
        Ok(sending)
    }

    pub fn parse_auth(
    bucket_response : &Vec<String> 
) -> Vec<AuthMechanism> {
    let mut v = Vec::new();

    for auth_mech in bucket_response {
        if let Some(m) = auth_mech.strip_prefix("250-AUTH ") {
            match m {
                "PLAIN" => v.push(AuthMechanism::Plain),
                "LOGIN" => v.push(AuthMechanism::Login),
                "XOAUTH2" => v.push(AuthMechanism::XOAuth2),
                "OAUTHBEARER" => v.push(AuthMechanism::OAuthBearer),
                "PLAIN-CLIENTTOKEN" => v.push(AuthMechanism::PlainClientToken),
                "XOAUTH" => v.push(AuthMechanism::XOAuth),
                x => v.push(AuthMechanism::Unknown(x.into())),
            };
        }
    };

    v
}

    pub fn authenticating (
        &mut self
    ) -> Result<(), Box<dyn std::error::Error>> {

        let closure : Option<Closure> = Some(Box::new(|response_result: &mut Vec<String>, response : String| {
            response_result.push(response);
        }));

        let mut response_result : Vec<String> = Vec::new();
        let _ = self.communicating(b"EHLO\r\n", closure.as_ref(), &mut response_result);
        let auth_mechs = Self::parse_auth(&response_result);
        let result = cli_smtp(auth_mechs)?;
        if true {
            return Err("".into());
        }

        Ok(())

    }

    pub fn upgrade_tls(
        mut self,
        host: &str,
    ) -> Result<LiveSmtp<StreamOwned<ClientConnection, T>>, Box<dyn std::error::Error>> {
        let mut response_result: Vec<String> = Vec::new();
        let closure : Option<Closure> = Some(Box::new(|response_result: &mut Vec<String>, response : String| {
            response_result.push(response);
        }));
        let _ = self.communicating(b"EHLO\r\n", closure.as_ref(), &mut response_result)?;
        let is_tls_supported = response_result.iter().any(|response| {
            response.starts_with("250-STARTTLS") || response.starts_with( "250 STARTTLS")
        });

        if !is_tls_supported {
            return Err("STARTTLS not supported".into());
        }

        let _ = self.communicating(b"STARTTLS\r\n", closure.as_ref(), &mut response_result);
        let is_tls_ready = &response_result[response_result.len() - 1];

        if is_tls_ready[0..2] != *"220" {
            return Err("TLS is not ready".into())
        }

        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
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
