#![allow(clippy::field_reassign_with_default)]

mod config;
mod error;
mod mcp;
mod stash;
#[cfg(test)]
mod integration_tests;

use std::path::PathBuf;

use anyhow::Context;
use tracing::info;

use crate::{config::Config, mcp::StashMcpHandler, stash::StashClient};

fn usage() -> ! {
    eprintln!(
        "stash-mcp — MCP server for StashApp\n\n\
         USAGE:\n\
         \tstash-mcp [OPTIONS]\n\n\
         OPTIONS:\n\
         \t--config <path>   Path to env-format config file (default: ./stash.env)\n\
         \t-c <path>         Alias for --config\n\
         \t--help, -h        Print this help message"
    );
    std::process::exit(0);
}

fn parse_config_path() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => usage(),
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    return Some(PathBuf::from(&args[i + 1]));
                } else {
                    eprintln!("error: --config requires a path argument");
                    std::process::exit(1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn load_config() -> anyhow::Result<Config> {
    if let Some(path) = parse_config_path() {
        Config::from_file(&path)
            .with_context(|| format!("loading config from '{}'", path.display()))
    } else {
        let default = PathBuf::from("stash.env");
        if default.exists() {
            Config::from_file(&default).context("loading config from './stash.env'")
        } else {
            Config::from_env().map_err(anyhow::Error::from)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr so stdout remains clean for the MCP stdio transport.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stash_mcp=info".parse().unwrap()),
        )
        .init();

    let config = load_config().context("failed to load configuration")?;

    info!(
        endpoint = %config.base_url(),
        favorites_only = config.favorites_only,
        "stash-mcp starting"
    );

    let client = StashClient::new(&config);
    let handler = StashMcpHandler::new(client, config);

    let (stdin, stdout) = rmcp::transport::io::stdio();
    let server = rmcp::serve_server(handler, (stdin, stdout))
        .await
        .context("failed to start MCP server")?;

    server.waiting().await.context("MCP server error")?;

    Ok(())
}
