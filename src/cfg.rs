// Configuration file handling

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use rsb::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    pub name: String,
    pub listen_port: u16,
    pub target_addr: String,
    pub target_port: u16,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub mode: String, // "tcp" or "http"
    #[serde(default)]
    pub host: Option<String>, // Host header matching
    #[serde(default = "default_log_level")]
    pub log_level: String, // "none", "basic", "verbose"
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64, // Connection timeout in seconds
    #[serde(default = "default_max_retries")]
    pub max_retries: u32, // Max retry attempts
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_listen_port")]
    pub listen_port: u16,

    #[serde(default = "default_target_addr")]
    pub target_addr: String,

    #[serde(default = "default_target_port")]
    pub target_port: u16,

    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    #[serde(default = "default_buffer_size")]
    pub buffer_size_kb: usize,

    #[serde(default = "default_log_requests")]
    pub log_requests: bool,

    #[serde(default = "default_log_format")]
    pub log_format: String,

    #[serde(default)]
    pub routes: Vec<Route>,
}

// Default values for config
fn default_listen_addr() -> String { "0.0.0.0".to_string() }
fn default_listen_port() -> u16 { 8080 }
fn default_target_addr() -> String { "127.0.0.1".to_string() }
fn default_target_port() -> u16 { 80 }
fn default_max_connections() -> usize { 100 }
fn default_buffer_size() -> usize { 8 }
fn default_log_requests() -> bool { true }
fn default_log_format() -> String { "default".to_string() }
fn default_log_level() -> String { "basic".to_string() }
fn default_timeout_seconds() -> u64 { 30 }
fn default_max_retries() -> u32 { 2 }

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_addr: default_listen_addr(),
            listen_port: default_listen_port(),
            target_addr: default_target_addr(),
            target_port: default_target_port(),
            max_connections: default_max_connections(),
            buffer_size_kb: default_buffer_size(),
            log_requests: default_log_requests(),
            log_format: default_log_format(),
            routes: vec![],
        }
    }
}

pub fn load_config(path: &PathBuf) -> Result<Config> {
    if !path.exists() {
        echo!("⚠️  Config file not found at: {}", path.display());
        echo!("   Using default configuration...");
        echo!("   Run 'porty --generate-config' to create an example config file");
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .context(format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

pub fn generate_example_config(path: &PathBuf) -> Result<()> {
    let example_config = Config {
        listen_addr: "0.0.0.0".to_string(),
        listen_port: 1455,
        target_addr: "127.0.0.1".to_string(),
        target_port: 1455,
        max_connections: 100,
        buffer_size_kb: 8,
        log_requests: true,
        log_format: "default".to_string(),
        routes: vec![
            Route {
                name: "web".to_string(),
                listen_port: 8080,
                target_addr: "127.0.0.1".to_string(),
                target_port: 80,
                enabled: false,
                mode: "tcp".to_string(),
                host: None,
                log_level: default_log_level(),
                timeout_seconds: default_timeout_seconds(),
                max_retries: default_max_retries(),
            },
            Route {
                name: "api-host-routing".to_string(),
                listen_port: 9080,
                target_addr: "api.example.com".to_string(),
                target_port: 80,
                enabled: false,
                mode: "http".to_string(),
                host: Some("api.example.com".to_string()),
                log_level: "verbose".to_string(),
                timeout_seconds: 10,
                max_retries: 3,
            },
            Route {
                name: "ssh".to_string(),
                listen_port: 2222,
                target_addr: "127.0.0.1".to_string(),
                target_port: 22,
                enabled: false,
                mode: "tcp".to_string(),
                host: None,
                log_level: default_log_level(),
                timeout_seconds: default_timeout_seconds(),
                max_retries: default_max_retries(),
            },
        ],
    };

    let toml_string = toml::to_string_pretty(&example_config)?;

    // Add comments to the generated TOML
    let commented_toml = format!(
        "# Porty Configuration File\n\
         # Generated with 'porty --generate-config'\n\n\
         # Main forwarding configuration\n\
         {}\n\n\
         # Additional routes (optional)\n\
         # Enable routes by setting 'enabled = true'\n",
        toml_string
    );

    fs::write(path, commented_toml)?;
    echo!("✅ Example config file created at: {}", path.display());
    echo!("   Edit this file to configure your port forwarding rules");

    Ok(())
}