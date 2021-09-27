use std::convert::{TryFrom, TryInto};

use sqlx::postgres::{PgConnectOptions, PgSslMode};

use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
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
    //         self.username, self.password, self.host, self.port, self.database_name,
    //     )
    // }

    // pub fn connection_string_without_db(&self) -> String {
    //     format!(
    //         "postgres://{}:{}@{}:{}",
    //         self.username, self.password, self.host, self.port,
    //     )
    // }
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize the configuration reader
    let mut settings = config::Config::default();

    // Create a base path to reference config files against
    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");

    let configuration_directory = base_path.join("configuration");

    // Read the base config file
    settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;

    // Detect the running environment. Default to 'local' if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    // Layer on the environment-specific values.
    settings.merge(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    )?;

    // Add in settings from envrionment variables (prefixed with 'APP' and using '__' as a separator)
    // E.g. 'APP_APPLICATION__PORT=5001' would set 'Settings.application.port'
    settings.merge(config::Environment::with_prefix("app").separator("__"))?;
    // Try to convert the configuration values it read into our `Settings` type
    settings.try_into()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'.",
                other
            )),
        }
    }
}
