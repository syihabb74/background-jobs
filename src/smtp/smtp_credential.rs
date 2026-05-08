use base64::{Engine, engine::general_purpose};

use crate::smtp::auth_mechanism::AuthMechanism;

pub enum Challenge {
    Challenging,
    NonChallenging,
    NotSupported,
}

#[derive(Debug)]
pub enum SmtpCredential {
    EmailPassword { email: String, password: String },

    OAuth { email: String, access_token: String },

    OAuthBearer { bearer_token: String },
}

impl SmtpCredential {
    pub fn new_email_password(email: String, password: String) -> Self {
        SmtpCredential::EmailPassword { email, password }
    }

    pub fn new_oauth(email: String, access_token: String) -> Self {
        SmtpCredential::OAuth {
            email,
            access_token,
        }
    }

    pub fn new_oauth_bearer(bearer_token: String) -> Self {
        SmtpCredential::OAuthBearer { bearer_token }
    }

    pub fn encode(plain: &String) -> String {
        general_purpose::STANDARD.encode(plain)
    }

    pub fn encode_auth(
        &self,
        mechanism: &AuthMechanism,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match (self, mechanism) {
            (
                SmtpCredential::EmailPassword { email, password },
                AuthMechanism::Plain | AuthMechanism::PlainClientToken,
            ) => Ok(Self::encode(&format!("\0{}\0{}", email, password))),
            (
                SmtpCredential::OAuth {
                    email,
                    access_token,
                },
                AuthMechanism::XOAuth,
            ) => Ok(Self::encode(&format!("\0{}\0{}", email, access_token))),
            (
                SmtpCredential::OAuth {
                    email,
                    access_token,
                },
                AuthMechanism::XOAuth2,
            ) => Ok(Self::encode(&format!("\0{}\0{}", email, access_token))),
            (SmtpCredential::OAuthBearer { bearer_token }, AuthMechanism::OAuthBearer) => Ok(
                Self::encode(&format!("n,,\x01auth=Bearer {}\x01\x01", bearer_token)),
            ),
            (SmtpCredential::EmailPassword { email, password }, AuthMechanism::Login) => Ok(
                format!("{},{}", Self::encode(email), Self::encode(password)),
            ),
            _ => Err(format!("Auth method currently is not supported").into()),
        }
    }

    pub fn check_challenging_mechanism(&self, mechanism: &AuthMechanism) -> Challenge {
        match (self, mechanism) {
            (
                _,
                AuthMechanism::Plain
                | AuthMechanism::PlainClientToken
                | AuthMechanism::XOAuth
                | AuthMechanism::XOAuth2
                | AuthMechanism::OAuthBearer,
            ) => Challenge::NonChallenging,
            (_, AuthMechanism::Login) => Challenge::Challenging,
            _ => Challenge::NotSupported,
        }
    }
}
