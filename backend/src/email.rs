use anyhow::{Context, Result};
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub async fn send_otp_email(to_email: &str, otp: &str) -> Result<()> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);
    let smtp_username =
        std::env::var("SMTP_USERNAME").context("SMTP_USERNAME env var is required")?;
    let smtp_password =
        std::env::var("SMTP_PASSWORD").context("SMTP_PASSWORD env var is required")?;
    let from_email =
        std::env::var("FROM_EMAIL").unwrap_or_else(|_| smtp_username.clone());

    let from: Mailbox = format!("Flight Tracker <{}>", from_email).parse()?;
    let to: Mailbox = to_email.parse()?;

    let body = format!(
        "Your one-time login code is:\n\n    {otp}\n\nThis code expires in 15 minutes.\n\nIf you didn't request this, you can ignore this email."
    );

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("Your Flight Tracker login code")
        .body(body)?;

    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
        .port(smtp_port)
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    Ok(())
}
