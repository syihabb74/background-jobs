use std::{
    io::{self, BufRead, BufReader, Read, Write}, net::TcpStream, sync::Arc
};

use base64::{Engine, engine::general_purpose};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;

use crate::{Closure, cli::{cli_auth_credentials, cli_auth_smtp}, smtp::{auth_mechanism::AuthMechanism, tcp_com::{read_response, write_cmd}}};

pub struct SmtpConfig {
    auth_mechanism : Option<AuthMechanism>,
    credentials : Option<SmtpCredential>,
    host: &'static str,
}





#[derive(Debug)]
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

    pub fn encode(plain: &String) -> String {
        general_purpose::STANDARD.encode(plain)
    }

    pub fn encode_auth(&self, mechanism: &AuthMechanism) -> Option<String> {
    match (self, mechanism) {
        (SmtpCredential::EmailPassword { email, password }, AuthMechanism::Plain | AuthMechanism::PlainClientToken) => {
            Some(format!("\0{}\0{}", Self::encode(email), Self::encode(password)))
        },
        (SmtpCredential::OAuth { email, access_token }, AuthMechanism::XOAuth) => {
            Some(format!("user={}\x01auth=OAuth {}\x01\x01", Self::encode(email), Self::encode(access_token)))
        },
        (SmtpCredential::OAuth { email, access_token }, AuthMechanism::XOAuth2) => {
            Some(format!("user={}\x01auth=Bearer {}\x01\x01", Self::encode(email), Self::encode(access_token)))
        },
        (SmtpCredential::OAuthBearer { bearer_token }, AuthMechanism::OAuthBearer) => {
            Some(format!("n,,\x01auth=Bearer {}\x01\x01", Self::encode(bearer_token)))
        },
        _ => None
    }
}


}

impl SmtpConfig {

    pub fn new(host : &'static str) -> Self {
        Self { host,credentials : None, auth_mechanism : None }
    }

    pub fn connect(&self) -> Result<LiveSmtp<TcpStream>, io::Error> {
        let stream = TcpStream::connect(self.host)?;
        let server_name = self.host_name();
        Ok(LiveSmtp { stream , server_name})
    }

     fn host_name(&self) -> String {
        let host_name = self.host
        .split_once(":")
        .map(|(host, _)| {
            host.to_string()
        })
        .unwrap();
        println!("{}", host_name);
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

}

#[derive(Debug)]
pub struct LiveSmtp<T: Read + Write> {
    stream: T,
    server_name :  String
}

impl<T: Read + Write> LiveSmtp<T> {

    pub fn communicating(
        &mut self,
        cmd: &[u8],
        closure: Option<&Closure>,
        response_resullt: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_cmd(&mut self.stream, cmd)?;
        read_response(&mut self.stream,closure, response_resullt)?;
        Ok(())
    }

    pub fn parse_auth(bucket_response: &Vec<String>) -> Vec<AuthMechanism> {
    let mut v = Vec::new();

    for line in bucket_response {
        if let Some(mechs) = line.trim().strip_prefix("250-AUTH ") {
            for mech in mechs.split_whitespace() {
                match mech {
                    "PLAIN" => v.push(AuthMechanism::Plain),
                    "LOGIN" => v.push(AuthMechanism::Login),
                    "XOAUTH2" => v.push(AuthMechanism::XOAuth2),
                    "OAUTHBEARER" => v.push(AuthMechanism::OAuthBearer),
                    "PLAIN-CLIENTTOKEN" => v.push(AuthMechanism::PlainClientToken),
                    "XOAUTH" => v.push(AuthMechanism::XOAuth),
                    x => v.push(AuthMechanism::Unknown(x.into())),
                }
            }
        }
    }

    v
}

    pub fn authenticating (
        &mut self
    ) -> Result<(), Box<dyn std::error::Error>> {

        let closure : Option<Closure> = Some(Box::new(|response_result: &mut Vec<String>, response : String| {
            response_result.push(response);
        }));

        let mut response_result : Vec<String> = Vec::new();
        let _ = self.communicating(b"EHLO mylocalhost\r\n", closure.as_ref(), &mut response_result);
        let _ = self.communicating(b"AUTH LOGIN\r\n", closure.as_ref(), &mut response_result);
        let auth_mechs = Self::parse_auth(&response_result);
        println!("{:#?}", auth_mechs);
        let auth_mech = cli_auth_smtp(auth_mechs)?;
        let credentials = cli_auth_credentials(&auth_mech)?;
        println!("{:?}", credentials);
        if true {
            return Err("".into());
        }

        Ok(())

    }

    pub fn auth_process(input : Option<String>) {
        
    }

    pub fn upgrade_tls(
        mut self,
    ) -> Result<LiveSmtp<StreamOwned<ClientConnection, T>>, Box<dyn std::error::Error>> {
        let mut response_result: Vec<String> = Vec::new();
        let closure : Option<Closure> = Some(Box::new(|response_result: &mut Vec<String>, response : String| {
            response_result.push(response);
        }));
        let _ = self.communicating(b"EHLO mylocalhost\r\n", closure.as_ref(), &mut response_result)?;
        let is_tls_supported = response_result.iter().any(|response| {
            response.starts_with("250-STARTTLS") || response.starts_with( "250 STARTTLS")
        });
        println!("{:?}, {}", response_result, is_tls_supported);

        if !is_tls_supported {
            return Err("STARTTLS not supported".into());
        }

        let _ = self.communicating(b"STARTTLS\r\n", closure.as_ref(), &mut response_result);
        let is_tls_ready = &response_result[response_result.len() - 1];

        println!("{:?}", is_tls_ready);

        if !is_tls_ready.starts_with("220") {
            return Err("TLS is not ready".into())
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
            server_name : self.server_name
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
