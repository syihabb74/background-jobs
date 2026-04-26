use std::io::{Read, Write};

pub struct SmtpConfig {
    host: &'static str,
    port: u16,
}

impl SmtpConfig {
    pub fn builder(host: &'static str, port: u16) -> Self {
        Self { host, port }
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
