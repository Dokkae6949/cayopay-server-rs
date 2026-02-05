use domain::Email;
use lettre::{
  message::header::ContentType,
  transport::smtp::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
  },
  AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
  #[error("Failed to parse email address: {0}")]
  AddressParse(String),
  #[error("Failed to build email: {0}")]
  Build(#[from] lettre::error::Error),
  #[error("Failed to send email: {0}")]
  Transport(#[from] lettre::transport::smtp::Error),
}

#[derive(Debug, Clone)]
pub struct EmailServiceConfig {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub from: String,
}

#[derive(Clone)]
pub struct EmailService {
  mailer: AsyncSmtpTransport<Tokio1Executor>,
  from: String,
}

impl EmailService {
  pub fn new(config: EmailServiceConfig) -> Self {
    tracing::info!(
      "Initializing EmailService with host: {}, port: {}",
      config.host,
      config.port
    );

    let creds = Credentials::new(config.username, config.password);

    let mut mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
      .expect("mailer should have been created")
      .port(config.port)
      .credentials(creds);

    match config.port {
      465 => {
        tracing::info!("Using Implicit TLS (Wrapper) for port 465");
        mailer_builder = mailer_builder.tls(Tls::Wrapper(
          TlsParameters::new(config.host.clone()).expect("failed to create tls parameters"),
        ));
      }
      587 => {
        tracing::info!("Using Opportunistic TLS (STARTTLS) for port 587");
        mailer_builder = mailer_builder.tls(Tls::Opportunistic(
          TlsParameters::new(config.host.clone()).expect("failed to create tls parameters"),
        ));
      }
      _ => {
        tracing::info!("Using default TLS settings for port {}", config.port);
      }
    }

    let mailer = mailer_builder.build();

    Self {
      mailer,
      from: config.from,
    }
  }

  pub async fn send_invite(
    &self,
    email: &Email,
    token: &str,
    inviter_name: &str,
  ) -> Result<(), EmailError> {
    let email_str = email.expose();
    let email_msg = Message::builder()
      .from(self.from.parse().map_err(|e| {
        EmailError::AddressParse(format!("From address error: {}", e))
      })?)
      .to(email_str.parse().map_err(|e| {
        EmailError::AddressParse(format!("To address error: {}", e))
      })?)
      .subject("You have been invited to CayoPay")
      .header(ContentType::TEXT_HTML)
      .body(format!(
        "<h1>CayoPay Invitation</h1><br><p>You have been invited to CayoPay by <b>{}</b>.</p><p>Your invite token is: <i>{}</i></p>",
        inviter_name, token
      ))?;

    self.mailer.send(email_msg).await?;

    Ok(())
  }
}
