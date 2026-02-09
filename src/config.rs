//! Configuration for tickit-sync server

use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub tokens: Vec<TokenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind address
    #[serde(default = "default_bind")]
    pub bind: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Human-readable name for the token
    pub name: String,
    /// The hashed API token (argon2 hash, or plain text for backwards compat)
    pub token_hash: String,
}

fn default_bind() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3030
}

fn default_db_path() -> PathBuf {
    PathBuf::from("tickit-sync.sqlite")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind: default_bind(),
                port: default_port(),
            },
            database: DatabaseConfig {
                path: default_db_path(),
            },
            tokens: Vec::new(),
        }
    }
}

impl Config {
    /// Default config path
    pub fn default_path() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(env_path) = std::env::var("TICKIT_SYNC_CONFIG") {
            return Ok(PathBuf::from(env_path));
        }

        // Check for config in current directory
        let local = PathBuf::from("config.toml");
        if local.exists() {
            return Ok(local);
        }

        // Check /data/config.toml (Docker default)
        let data_config = PathBuf::from("/data/config.toml");
        if data_config.exists() {
            return Ok(data_config);
        }

        // Then check XDG config
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("tickit-sync");

        Ok(config_dir.join("config.toml"))
    }

    /// Load config from default path
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        if path.exists() {
            Self::load_from(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load config from specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path).context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config file")
    }

    /// Save config to specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        // Add helpful comments
        let with_comments = format!(
            "# tickit-sync configuration\n\
             # See: https://github.com/ricardodantas/tickit-sync\n\n\
             {}\n\n\
             # Add tokens with: tickit-sync token --name <device-name>\n",
            content
        );

        std::fs::write(path, with_comments).context("Failed to write config file")?;

        Ok(())
    }

    /// Check if a token is valid (supports both hashed and legacy plain tokens)
    pub fn validate_token(&self, token: &str) -> bool {
        let argon2 = Argon2::default();

        for t in &self.tokens {
            // Try to parse as argon2 hash
            if let Ok(parsed_hash) = PasswordHash::new(&t.token_hash) {
                if argon2
                    .verify_password(token.as_bytes(), &parsed_hash)
                    .is_ok()
                {
                    return true;
                }
            } else {
                // Fallback: plain text comparison (legacy/backwards compat)
                if t.token_hash == token {
                    return true;
                }
            }
        }
        false
    }
}

/// Hash a token using argon2
pub fn hash_token(token: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(token.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash token: {}", e))?;
    Ok(hash.to_string())
}
