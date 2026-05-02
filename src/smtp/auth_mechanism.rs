#[derive(Debug)]
pub enum AuthMechanism {
    Unknown(String),
    Plain,
    Login,
    XOAuth,
    XOAuth2,
    OAuthBearer,
    PlainClientToken
}