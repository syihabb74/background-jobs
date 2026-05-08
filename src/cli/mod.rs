use crate::smtp::{auth_mechanism::AuthMechanism, smtp_credential::SmtpCredential};
use std::io::{self, Error, Write};

pub fn cli_auth_smtp(
    auth_mechs: Vec<AuthMechanism>,
) -> Result<AuthMechanism, Box<dyn std::error::Error>> {
    if auth_mechs.is_empty() {
        return Err("Tidak ada metode autentikasi yang tersedia".into());
    }

    println!("Server supports authentication:\n");

    for (i, auth) in auth_mechs.iter().enumerate() {
        auth.cli_display(i);
    }

    let mut input = String::new();
    prompt("\nChoose authentication method by number", &mut input)?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| format!("'{}' Invalid input", input.trim()))?;

    // usize tidak perlu cek < 0
    if choice >= auth_mechs.len() {
        return Err(format!("Expected 0 - {}", auth_mechs.len() - 1).into());
    }

    // into_iter().nth() untuk move keluar dari Vec
    auth_mechs
        .into_iter()
        .nth(choice)
        .ok_or_else(|| "Index tidak ditemukan".into())
}

pub fn prompt(label: &str, output: &mut String) -> Result<(), Error> {
    print!("{}: ", label);
    io::stdout().flush().unwrap();
    io::stdin().read_line(output).unwrap();
    *output = output.trim().to_string(); // hapus \n
    Ok(())
}

pub fn cli_auth_credentials(
    auth_mechanism: &AuthMechanism,
) -> Result<SmtpCredential, Box<dyn std::error::Error>> {
    auth_mechanism.generate_credentials()
}
