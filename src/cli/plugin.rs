use super::output::{self, resolve_config};
use crate::lib::client::EagleClient;
use crate::lib::types::{PluginDiscovery, Status};
use clap::{Arg, ArgMatches, Command};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Discovery directory where plugins write their server info.
const DISCOVERY_DIR: &str = ".eagle-plugins/servers";

pub fn build() -> Command {
    Command::new("plugin")
        .about("Discover and call Eagle plugin servers")
        .subcommand(Command::new("list").about("List running plugin servers"))
        .subcommand(
            Command::new("routes")
                .about("List routes for a plugin server")
                .arg(
                    Arg::new("plugin-id")
                        .help("Plugin ID (or prefix)")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("call")
                .about("Call a plugin server route")
                .arg(
                    Arg::new("plugin-id")
                        .help("Plugin ID (or prefix)")
                        .required(true),
                )
                .arg(
                    Arg::new("method")
                        .help("HTTP method (GET, POST, PUT, DELETE)")
                        .required(true),
                )
                .arg(Arg::new("path").help("Route path (e.g. /health)").required(true))
                .arg(
                    Arg::new("body")
                        .long("body")
                        .value_name("JSON")
                        .help("JSON request body"),
                ),
        )
}

/// Execute the plugin subcommand.
///
/// This does NOT take `&EagleClient` because it doesn't talk to Eagle's API.
/// It reads local discovery files and creates its own clients per-call.
pub async fn execute(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let config = resolve_config(matches);

    match matches.subcommand() {
        Some(("list", _)) => {
            let plugins = list_live_plugins()?;
            if plugins.is_empty() {
                eprintln!("No running plugin servers found");
                return Ok(());
            }
            output::output(&plugins, &config)?;
        }
        Some(("routes", sub_matches)) => {
            let plugin_id = sub_matches
                .get_one::<String>("plugin-id")
                .expect("plugin-id is required");
            let plugin = find_plugin(plugin_id)?;
            output::output(&plugin.routes, &config)?;
        }
        Some(("call", sub_matches)) => {
            let plugin_id = sub_matches
                .get_one::<String>("plugin-id")
                .expect("plugin-id is required");
            let method = sub_matches
                .get_one::<String>("method")
                .expect("method is required");
            let path = sub_matches
                .get_one::<String>("path")
                .expect("path is required");
            let body = sub_matches.get_one::<String>("body");

            let plugin = find_plugin(plugin_id)?;
            let data = call_plugin(&plugin, method, path, body.map(|s| s.as_str())).await?;
            output::output_value(&data, &config)?;
        }
        _ => {
            eprintln!("Error: No subcommand was used. Try: list, routes, call");
            std::process::exit(super::output::exit_code::USAGE);
        }
    }

    Ok(())
}

// =============================================================================
// Discovery helpers
// =============================================================================

/// Get the discovery directory path (~/.eagle-plugins/servers/).
fn discovery_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DISCOVERY_DIR)
}

/// Read all discovery files from the discovery directory.
fn read_discovery_files() -> Result<Vec<PluginDiscovery>, Box<dyn Error>> {
    let dir = discovery_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut plugins = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<PluginDiscovery>(&contents) {
                Ok(discovery) => plugins.push(discovery),
                Err(e) => {
                    eprintln!(
                        "Warning: invalid discovery file {}: {}",
                        path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "Warning: could not read {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }
    Ok(plugins)
}

/// Check if a PID is still alive (Unix-only, kill -0).
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

/// Read discovery files and prune stale ones (dead PIDs).
fn list_live_plugins() -> Result<Vec<PluginDiscovery>, Box<dyn Error>> {
    let plugins = read_discovery_files()?;
    let dir = discovery_dir();
    let mut live = Vec::new();

    for plugin in plugins {
        if is_pid_alive(plugin.pid) {
            live.push(plugin);
        } else {
            // Stale discovery file — remove it
            let stale_path = dir.join(format!("{}.json", plugin.plugin_id));
            if stale_path.exists() {
                let _ = fs::remove_file(&stale_path);
                eprintln!(
                    "Pruned stale discovery for {} (PID {} not running)",
                    plugin.plugin_id, plugin.pid
                );
            }
        }
    }

    Ok(live)
}

/// Find a plugin by exact ID or prefix match.
fn find_plugin(plugin_id: &str) -> Result<PluginDiscovery, Box<dyn Error>> {
    let plugins = list_live_plugins()?;

    // Exact match first
    if let Some(p) = plugins.iter().find(|p| p.plugin_id == plugin_id) {
        return Ok(clone_discovery(p));
    }

    // Prefix match
    let matches: Vec<&PluginDiscovery> = plugins
        .iter()
        .filter(|p| p.plugin_id.starts_with(plugin_id))
        .collect();

    match matches.len() {
        0 => Err(format!("No plugin found matching '{}'", plugin_id).into()),
        1 => Ok(clone_discovery(matches[0])),
        _ => {
            let ids: Vec<&str> = matches.iter().map(|p| p.plugin_id.as_str()).collect();
            Err(format!(
                "Ambiguous plugin ID '{}', matches: {}",
                plugin_id,
                ids.join(", ")
            )
            .into())
        }
    }
}

/// Clone a PluginDiscovery (manual since we can't derive Clone on the outer struct easily).
fn clone_discovery(p: &PluginDiscovery) -> PluginDiscovery {
    PluginDiscovery {
        plugin_id: p.plugin_id.clone(),
        plugin_name: p.plugin_name.clone(),
        version: p.version.clone(),
        port: p.port,
        pid: p.pid,
        started_at: p.started_at.clone(),
        routes: p.routes.clone(),
    }
}

// =============================================================================
// Plugin HTTP call
// =============================================================================

/// Response envelope from plugin servers: { status: "success"|"error", data: ... }
#[derive(Debug, Deserialize)]
struct PluginResponse {
    pub status: Status,
    pub data: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// Call a plugin server route and return the `data` field from the response.
async fn call_plugin(
    plugin: &PluginDiscovery,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let client = EagleClient::new("127.0.0.1", plugin.port);

    // Build URI directly (plugin servers don't use /api/{resource}/{action} format)
    let uri: hyper::Uri = format!("http://127.0.0.1:{}{}", plugin.port, path).parse()?;

    let http_method: hyper::Method = method.to_uppercase().parse()?;
    let http_body = match body {
        Some(json_str) => hyper::Body::from(json_str.to_string()),
        None => hyper::Body::empty(),
    };

    let response: PluginResponse = client.execute_request(uri, http_method, http_body).await?;

    match response.status {
        Status::Success => Ok(response.data.unwrap_or(serde_json::Value::Null)),
        Status::Error => {
            let msg = response
                .message
                .unwrap_or_else(|| "Plugin returned error".to_string());
            Err(msg.into())
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_dir_contains_expected_path() {
        let dir = discovery_dir();
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains(".eagle-plugins/servers"));
    }

    #[test]
    fn plugin_response_success_roundtrip() {
        let json = r#"{
            "status": "success",
            "data": { "pluginId": "test", "selectedItems": 3 }
        }"#;
        let resp: PluginResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(resp.status, Status::Success));
        assert!(resp.data.is_some());
    }

    #[test]
    fn plugin_response_error_roundtrip() {
        let json = r#"{
            "status": "error",
            "message": "No items selected"
        }"#;
        let resp: PluginResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(resp.status, Status::Error));
        assert_eq!(resp.message.as_deref(), Some("No items selected"));
    }

    #[test]
    fn clone_discovery_preserves_all_fields() {
        let original = PluginDiscovery {
            plugin_id: "abc-123".to_string(),
            plugin_name: "Test".to_string(),
            version: "1.0.0".to_string(),
            port: 41600,
            pid: 99999,
            started_at: "2025-01-01T00:00:00Z".to_string(),
            routes: vec![crate::lib::types::PluginRoute {
                method: "GET".to_string(),
                path: "/health".to_string(),
            }],
        };
        let cloned = clone_discovery(&original);
        assert_eq!(cloned.plugin_id, original.plugin_id);
        assert_eq!(cloned.port, original.port);
        assert_eq!(cloned.routes.len(), 1);
    }
}
