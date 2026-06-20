//! Leptos MCP Server
//!
//! A Model Context Protocol server that provides Leptos documentation
//! and code assistance tools for AI agents.
//!
//! Implements MCP protocol via JSON-RPC over stdio.

use anyhow::Result;
#[cfg(feature = "stdio")]
use leptos_mcp_server::sdk;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_HOST: &str = "127.0.0.1";
const TRANSPORT_ENV: &str = "LEPTOS_MCP_TRANSPORT";
const HOST_ENV: &str = "LEPTOS_MCP_HOST";
const PORT_ENV: &str = "LEPTOS_MCP_PORT";

#[derive(Debug, Clone, PartialEq, Eq)]
enum TransportSelection {
    Stdio,
    StreamableHttp { host: String, port: Option<String> },
    Sse { host: String, port: Option<String> },
}

impl TransportSelection {
    fn name(&self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::StreamableHttp { .. } => "streamable-http",
            Self::Sse { .. } => "sse",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliAction {
    Run(TransportSelection),
    Help,
}

#[tokio::main]
async fn main() -> Result<()> {
    let action = parse_transport_selection(std::env::args().skip(1), |name| std::env::var(name))
        .map_err(|error| {
            eprintln!("{error}");
            anyhow::anyhow!(error)
        })?;

    if action == CliAction::Help {
        print_help();
        return Ok(());
    }

    let CliAction::Run(selection) = action else {
        unreachable!("help action returned before server startup")
    };

    let transport_name = selection.name();
    if matches!(
        selection,
        TransportSelection::StreamableHttp { .. } | TransportSelection::Sse { .. }
    ) {
        return Err(unsupported_network_transport(transport_name));
    }

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("leptos_mcp_server=info".parse()?))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Leptos MCP Server...");

    start_stdio().await?;

    Ok(())
}

#[cfg(feature = "stdio")]
async fn start_stdio() -> Result<()> {
    sdk::start_stdio()
        .await
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(not(feature = "stdio"))]
async fn start_stdio() -> Result<()> {
    let message = "transport 'stdio' is disabled in this build; rebuild with feature 'stdio'";
    eprintln!("{message}");
    Err(anyhow::anyhow!(message))
}

fn unsupported_network_transport(transport: &str) -> anyhow::Error {
    let message = format!(
        "transport '{transport}' is not supported in this build: network transports are deferred/disabled because request body/message limits and read/request/handler timeouts cannot be configured and verified with the current SDK/server stack; no network listener was started. Network defaults: host {DEFAULT_HOST}; port must be provided explicitly with --port or {PORT_ENV}."
    );
    eprintln!("{message}");
    anyhow::anyhow!(message)
}

fn parse_transport_selection(
    args: impl IntoIterator<Item = String>,
    mut env: impl FnMut(&str) -> Result<String, std::env::VarError>,
) -> Result<CliAction, String> {
    let mut transport = env(TRANSPORT_ENV).ok();
    let mut host = env(HOST_ENV).ok();
    let mut port = env(PORT_ENV).ok();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(CliAction::Help),
            "--transport" => {
                transport = Some(next_arg(&mut args, "--transport")?);
            }
            "--host" => {
                host = Some(next_arg(&mut args, "--host")?);
            }
            "--port" => {
                port = Some(next_arg(&mut args, "--port")?);
            }
            value if value.starts_with("--transport=") => {
                transport = Some(value["--transport=".len()..].to_string());
            }
            value if value.starts_with("--host=") => {
                host = Some(value["--host=".len()..].to_string());
            }
            value if value.starts_with("--port=") => {
                port = Some(value["--port=".len()..].to_string());
            }
            _ => {
                return Err(format!(
                    "unsupported argument '{arg}'. Use --help for usage."
                ));
            }
        }
    }

    let transport = transport.unwrap_or_else(|| "stdio".to_string());
    let selection = match transport.as_str() {
        "stdio" => {
            if host.is_some() || port.is_some() {
                return Err(format!(
                    "--host/--port and {HOST_ENV}/{PORT_ENV} are only valid with network transports; stdio is the default transport"
                ));
            }
            TransportSelection::Stdio
        }
        "streamable-http" | "http" => TransportSelection::StreamableHttp {
            host: host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
            port,
        },
        "sse" => TransportSelection::Sse {
            host: host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
            port,
        },
        _ => {
            return Err(format!(
                "unsupported transport '{transport}'. Supported transport: stdio. Deferred network transports: streamable-http, sse."
            ));
        }
    };

    Ok(CliAction::Run(selection))
}

fn next_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn print_help() {
    println!(
        "Leptos MCP Server\n\nUSAGE:\n    leptos-mcp-server [--transport stdio]\n\nOPTIONS:\n    --transport <stdio|streamable-http|sse>  Select transport (default/env: stdio; only stdio is implemented)\n    --host <HOST>                            Future network host input (default: {DEFAULT_HOST}; network transports deferred)\n    --port <PORT>                            Required for future network transports\n    -h, --help                               Print help\n\nENVIRONMENT:\n    {TRANSPORT_ENV}=stdio|streamable-http|sse\n    {HOST_ENV}=HOST (default: {DEFAULT_HOST})\n    {PORT_ENV}=PORT\n\nNetwork transports are currently deferred/unsupported in this build and will fail without starting listeners. The documented future safe host default is {DEFAULT_HOST}; a port must be provided explicitly. Do not expose a future network listener publicly without separate production controls. Because network transports are disabled, network authentication, CORS allowlists, request body/message limits, and read/request/handler timeouts are not configured. There is no default wildcard CORS policy. Network support remains disabled until those guardrails can be configured and verified with sanitized malformed-input errors."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Result<String, std::env::VarError> {
        Err(std::env::VarError::NotPresent)
    }

    #[test]
    fn defaults_to_stdio() {
        assert_eq!(
            parse_transport_selection([], no_env),
            Ok(CliAction::Run(TransportSelection::Stdio))
        );
    }

    #[test]
    fn accepts_explicit_stdio() {
        assert_eq!(
            parse_transport_selection(["--transport".into(), "stdio".into()], no_env),
            Ok(CliAction::Run(TransportSelection::Stdio))
        );
    }

    #[test]
    fn rejects_stdio_with_explicit_host_even_when_it_matches_default() {
        let error = parse_transport_selection(["--host=127.0.0.1".into()], no_env)
            .expect_err("host configuration should not be accepted for stdio");

        assert!(error.contains("--host/--port"));
        assert!(error.contains("stdio"));
    }

    #[test]
    fn rejects_stdio_with_host_env_even_when_it_matches_default() {
        let error = parse_transport_selection([], |name| match name {
            HOST_ENV => Ok(DEFAULT_HOST.to_string()),
            _ => Err(std::env::VarError::NotPresent),
        })
        .expect_err("host environment should not be accepted for stdio");

        assert!(error.contains(HOST_ENV));
        assert!(error.contains("stdio"));
    }

    #[test]
    fn parses_deferred_network_selection_without_port_default() {
        assert_eq!(
            parse_transport_selection(["--transport=streamable-http".into()], no_env),
            Ok(CliAction::Run(TransportSelection::StreamableHttp {
                host: DEFAULT_HOST.to_string(),
                port: None,
            }))
        );
    }

    #[test]
    fn rejects_unknown_transport() {
        let error = parse_transport_selection(["--transport".into(), "websocket".into()], no_env)
            .expect_err("unknown transport should fail");

        assert!(error.contains("unsupported transport 'websocket'"));
    }
}
