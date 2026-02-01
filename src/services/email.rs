use lettre::{
  message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
  AsyncTransport, Message, Tokio1Executor,
};

use crate::{
  config::Config,
  error::{AppError, AppResult},
  types::Email,
};

#[derive(Clone)]
pub struct EmailService {
  mailer: AsyncSmtpTransport<Tokio1Executor>,
  from: String,
}

impl EmailService {
  pub fn new(config: &Config) -> Self {
    let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
      .expect("mailer should have been created")
      .port(config.smtp_port)
      .credentials(creds)
      .build();

    Self {
      mailer,
      from: config.smtp_from.clone(),
    }
  }

  pub async fn send_invite(&self, email: &Email, token: &str, inviter_name: &str) -> AppResult<()> {
    let email_msg = Message::builder()
      .from(self.from.parse().map_err(|e| {
        tracing::error!("Failed to parse from address: {}", e);
        AppError::InternalServerError
      })?)
      .to(email.expose().parse().map_err(|e| {
        tracing::error!("Failed to parse to address: {}", e);
        AppError::InternalServerError
      })?)
      .subject("You have been invited to CayoPay")
      .header(ContentType::TEXT_PLAIN)
      .body(format!(
        "You have been invited to CayoPay by {}.\n\nYour invite token is: {}\n",
        inviter_name, token
      ))
      .map_err(|e| {
        tracing::error!("Failed to build email: {}", e);
        AppError::InternalServerError
      })?;

    self.mailer.send(email_msg).await.map_err(|e| {
      tracing::error!("Failed to send email: {}", e);
      AppError::InternalServerError
    })?;

    Ok(())
  }
}
