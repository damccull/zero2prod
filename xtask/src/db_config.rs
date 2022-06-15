use std::env;

const DEFAULT_DB_USER: &str = "postgres";
const DEFAULT_DB_PASSWORD: &str = "password";
const DEFAULT_DB_NAME: &str = "norseline";
const DEFAULT_DB_PORT: &str = "5432";

pub struct DbConfig {
    username: String,
    password: String,
    db_name: String,
    db_port: String,
}
impl DbConfig {
    pub fn get_config() -> Self {
        Self {
            username: env::var("POSTGRES_USER").unwrap_or_else(|_| DEFAULT_DB_USER.to_string()),
            password: env::var("POSTGRES_PASSWORD")
                .unwrap_or_else(|_| DEFAULT_DB_PASSWORD.to_string()),
            db_name: env::var("POSTGRES_DB").unwrap_or_else(|_| DEFAULT_DB_NAME.to_string()),
            db_port: env::var("POSTGRES_PORT").unwrap_or_else(|_| DEFAULT_DB_PORT.to_string()),
        }
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }
    pub fn password(&self) -> String {
        self.password.clone()
    }
    pub fn db_name(&self) -> String {
        self.db_name.clone()
    }
    pub fn db_port(&self) -> String {
        self.db_port.clone()
    }
}
