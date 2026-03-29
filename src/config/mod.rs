use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub database_name: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_expiry_secs: u64,
    pub jwt_refresh_expiry_secs: u64,
    pub server_host: String,
    pub server_port: u16,
    pub environment: Environment,
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let database_url = required_env("DATABASE_URL");
        let database_name = required_env("DATABASE_NAME");
        let jwt_access_secret = required_env("JWT_ACCESS_SECRET");
        let jwt_refresh_secret = required_env("JWT_REFRESH_SECRET");

        if jwt_access_secret.len() < 32 {
            panic!("JWT_ACCESS_SECRET must be at least 32 characters long");
        }
        if jwt_refresh_secret.len() < 32 {
            panic!("JWT_REFRESH_SECRET must be at least 32 characters long");
        }
        if jwt_access_secret == jwt_refresh_secret {
            panic!("JWT_ACCESS_SECRET and JWT_REFRESH_SECRET must be different");
        }

        let jwt_access_expiry_secs = optional_env_u64("JWT_ACCESS_EXPIRY_SECS", 900);
        let jwt_refresh_expiry_secs = optional_env_u64("JWT_REFRESH_EXPIRY_SECS", 604_800);

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("SERVER_PORT must be a valid port number (1-65535)");

        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            _ => Environment::Development,
        };

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        AppConfig {
            database_url,
            database_name,
            jwt_access_secret,
            jwt_refresh_secret,
            jwt_access_expiry_secs,
            jwt_refresh_expiry_secs,
            server_host,
            server_port,
            environment,
            allowed_origins,
        }
    }
}

fn required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Required environment variable '{key}' is not set"))
}

fn optional_env_u64(key: &str, default: u64) -> u64 {
    env::var(key).map_or(default, |v| {
        v.parse::<u64>()
            .unwrap_or_else(|_| panic!("Environment variable '{key}' must be a positive integer"))
    })
}
