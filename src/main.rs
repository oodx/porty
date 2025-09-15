// Porty - Lightweight async TCP/HTTP port forwarder
// Main entry point with RSB dispatch pattern

use anyhow::Result;
use porty::{load_config, generate_example_config, run_porty_server};
use rsb::prelude::*;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = bootstrap!();
    options!(&args);

    // TODO: Re-add register_function calls once RSB registry issues are fixed

    // Handle setup commands first
    if pre_dispatch!(&args, {
        "generate-config" => cmd_generate_config
    }) {
        return;
    }

    // Main command dispatch (defaults to start)
    dispatch!(&args, {
        "start" => cmd_start,
        "help" => cmd_help,
        "version" => cmd_version
    });
}

fn cmd_start(args: Args) -> i32 {
    match run_async_start(args) {
        Ok(_) => 0,
        Err(e) => {
            stderr!("❌ Failed to start: {}", e);
            1
        }
    }
}

fn cmd_generate_config(args: Args) -> i32 {

    let config_path = args.get_or(1, "config.toml");

    match generate_example_config(&std::path::PathBuf::from(config_path.clone())) {
        Ok(_) => {
            echo!("✅ Generated example config: {}", config_path);
            0
        }
        Err(e) => {
            stderr!("❌ Failed to generate config: {}", e);
            1
        }
    }
}

fn cmd_help(_args: Args) -> i32 {
    let help_text = format!(r#"🚀 Porty v{} - Lightweight TCP/HTTP port forwarder

Usage: porty [COMMAND] [OPTIONS]

Commands:
  start              Start the proxy server (default)
  generate-config    Generate example configuration
  help               Show this help message
  version            Show version information

Options:
  --config FILE      Configuration file path [default: config.toml]
  --listen-port N    Override listen port
  --target-port N    Override target port
  --verbose          Enable verbose logging
  --daemon           Run as daemon (Unix only)"#, env!("CARGO_PKG_VERSION"));

    echo!("{}", help_text);
    0
}

fn cmd_version(_args: Args) -> i32 {
    echo!("🚀 Porty v{}", env!("CARGO_PKG_VERSION"));
    0
}

#[tokio::main]
async fn run_async_start(_args: Args) -> Result<()> {
    // Load config file path from global context only
    let config_path = if has_var("opt_config") {
        get_var("opt_config")
    } else {
        "config.toml".to_string()
    };

    let mut config = load_config(&std::path::PathBuf::from(config_path.clone()))?;

    // Override config with global context (RSB pattern)
    if has_var("opt_listen_port") {
        config.listen_port = get_var("opt_listen_port").parse().unwrap_or(config.listen_port);
    }
    if has_var("opt_target_port") {
        config.target_port = get_var("opt_target_port").parse().unwrap_or(config.target_port);
    }
    if has_var("opt_listen_addr") {
        config.listen_addr = get_var("opt_listen_addr");
    }
    if has_var("opt_target_addr") {
        config.target_addr = get_var("opt_target_addr");
    }

    // Daemonize if requested (Unix only)
    #[cfg(unix)]
    if is_true("opt_daemon") {
        daemonize()?;
    }

    // Print startup message
    echo!("🚀 Porty v{} starting up", env!("CARGO_PKG_VERSION"));
    echo!("📁 Config loaded from: {}", config_path);
    echo!("🔊 Main route: {}:{} -> {}:{}",
        config.listen_addr, config.listen_port,
        config.target_addr, config.target_port);

    run_porty_server(config).await
}

#[cfg(unix)]
fn daemonize() -> Result<()> {
    use std::env;
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    if env::var("DAEMONIZED").is_ok() {
        return Ok(());
    }

    let args: Vec<String> = env::args().collect();
    let _ = Command::new(&args[0])
        .args(&args[1..])
        .env("DAEMONIZED", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .exec();

    Ok(())
}