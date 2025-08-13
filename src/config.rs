//! Configuration management for the Astor system

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub redis: RedisConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout: u64,
    pub max_request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub bcrypt_cost: u32,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();

        // Database configuration
        cfg = cfg
            .set_default("database.max_connections", 20)?
            .set_default("database.min_connections", 5)?
            .set_default("database.acquire_timeout", 30)?
            .set_default("database.idle_timeout", 600)?;

        // Server configuration
        cfg = cfg
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.cors_origins", Vec::<String>::new())?
            .set_default("server.request_timeout", 30)?
            .set_default("server.max_request_size", 1048576)?; // 1MB

        // Security configuration
        cfg = cfg
            .set_default("security.jwt_expiration", 86400)? // 24 hours
            .set_default("security.bcrypt_cost", 12)?
            .set_default("security.rate_limit_requests", 100)?
            .set_default("security.rate_limit_window", 60)?; // 1 minute

        // Redis configuration
        cfg = cfg
            .set_default("redis.max_connections", 10)?
            .set_default("redis.connection_timeout", 5)?;

        // Logging configuration
        cfg = cfg
            .set_default("logging.level", "info")?
            .set_default("logging.format", "json")?;

        // Override with environment variables
        cfg = cfg.add_source(config::Environment::with_prefix("ASTOR"));

        // Required environment variables
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| config::ConfigError::Message("DATABASE_URL is required".to_string()))?;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| config::ConfigError::Message("JWT_SECRET is required".to_string()))?;
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| config::ConfigError::Message("REDIS_URL is required".to_string()))?;

        cfg = cfg
            .set_override("database.url", database_url)?
            .set_override("security.jwt_secret", jwt_secret)?
            .set_override("redis.url", redis_url)?;

        cfg.build()?.try_deserialize()
    }
}
