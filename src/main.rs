//! tickit-sync - Self-hosted sync server for Tickit
//!
//! A minimal sync server that stores tasks, lists, and tags,
//! enabling sync across multiple Tickit clients.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod api;
mod config;
mod db;
mod models;

use config::Config;

#[derive(Parser)]
#[command(name = "tickit-sync")]
#[command(about = "Self-hosted sync server for Tickit task manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the sync server
    Serve {
        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,

        /// Bind address (overrides config)
        #[arg(short, long)]
        bind: Option<String>,
    },

    /// Generate a new API token
    Token {
        /// Name/label for the token
        #[arg(short, long)]
        name: Option<String>,

        /// List all configured tokens
        #[arg(long)]
        list: bool,

        /// Revoke a token by name
        #[arg(long)]
        revoke: Option<String>,

        /// Config file path (for list/revoke operations)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Initialize a new config file
    Init {
        /// Output path for config file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tickit_sync=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { config, port, bind } => {
            let mut cfg = if let Some(path) = config {
                Config::load_from(&path)?
            } else {
                Config::load()?
            };

            // Override with CLI args
            if let Some(p) = port {
                cfg.server.port = p;
            }
            if let Some(b) = bind {
                cfg.server.bind = b;
            }

            run_server(cfg).await
        }

        Commands::Token {
            name,
            list,
            revoke,
            config,
        } => {
            let config_path = if let Some(path) = config {
                path
            } else {
                Config::default_path()?
            };

            // List tokens
            if list {
                if !config_path.exists() {
                    println!("No config file found at {}", config_path.display());
                    println!("Run 'tickit-sync init' to create one.");
                    return Ok(());
                }

                let cfg = Config::load_from(&config_path)?;
                if cfg.tokens.is_empty() {
                    println!("No tokens configured.");
                    println!("Generate one with: tickit-sync token --name <device-name>");
                } else {
                    println!("Configured tokens:");
                    println!();
                    for token in &cfg.tokens {
                        // Show truncated hash (first 20 chars)
                        let hash_preview = if token.token_hash.len() > 20 {
                            format!("{}...", &token.token_hash[..20])
                        } else {
                            token.token_hash.clone()
                        };
                        println!("  {} - {}", token.name, hash_preview);
                    }
                }
                return Ok(());
            }

            // Revoke token
            if let Some(token_name) = revoke {
                if !config_path.exists() {
                    println!("No config file found at {}", config_path.display());
                    return Ok(());
                }

                let mut cfg = Config::load_from(&config_path)?;
                let original_len = cfg.tokens.len();
                cfg.tokens.retain(|t| t.name != token_name);

                if cfg.tokens.len() == original_len {
                    println!("Token '{}' not found.", token_name);
                } else {
                    cfg.save_to(&config_path)?;
                    println!("Revoked token '{}'.", token_name);
                }
                return Ok(());
            }

            // Generate new token
            let token = generate_token();
            let label = name.unwrap_or_else(|| "default".to_string());

            // Auto-save to config if it exists
            if config_path.exists() {
                let mut cfg = Config::load_from(&config_path)?;

                // Check if token name already exists
                if cfg.tokens.iter().any(|t| t.name == label) {
                    println!(
                        "Token '{}' already exists. Use --revoke first to replace it.",
                        label
                    );
                    return Ok(());
                }

                // Hash the token before storing
                let token_hash = config::hash_token(&token)?;

                cfg.tokens.push(config::TokenConfig {
                    name: label.clone(),
                    token_hash,
                });
                cfg.save_to(&config_path)?;

                println!("✅ Generated API token for '{}'\n", label);
                println!("Token: {}\n", token);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📱 MOBILE APP (tickit-mobile):");
                println!("   Settings → Sync Server: http://YOUR_SERVER_IP:3030");
                println!("   Settings → Sync Token: {}", token);
                println!("   Settings → Sync Enabled: ON\n");
                println!("💻 DESKTOP CLI (tickit):");
                println!("   Press 's' to open Settings, then configure:");
                println!("   • Sync Server: http://YOUR_SERVER_IP:3030");
                println!("   • Sync Token: {}", token);
                println!("   • Sync Enabled: ON\n");
                println!("   Or add to ~/.config/tickit/config.toml:");
                println!("   [sync]");
                println!("   enabled = true");
                println!("   server = \"http://YOUR_SERVER_IP:3030\"");
                println!("   token = \"{}\"", token);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("⚠️  Save this token now - it cannot be retrieved later!");
            } else {
                // Hash for display (manual setup case)
                let token_hash = config::hash_token(&token)?;

                println!("Generated API token for '{}':\n", label);
                println!("Token: {}\n", token);
                println!("Add this to your server's config.toml:\n");
                println!("  [[tokens]]");
                println!("  name = \"{}\"", label);
                println!("  token_hash = \"{}\"\n", token_hash);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📱 MOBILE APP (tickit-mobile):");
                println!("   Settings → Sync Server: http://YOUR_SERVER_IP:3030");
                println!("   Settings → Sync Token: {}", token);
                println!("   Settings → Sync Enabled: ON\n");
                println!("💻 DESKTOP CLI (tickit):");
                println!("   Press 's' to open Settings, then configure:");
                println!("   • Sync Server: http://YOUR_SERVER_IP:3030");
                println!("   • Sync Token: {}", token);
                println!("   • Sync Enabled: ON");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("⚠️  Save this token now - it cannot be retrieved later!");
            }

            Ok(())
        }

        Commands::Init { output } => {
            let path = output.unwrap_or_else(|| PathBuf::from("config.toml"));
            let cfg = Config::default();
            cfg.save_to(&path)?;

            println!("Created config file: {}", path.display());
            println!();
            println!("Next steps:");
            println!("  1. Generate a token: tickit-sync token --name my-device");
            println!("  2. Add the token to config.toml");
            println!(
                "  3. Start the server: tickit-sync serve --config {}",
                path.display()
            );

            Ok(())
        }
    }
}

async fn run_server(config: Config) -> Result<()> {
    let db = db::Database::open(&config.database.path).context("Failed to open database")?;

    let state = api::AppState::new(db, config.clone());
    let app = api::create_router(state);

    let addr = format!("{}:{}", config.server.bind, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 tickit-sync server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.random();

    // Base64-like encoding but URL-safe
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let token_body: String = bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();

    format!("tks_{}", token_body)
}
