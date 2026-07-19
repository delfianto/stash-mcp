#![allow(clippy::field_reassign_with_default)]

mod config;
mod error;
#[cfg(test)]
mod integration_tests;
mod mcp;
mod stash;

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

    serve(handler).await
}

// ── Antigravity (AGY CLI) protocol-violation shim ─────────────────────────────

/// Non-MCP JSON-RPC methods that Google's Antigravity ("AGY") CLI illegally
/// sends *before* the mandatory `initialize` handshake. AGY merged its
/// proprietary "plugin" discovery into standard MCP, so it probes every server
/// with `server/discover` first to decide whether the executable is an AGY
/// plugin. The MCP spec requires `initialize` to be the very first request;
/// rmcp's strict state machine treats this stray frame as fatal and drops the
/// connection (`expect initialized request, but received: ...server/discover...`).
///
/// Node-based servers survive only because they answer the unknown method with
/// a JSON-RPC `-32601` error and keep the pipe open, after which AGY proceeds
/// to a normal `initialize`. `intercept_probe` reproduces exactly that.
/// Extend this list if AGY starts sending further pre-init probe methods.
const AGY_PROBE_METHODS: &[&str] = &["server/discover"];

/// If `line` is one of AGY's illegal pre-init probe *requests*, return the
/// newline-terminated `-32601 Method not found` reply to send back, so the
/// frame can be swallowed instead of forwarded to rmcp. Any other frame —
/// including a real `initialize` — returns `None` and passes through untouched.
fn intercept_probe(line: &[u8]) -> Option<Vec<u8>> {
    let msg: serde_json::Value = serde_json::from_slice(line).ok()?;
    let method = msg.get("method")?.as_str()?;
    if !AGY_PROBE_METHODS.contains(&method) {
        return None;
    }
    // Only requests (those carrying an `id`) expect a reply; echo the id back.
    let id = msg.get("id")?;
    let reply = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32601, "message": "Method not found" },
    });
    let mut bytes = serde_json::to_vec(&reply).ok()?;
    bytes.push(b'\n');
    Some(bytes)
}

/// Serve the MCP protocol on stdio. rmcp enforces the MCP lifecycle strictly,
/// so we can't hand it raw stdio: AGY's pre-init `server/discover` probe would
/// kill it before the handshake (see `intercept_probe`). Instead we sit between
/// the client and rmcp on a pair of in-memory pipes and filter the client's
/// input stream, answering the probe ourselves and passing everything else
/// through byte-for-byte. A single task owns the real stdout so injected
/// replies and rmcp's output can never interleave.
async fn serve(handler: StashMcpHandler) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

    const PIPE_BUF: usize = 64 * 1024;
    let (mut to_server, from_client) = tokio::io::duplex(PIPE_BUF); // filter → rmcp stdin
    let (to_client, mut from_server) = tokio::io::duplex(PIPE_BUF); // rmcp stdout → forwarder

    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(frame) = out_rx.recv().await {
            if stdout.write_all(&frame).await.is_err() {
                break;
            }
            let _ = stdout.flush().await;
        }
    });

    // Pump rmcp's output to the real stdout, byte-for-byte.
    {
        let out_tx = out_tx.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                match from_server.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if out_tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }

    // Filter the client's input: answer AGY's illegal `server/discover` probe
    // ourselves and drop it; forward every other frame to rmcp unchanged.
    tokio::spawn(async move {
        let mut reader = BufReader::new(tokio::io::stdin());
        let mut line: Vec<u8> = Vec::new();
        loop {
            line.clear();
            match reader.read_until(b'\n', &mut line).await {
                Ok(0) | Err(_) => break, // EOF: dropping `to_server` shuts rmcp down
                Ok(_) => {
                    if let Some(reply) = intercept_probe(&line) {
                        info!("swallowed non-MCP `server/discover` probe (Antigravity CLI)");
                        if out_tx.send(reply).is_err() {
                            break;
                        }
                        continue;
                    }
                    if to_server.write_all(&line).await.is_err() {
                        break;
                    }
                    let _ = to_server.flush().await;
                }
            }
        }
    });

    let server = rmcp::serve_server(handler, (from_client, to_client))
        .await
        .context("failed to start MCP server")?;
    server.waiting().await.context("MCP server error")?;
    Ok(())
}

// ── Tests: Antigravity `server/discover` probe interception ───────────────────

#[cfg(test)]
mod agy_shim_tests {
    use super::*;

    #[test]
    fn intercepts_agy_server_discover_probe() {
        let line = br#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{}}"#;
        let reply = intercept_probe(line).expect("server/discover must be intercepted");
        assert!(reply.ends_with(b"\n"), "reply must be newline-framed");
        let v: serde_json::Value = serde_json::from_slice(&reply).unwrap();
        assert_eq!(v["jsonrpc"], serde_json::json!("2.0"));
        assert_eq!(
            v["id"],
            serde_json::json!(1),
            "must echo the request id back"
        );
        assert_eq!(
            v["error"]["code"],
            serde_json::json!(-32601),
            "must be Method not found"
        );
    }

    #[test]
    fn intercept_echoes_string_id() {
        let line = br#"{"jsonrpc":"2.0","id":"probe-7","method":"server/discover"}"#;
        let reply = intercept_probe(line).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&reply).unwrap();
        assert_eq!(v["id"], serde_json::json!("probe-7"));
    }

    #[test]
    fn initialize_passes_through() {
        let line = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        assert!(
            intercept_probe(line).is_none(),
            "initialize must never be swallowed"
        );
    }

    #[test]
    fn normal_mcp_traffic_passes_through() {
        assert!(
            intercept_probe(br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none()
        );
        assert!(intercept_probe(br#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#).is_none());
        assert!(
            intercept_probe(br#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{}}"#)
                .is_none()
        );
    }

    #[test]
    fn non_json_and_empty_lines_pass_through() {
        assert!(intercept_probe(b"this is not json\n").is_none());
        assert!(intercept_probe(b"\n").is_none());
        assert!(intercept_probe(b"").is_none());
    }
}
