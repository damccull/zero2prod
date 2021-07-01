use std::convert::{TryFrom, TryInto};

use serde::Deserialize;

use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationSettings {
    pub listen_address: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub listen_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
    // pub fn connection_string(&self) -> String {
    //     format!(
    //         "postgres://{}:{}@{}:{}/{}",
    //         self.username, self.password, self.host, self.port, self.database_name
    //     )
    // }

    // pub fn connection_string_without_db(&self) -> String {
    //     format!(
    //         "postgres://{}:{}@{}:{}",
    //         self.username, self.password, self.host, self.port
    //     )
    // }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize the config reader
    let mut settings = config::Config::default();
    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");
    let configuration_directory = base_path.join("configuration");

    // Read the base config file
    settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;

    // Detect the running environment
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    // Layer on the environment-specific values
    settings.merge(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    )?;

    // Add settings from environment variables
    // Prefix variables with 'APP_' and separate the path with '__'
    // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
    settings.merge(config::Environment::with_prefix("app").separator("__"))?;

    // Try to convert the configuration values into our settings type
    settings.try_into()
}

/// The possible environments for this app to run in
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'development' or 'production'.",
                other
            )),
        }
    }
}
