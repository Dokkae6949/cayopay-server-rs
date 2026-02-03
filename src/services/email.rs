use lettre::{
  message::header::ContentType,
  transport::smtp::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
  },
  AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
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
  pub fn new(config: &Config) -> AppResult<Self> {
    tracing::info!(
      "Initializing EmailService with host: {}, port: {}",
      config.smtp_host,
      config.smtp_port
    );

    let creds = Credentials::new(
      config.smtp_username.clone().expose().to_string(),
      config.smtp_password.clone().expose().to_string(),
    );

    let mut mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
      .map_err(|e| {
        tracing::error!("Failed to create SMTP transport: {}", e);
        AppError::InternalServerError
      })?
      .port(config.smtp_port)
      .credentials(creds);

    match config.smtp_port {
      465 => {
        tracing::info!("Using Implicit TLS (Wrapper) for port 465");
        let tls_params = TlsParameters::new(config.smtp_host.clone()).map_err(|e| {
          tracing::error!("Failed to create TLS parameters: {}", e);
          AppError::InternalServerError
        })?;
        mailer_builder = mailer_builder.tls(Tls::Wrapper(tls_params));
      }
      587 => {
        tracing::info!("Using Opportunistic TLS (STARTTLS) for port 587");
        let tls_params = TlsParameters::new(config.smtp_host.clone()).map_err(|e| {
          tracing::error!("Failed to create TLS parameters: {}", e);
          AppError::InternalServerError
        })?;
        mailer_builder = mailer_builder.tls(Tls::Opportunistic(tls_params));
      }
      _ => {
        tracing::info!("Using default TLS settings for port {}", config.smtp_port);
      }
    }

    let mailer = mailer_builder.build();

    Ok(Self {
      mailer,
      from: config.smtp_from.clone(),
    })
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
      .header(ContentType::TEXT_HTML)
      .body(format!(
        "<h1>CayoPay Invitation</h1><br><p>You have been invited to CayoPay by <b>{}</b>.</p><br><p>Your invite token is: <i>{}</i></p>",
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
