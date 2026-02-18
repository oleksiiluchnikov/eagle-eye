use clap::ArgMatches;
use serde::Serialize;
use serde_json::Value;
use std::io::{self, IsTerminal, Write};

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
    Ndjson,
    Table,
}

/// Resolved output configuration from CLI flags.
///
/// Decoupled from `clap::ArgMatches` so the output pipeline can be tested
/// without constructing a real CLI invocation.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub format: OutputFormat,
    /// Whether the user explicitly requested a format (`--json` or `--output`).
    pub explicit: bool,
    /// Comma-separated field names for projection (`--fields`).
    pub fields: Option<Vec<String>>,
    /// Output count instead of data (`--count`).
    pub count: bool,
    /// Omit table headers (`--no-header`).
    pub no_header: bool,
}

/// Build an `OutputConfig` from CLI matches.
///
/// Priority: `--json` flag > `--output FORMAT` > TTY auto-detect.
/// When stdout is a terminal and no explicit format is given, defaults to Table.
/// When piped (not a terminal), defaults to Json.
pub fn resolve_config(matches: &ArgMatches) -> OutputConfig {
    let explicit = matches.get_flag("json") || matches.get_one::<String>("output").is_some();

    let format = if matches.get_flag("json") {
        OutputFormat::Json
    } else if let Some(fmt) = matches.get_one::<String>("output") {
        match fmt.as_str() {
            "compact" => OutputFormat::Compact,
            "ndjson" => OutputFormat::Ndjson,
            "table" => OutputFormat::Table,
            _ => OutputFormat::Json,
        }
    } else if io::stdout().is_terminal() {
        OutputFormat::Table
    } else {
        OutputFormat::Json
    };

    let fields = matches.get_one::<String>("fields").map(|s| {
        s.split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect()
    });

    let count = matches.get_flag("count");
    let no_header = matches.get_flag("no-header");

    OutputConfig {
        format,
        explicit,
        fields,
        count,
        no_header,
    }
}

// ---------------------------------------------------------------------------
// Field projection
// ---------------------------------------------------------------------------

/// Apply field projection to a JSON value.
///
/// - Array of objects: each object is projected.
/// - Object: projected directly.
/// - Other: returned unchanged.
fn project_fields(value: Value, fields: &[String]) -> Value {
    match value {
        Value::Array(arr) => {
            let projected: Vec<Value> =
                arr.into_iter().map(|v| project_object(v, fields)).collect();
            Value::Array(projected)
        }
        Value::Object(_) => project_object(value, fields),
        other => other,
    }
}

fn project_object(value: Value, fields: &[String]) -> Value {
    if let Value::Object(map) = value {
        let projected: serde_json::Map<String, Value> = map
            .into_iter()
            .filter(|(key, _)| fields.iter().any(|f| f == key))
            .collect();
        Value::Object(projected)
    } else {
        value
    }
}

// ---------------------------------------------------------------------------
// Count
// ---------------------------------------------------------------------------

/// Count elements in a JSON value.
fn count_value(value: &Value) -> usize {
    match value {
        Value::Array(arr) => arr.len(),
        Value::Null => 0,
        _ => 1,
    }
}

// ---------------------------------------------------------------------------
// Public output functions
// ---------------------------------------------------------------------------

/// Print serializable data to stdout respecting all output flags.
///
/// This is the **single output path** for all handlers. It ensures:
/// - `.data` is printed (never the `{status, data}` wrapper)
/// - stdout has clean, structured data only
/// - `--fields` projection is applied
/// - `--count` outputs only the count
/// - Format respects `--json` / `--output` / TTY auto-detect
pub fn output<T: Serialize>(
    data: &T,
    config: &OutputConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = serde_json::to_value(data)?;
    output_pipeline(value, config)
}

/// Print a raw `serde_json::Value` through the output pipeline.
pub fn output_value(
    value: &Value,
    config: &OutputConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    output_pipeline(value.clone(), config)
}

/// The unified output pipeline.
fn output_pipeline(value: Value, config: &OutputConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 1. --count: output count and return early
    if config.count {
        let count = count_value(&value);
        println!("{}", count);
        return Ok(());
    }

    // 2. --fields projection
    let value = match &config.fields {
        Some(fields) => project_fields(value, fields),
        None => value,
    };

    // 3. Render in requested format
    write_formatted(&value, config)
}

/// Write a Value to stdout in the configured format.
fn write_formatted(value: &Value, config: &OutputConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = io::stdout().lock();
    match config.format {
        OutputFormat::Json => {
            serde_json::to_writer_pretty(&mut out, value)?;
            writeln!(out)?;
        }
        OutputFormat::Compact => {
            serde_json::to_writer(&mut out, value)?;
            writeln!(out)?;
        }
        OutputFormat::Ndjson => {
            render_ndjson(value, &mut out)?;
        }
        OutputFormat::Table => {
            render_table(value, config.no_header, &mut out)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// NDJSON rendering
// ---------------------------------------------------------------------------

/// Render a value as newline-delimited JSON.
///
/// - Array: one compact JSON object per line.
/// - Non-array: single compact JSON line.
fn render_ndjson<W: Write>(value: &Value, out: &mut W) -> Result<(), Box<dyn std::error::Error>> {
    if let Value::Array(arr) = value {
        for item in arr {
            serde_json::to_writer(&mut *out, item)?;
            writeln!(out)?;
        }
    } else {
        serde_json::to_writer(&mut *out, value)?;
        writeln!(out)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Table rendering (zero external dependencies)
// ---------------------------------------------------------------------------

/// Render a JSON value as an aligned table.
///
/// - Array of objects: columns from first object's keys.
/// - Single object: two-column KEY / VALUE layout.
/// - Scalars / non-object arrays: fall back to JSON pretty-print.
fn render_table<W: Write>(
    value: &Value,
    no_header: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        Value::Array(arr) if !arr.is_empty() && arr[0].is_object() => {
            render_object_array_table(arr, no_header, out)?;
        }
        Value::Object(map) => {
            render_single_object_table(map, no_header, out)?;
        }
        _ => {
            serde_json::to_writer_pretty(&mut *out, value)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

/// Render an array of JSON objects as an aligned table.
fn render_object_array_table<W: Write>(
    arr: &[Value],
    no_header: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let columns: Vec<String> = if let Value::Object(first) = &arr[0] {
        first.keys().cloned().collect()
    } else {
        return Ok(());
    };

    let rows: Vec<Vec<String>> = arr
        .iter()
        .map(|item| {
            columns
                .iter()
                .map(|col| format_cell(item.get(col).unwrap_or(&Value::Null)))
                .collect()
        })
        .collect();

    // Column widths: max of header and all cell values, capped at 60.
    let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() && cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }
    for w in &mut widths {
        if *w > 60 {
            *w = 60;
        }
    }

    if !no_header {
        let header: String = columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let w = widths[i];
                let display = truncate_str(&col.to_uppercase(), w);
                format!("{:<width$}", display, width = w)
            })
            .collect::<Vec<_>>()
            .join("  ");
        writeln!(out, "{}", header)?;

        let sep: String = widths
            .iter()
            .map(|w| "-".repeat(*w))
            .collect::<Vec<_>>()
            .join("  ");
        writeln!(out, "{}", sep)?;
    }

    for row in &rows {
        let line: String = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = if i < widths.len() {
                    widths[i]
                } else {
                    cell.len()
                };
                let display = truncate_str(cell, w);
                format!("{:<width$}", display, width = w)
            })
            .collect::<Vec<_>>()
            .join("  ");
        writeln!(out, "{}", line)?;
    }

    Ok(())
}

/// Render a single JSON object as a two-column KEY / VALUE table.
fn render_single_object_table<W: Write>(
    map: &serde_json::Map<String, Value>,
    no_header: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let key_width = map.keys().map(|k| k.len()).max().unwrap_or(0);

    if !no_header {
        writeln!(out, "{:<width$}  VALUE", "KEY", width = key_width)?;
        writeln!(out, "{}  {}", "-".repeat(key_width), "-".repeat(40))?;
    }

    for (key, val) in map {
        let cell = format_cell(val);
        let display = truncate_str(&cell, 60);
        writeln!(out, "{:<width$}  {}", key, display, width = key_width)?;
    }

    Ok(())
}

/// Format a JSON value as a compact cell string for table display.
fn format_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Truncate a string to a maximum width, appending `~` if truncated.
fn truncate_str(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 1 {
        format!("{}~", &s[..max_width - 1])
    } else {
        "~".to_string()
    }
}

// ---------------------------------------------------------------------------
// Plain & error output helpers
// ---------------------------------------------------------------------------

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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ---- Field Projection -------------------------------------------------

    #[test]
    fn project_fields_object() {
        let value = json!({"id": "abc", "name": "test", "ext": "png"});
        let fields = vec!["id".to_string(), "name".to_string()];
        let result = project_fields(value, &fields);
        assert_eq!(result, json!({"id": "abc", "name": "test"}));
    }

    #[test]
    fn project_fields_array() {
        let value = json!([
            {"id": "a", "name": "alpha", "ext": "png"},
            {"id": "b", "name": "beta", "ext": "jpg"}
        ]);
        let fields = vec!["id".to_string(), "ext".to_string()];
        let result = project_fields(value, &fields);
        assert_eq!(
            result,
            json!([{"id": "a", "ext": "png"}, {"id": "b", "ext": "jpg"}])
        );
    }

    #[test]
    fn project_fields_missing_key() {
        let value = json!({"id": "abc", "name": "test"});
        let fields = vec!["id".to_string(), "nonexistent".to_string()];
        let result = project_fields(value, &fields);
        assert_eq!(result, json!({"id": "abc"}));
    }

    #[test]
    fn project_fields_scalar_unchanged() {
        let value = json!(42);
        let fields = vec!["id".to_string()];
        let result = project_fields(value.clone(), &fields);
        assert_eq!(result, value);
    }

    #[test]
    fn project_fields_empty_fields() {
        let value = json!({"id": "abc", "name": "test"});
        let fields: Vec<String> = vec![];
        let result = project_fields(value, &fields);
        assert_eq!(result, json!({}));
    }

    // ---- Count ------------------------------------------------------------

    #[test]
    fn count_array() {
        assert_eq!(count_value(&json!([1, 2, 3, 4, 5])), 5);
    }

    #[test]
    fn count_empty_array() {
        assert_eq!(count_value(&json!([])), 0);
    }

    #[test]
    fn count_object() {
        assert_eq!(count_value(&json!({"id": "abc"})), 1);
    }

    #[test]
    fn count_null() {
        assert_eq!(count_value(&json!(null)), 0);
    }

    #[test]
    fn count_scalar() {
        assert_eq!(count_value(&json!(42)), 1);
    }

    // ---- Format Cell ------------------------------------------------------

    #[test]
    fn format_cell_string() {
        assert_eq!(format_cell(&json!("hello")), "hello");
    }

    #[test]
    fn format_cell_number() {
        assert_eq!(format_cell(&json!(42)), "42");
    }

    #[test]
    fn format_cell_bool() {
        assert_eq!(format_cell(&json!(true)), "true");
    }

    #[test]
    fn format_cell_null() {
        assert_eq!(format_cell(&json!(null)), "");
    }

    #[test]
    fn format_cell_array() {
        assert_eq!(format_cell(&json!(["a", "b", "c"])), "a, b, c");
    }

    #[test]
    fn format_cell_object() {
        let val = json!({"key": "val"});
        let result = format_cell(&val);
        assert!(result.contains("key"));
        assert!(result.contains("val"));
    }

    // ---- Truncate ---------------------------------------------------------

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_str("hi", 10), "hi");
    }

    #[test]
    fn truncate_exact_width() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate_str("hello world", 6), "hello~");
    }

    #[test]
    fn truncate_width_one() {
        assert_eq!(truncate_str("hello", 1), "~");
    }

    // ---- Table Rendering --------------------------------------------------

    #[test]
    fn render_table_array_of_objects() {
        let value = json!([
            {"id": "a", "name": "Alpha"},
            {"id": "b", "name": "Beta"}
        ]);
        let mut buf = Vec::new();
        render_table(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("ID"));
        assert!(result.contains("NAME"));
        assert!(result.contains("Alpha"));
        assert!(result.contains("Beta"));
        assert!(result.contains("--"));
    }

    #[test]
    fn render_table_array_no_header() {
        let value = json!([{"id": "a", "name": "Alpha"}]);
        let mut buf = Vec::new();
        render_table(&value, true, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(!result.contains("--"));
        assert!(result.contains("Alpha"));
    }

    #[test]
    fn render_table_single_object() {
        let value = json!({"id": "abc", "name": "Test"});
        let mut buf = Vec::new();
        render_table(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("KEY"));
        assert!(result.contains("VALUE"));
        assert!(result.contains("abc"));
        assert!(result.contains("Test"));
    }

    #[test]
    fn render_table_scalar_fallback() {
        let value = json!(42);
        let mut buf = Vec::new();
        render_table(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result.trim(), "42");
    }

    #[test]
    fn render_table_empty_array() {
        let value = json!([]);
        let mut buf = Vec::new();
        render_table(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        // Empty array falls through to JSON fallback
        assert_eq!(result.trim(), "[]");
    }

    // ---- NDJSON -----------------------------------------------------------

    #[test]
    fn ndjson_array() {
        let value = json!([{"id": "a"}, {"id": "b"}]);
        let mut buf = Vec::new();
        render_ndjson(&value, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = result.trim().split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"a\""));
        assert!(lines[1].contains("\"b\""));
    }

    #[test]
    fn ndjson_single_object() {
        let value = json!({"id": "x"});
        let mut buf = Vec::new();
        render_ndjson(&value, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = result.trim().split('\n').collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("\"x\""));
    }

    // ---- OutputConfig construction ----------------------------------------

    #[test]
    fn output_config_defaults() {
        let config = OutputConfig {
            format: OutputFormat::Json,
            explicit: false,
            fields: None,
            count: false,
            no_header: false,
        };
        assert_eq!(config.format, OutputFormat::Json);
        assert!(!config.explicit);
        assert!(!config.count);
        assert!(!config.no_header);
    }

    #[test]
    fn output_config_with_fields() {
        let config = OutputConfig {
            format: OutputFormat::Table,
            explicit: true,
            fields: Some(vec!["id".to_string(), "name".to_string()]),
            count: false,
            no_header: false,
        };
        assert_eq!(config.fields.as_ref().unwrap().len(), 2);
    }

    // ---- Pipeline integration (output → buffer) ---------------------------

    #[test]
    fn pipeline_count_array() {
        // count_value is used by the pipeline; test it on a realistic payload
        let data = json!([{"id": "1"}, {"id": "2"}, {"id": "3"}]);
        assert_eq!(count_value(&data), 3);
    }

    #[test]
    fn pipeline_projection_then_table() {
        let value = json!([
            {"id": "a", "name": "Alpha", "ext": "png"},
            {"id": "b", "name": "Beta", "ext": "jpg"}
        ]);
        let fields = vec!["id".to_string(), "name".to_string()];
        let projected = project_fields(value, &fields);
        let mut buf = Vec::new();
        render_table(&projected, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("Alpha"));
        assert!(!result.contains("png"));
    }

    // ---- Legacy compat ----------------------------------------------------

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
