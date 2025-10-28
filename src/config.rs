use std::env;

use anyhow::Context;
use config::{Config, Environment, File};
use log::{debug, info};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub name: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            user: "mensatt".to_string(),
            password: "password".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            name: "mensatt".to_string(),
        }
    }
}

impl DatabaseConfig {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct JwtConfig {
    pub private_key_path: String,
    pub public_key_path: String,
    pub lifetime_in_secs: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            // Expect keys to be located in currrent directory by default
            private_key_path: "private_key.pem".to_string(),
            public_key_path: "public_key.pem".to_string(),
            lifetime_in_secs: 24 * 60 * 60, // Default JWT lifetime is one day
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ImageServiceConfig {
    pub url: String,
    pub api_key: String,
}

impl Default for ImageServiceConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3000".to_string(),
            api_key: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub jwt: JwtConfig,
    #[serde(default)]
    pub image_service: ImageServiceConfig,
    // HTTP path prefix if running behind a proxy
    #[serde(default)]
    pub proxy_prefix: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // If set, read config from CONFIG_PATH env variable, if not try to read from default path
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yml".to_string());

        // Load config; environment variables take priority over config file
        debug!("Loading configuration from '{}'", config_path);
        let config = Config::builder()
            // Loads values from config_path (if present)
            .add_source(File::with_name(&config_path).required(false))
            // Allow specifying config properties via variables named `MENSATT__<property>`
            .add_source(Environment::with_prefix("MENSATT").separator("__"))
            .build()
            .context("Faled to build configuration")?;
        let deserialized: Self = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        info!("Config loaded successfully");
        debug!("Config is {:?}", deserialized);

        Ok(deserialized)
    }
}
