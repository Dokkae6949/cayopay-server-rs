//! Minimal shared configuration
//! Only what's absolutely necessary - no over-abstraction

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
    
    pub owner_email: Email,
    pub owner_password: RawPassword,
    pub owner_first_name: String,
    pub owner_last_name: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

impl Config {
    pub fn init() -> Self {
        dotenvy::dotenv().ok();
        envy::from_env().expect("Failed to load config from environment")
    }
    
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    
    pub fn smtp_config(&self) -> crate::features::send_invite::SmtpConfig {
        crate::features::send_invite::SmtpConfig {
            host: self.smtp_host.clone(),
            port: self.smtp_port,
            username: self.smtp_username.expose().to_string(),
            password: self.smtp_password.expose().to_string(),
            from: self.smtp_from.clone(),
        }
    }
}
