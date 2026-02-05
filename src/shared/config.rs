use serde::Deserialize;

use domain::{Email, RawPassword};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  #[serde(default = "default_host")]
  pub host: String,
  #[serde(default = "default_port")]
  pub port: u16,

  pub database_url: String,
  #[serde(default)]
  pub database_migrations: bool,

  pub smtp_host: String,
  pub smtp_port: u16,
  pub smtp_username: Email,
  pub smtp_password: RawPassword,
  pub smtp_from: String,

  #[serde(default = "default_session_cookie_name")]
  pub session_cookie_name: String,

  #[serde(default = "default_session_expiration_days")]
  pub session_expiration_days: i64,

  #[serde(default = "default_owner_email")]
  pub owner_email: Email,
  #[serde(default = "default_owner_password")]
  pub owner_password: RawPassword,
  #[serde(default = "default_owner_first_name")]
  pub owner_first_name: String,
  #[serde(default = "default_owner_last_name")]
  pub owner_last_name: String,
}

fn default_host() -> String {
  "127.0.0.1".to_string()
}

fn default_port() -> u16 {
  3000
}

fn default_session_cookie_name() -> String {
  "cayopay_session".to_string()
}

fn default_session_expiration_days() -> i64 {
  1
}

fn default_owner_email() -> Email {
  Email::new("admin@example.com")
}

fn default_owner_password() -> RawPassword {
  RawPassword::new("password")
}

fn default_owner_first_name() -> String {
  "Admin".to_string()
}

fn default_owner_last_name() -> String {
  "User".to_string()
}

impl Config {
  pub fn init() -> Self {
    dotenvy::dotenv().ok();
    envy::from_env().expect("expected to load config from environment variables or .env file")
  }

  pub fn server_addr(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }

  pub fn email_config(&self) -> crate::services::EmailServiceConfig {
    crate::services::EmailServiceConfig {
      host: self.smtp_host.clone(),
      port: self.smtp_port,
      username: self.smtp_username.expose().to_string(),
      password: self.smtp_password.expose().to_string(),
      from: self.smtp_from.clone(),
    }
  }
}
