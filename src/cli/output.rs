use clap::ArgMatches;
use serde::Serialize;
use serde_json::Value;
use std::io::{self, Write};

/// Exit codes per agentic-cli convention.
pub mod exit_code {
    /// Operation completed successfully.
    pub const SUCCESS: i32 = 0;
    /// Runtime failure, API error, general error.
    pub const ERROR: i32 = 1;
    /// Invalid arguments, unknown flags (usage error).
    pub const USAGE: i32 = 2;
    /// Server not running, connection refused, timeout.
    pub const CONNECTION: i32 = 3;
}

/// Supported output formats.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Compact,
    // Future: Table, Csv, Ndjson, Id, Path
}

/// Resolve the output format from CLI flags.
///
/// Priority: `--json` flag > `--output FORMAT` > TTY auto-detect.
/// For now, auto-detect returns Json (table not yet implemented).
pub fn resolve_format(matches: &ArgMatches) -> OutputFormat {
    // --json is shorthand for --output json
    if matches.get_flag("json") {
        return OutputFormat::Json;
    }

    if let Some(fmt) = matches.get_one::<String>("output") {
        return match fmt.as_str() {
            "compact" => OutputFormat::Compact,
            // Default unknown formats to json
            _ => OutputFormat::Json,
        };
    }

    // TTY auto-detect: table when terminal, json when piped.
    // Since table is not implemented yet, always default to json.
    // TODO: change to `if is_terminal { Table } else { Json }` when table is implemented.
    OutputFormat::Json
}

/// Print serializable data to stdout in the requested format.
///
/// This is the **single output path** for all handlers. It ensures:
/// - Only `.data` is printed (never the `{status, data}` wrapper)
/// - stdout has clean, structured data only
/// - Format respects global `--json` / `--output` flags
pub fn output<T: Serialize>(
    data: &T,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = io::stdout().lock();
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(&mut out, data)?;
            writeln!(out)?;
        }
        OutputFormat::Compact => {
            serde_json::to_writer(&mut out, data)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

/// Print a raw `serde_json::Value` to stdout in the requested format.
///
/// Use this when you have a `Value` instead of a typed struct.
pub fn output_value(
    value: &Value,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = io::stdout().lock();
    match format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(&mut out, value)?;
            writeln!(out)?;
        }
        OutputFormat::Compact => {
            serde_json::to_writer(&mut out, value)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

/// Print a plain string to stdout (for single-value outputs like version, path, name).
pub fn output_plain(text: &str) {
    println!("{}", text);
}

/// Print an error message to stderr.
///
/// When `--json` is set, outputs structured JSON error.
/// Otherwise, outputs plain text prefixed with "Error: ".
pub fn output_error(message: &str, matches: &ArgMatches) {
    if matches.get_flag("json")
        || matches
            .get_one::<String>("output")
            .is_some_and(|f| f == "json" || f == "compact")
    {
        let err = serde_json::json!({
            "ok": false,
            "error": { "message": message }
        });
        eprintln!("{}", serde_json::to_string(&err).unwrap_or_default());
    } else {
        eprintln!("Error: {}", message);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn output_json_format() {
        let data = json!({"id": "abc", "name": "test"});
        let mut buf = Vec::new();
        serde_json::to_writer_pretty(&mut buf, &data).unwrap();
        buf.push(b'\n');
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("\"id\": \"abc\""));
        assert!(result.contains("\"name\": \"test\""));
    }

    #[test]
    fn output_compact_format() {
        let data = json!({"id": "abc"});
        let mut buf = Vec::new();
        serde_json::to_writer(&mut buf, &data).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "{\"id\":\"abc\"}");
    }
}
