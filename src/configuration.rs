use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
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

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize the configuration reader
    let mut settings = config::Config::default();

    // Add config values from a `configuration` file
    // Looks at any top-level file named `configuration` with an extension that `config`
    // knows how to parse: yaml, json, etc.
    settings.merge(config::File::with_name("configuration"))?;

    // Try to convert the configuration values it read into our `Settings` type
    settings.try_into()
}
