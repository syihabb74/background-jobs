use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub content: String,
}

impl Email {
    pub fn to_struct(buf: &[u8; 1024], n: usize) -> Self {
        let email = serde_json::from_slice::<Email>(&buf[..n]).unwrap();
        email
    }
}
