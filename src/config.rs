use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  #[serde(default = "default_host")]
  pub host: String,
  #[serde(default = "default_port")]
  pub port: u16,

  pub database_url: String,

  pub smtp_host: String,
  pub smtp_port: u16,
  pub smtp_username: String,
  pub smtp_password: String,
  pub smtp_from: String,

  #[serde(default = "default_session_cookie_name")]
  pub session_cookie_name: String,
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

impl Config {
  pub fn init() -> Self {
    dotenvy::dotenv().ok();
    envy::from_env().expect("expected to load config from environment variables or .env file")
  }

  pub fn server_addr(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}
