// CLI argument handling with RSB integration

use anyhow::Result;
use rsb::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct PortyArgs {
    pub config: PathBuf,
    pub generate_config: bool,
    pub listen_addr: Option<String>,
    pub listen_port: Option<u16>,
    pub target_addr: Option<String>,
    pub target_port: Option<u16>,
    pub daemon: bool,
    pub verbose: bool,
}

impl PortyArgs {
    pub fn from_rsb_args(mut args: Args) -> Self {
        let config = PathBuf::from(args.has_val("--config").unwrap_or_else(|| "config.toml".to_string()));
        let generate_config = args.has("--generate-config");
        let listen_addr = args.has_val("--listen-addr").or_else(|| args.has_val("-l"));
        let listen_port = args.has_val("--listen-port").or_else(|| args.has_val("-p")).and_then(|s| s.parse().ok());
        let target_addr = args.has_val("--target-addr").or_else(|| args.has_val("-t"));
        let target_port = args.has_val("--target-port").or_else(|| args.has_val("-P")).and_then(|s| s.parse().ok());
        let daemon = args.has("--daemon") || args.has("-d");
        let verbose = args.has("--verbose") || args.has("-v");

        PortyArgs {
            config,
            generate_config,
            listen_addr,
            listen_port,
            target_addr,
            target_port,
            daemon,
            verbose,
        }
    }
}