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
    /// Batch operation partially succeeded (some items failed).
    pub const PARTIAL: i32 = 4;
}

/// Supported output formats.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Compact,
    Ndjson,
    Table,
    /// Comma-separated values with header row.
    Csv,
    /// One ID per line (extracts the `id` field from each object).
    Id,
    /// One path per line (extracts the `path` field from each object).
    Path,
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
    /// Null-delimited output instead of newlines (`--print0`).
    pub print0: bool,
    /// Preview changes without executing (`--dry-run`).
    pub dry_run: bool,
    /// Suppress non-essential stderr output (`--quiet`).
    pub quiet: bool,
    /// jq filter expression (`--jq`). When set, bypasses format/fields and outputs raw JSON.
    pub jq: Option<String>,
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
            "csv" => OutputFormat::Csv,
            "id" => OutputFormat::Id,
            "path" => OutputFormat::Path,
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
    let print0 = matches.get_flag("print0");
    let dry_run = matches.get_flag("dry-run");
    let quiet = matches.get_flag("quiet");
    let jq = matches.get_one::<String>("jq").cloned();

    OutputConfig {
        format,
        explicit,
        fields,
        count,
        no_header,
        print0,
        dry_run,
        quiet,
        jq,
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
// jq filter (via jaq)
// ---------------------------------------------------------------------------

/// Apply a jq expression to a JSON value using the jaq engine.
///
/// Returns a vector of results (jq can produce multiple outputs).
/// When used via `--jq`, the output bypasses the normal format pipeline
/// and prints raw JSON results directly.
pub fn apply_jq_filter(input: &Value, filter_expr: &str) -> Result<Vec<Value>, String> {
    use jaq_core::{load, Compiler, Ctx, RcIter};
    use jaq_json::Val;
    use load::{Arena, File, Loader};

    let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
    let arena = Arena::default();
    let program = File {
        code: filter_expr,
        path: (),
    };
    let modules = loader
        .load(&arena, program)
        .map_err(|errs| format!("jq parse error: {:?}", errs))?;
    let filter = Compiler::default()
        .with_funs(jaq_std::funs().chain(jaq_json::funs()))
        .compile(modules)
        .map_err(|errs| format!("jq compile error: {:?}", errs))?;

    let inputs = RcIter::new(core::iter::empty());
    let input_val = Val::from(input.clone());
    let mut results = Vec::new();

    for item in filter.run((Ctx::new([], &inputs), input_val)) {
        match item {
            Ok(val) => {
                let json_val: Value = Value::from(val);
                results.push(json_val);
            }
            Err(err) => return Err(format!("jq runtime error: {}", err)),
        }
    }

    Ok(results)
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
    // 0. --jq: apply filter and output raw JSON results, bypassing format/fields/count
    if let Some(ref expr) = config.jq {
        let results = apply_jq_filter(&value, expr).map_err(|e| -> Box<dyn std::error::Error> {
            eprintln!("Error: {}", e);
            std::process::exit(exit_code::USAGE);
        })?;
        let mut out = io::stdout().lock();
        match results.len() {
            0 => { /* no output */ }
            1 => {
                serde_json::to_writer_pretty(&mut out, &results[0])?;
                writeln!(out)?;
            }
            _ => {
                for r in &results {
                    serde_json::to_writer_pretty(&mut out, r)?;
                    writeln!(out)?;
                }
            }
        }
        return Ok(());
    }

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
        OutputFormat::Csv => {
            render_csv(value, config.no_header, &mut out)?;
        }
        OutputFormat::Id => {
            render_field_lines(value, "id", config.print0, &mut out)?;
        }
        OutputFormat::Path => {
            render_field_lines(value, "path", config.print0, &mut out)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Field-line rendering (id / path formats)
// ---------------------------------------------------------------------------

/// Render a single field from each object, one per line.
///
/// - Array of objects: extract `field_name` from each, print one per line.
/// - Single object: extract `field_name`, print it.
/// - Scalar string: print it directly (supports raw string arrays).
fn render_field_lines<W: Write>(
    value: &Value,
    field_name: &str,
    print0: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let delim: &[u8] = if print0 { b"\0" } else { b"\n" };

    match value {
        Value::Array(arr) => {
            for item in arr {
                let text = match item {
                    Value::Object(map) => match map.get(field_name) {
                        Some(Value::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    },
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                out.write_all(text.as_bytes())?;
                out.write_all(delim)?;
            }
        }
        Value::Object(map) => {
            let text = match map.get(field_name) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            };
            out.write_all(text.as_bytes())?;
            out.write_all(delim)?;
        }
        Value::String(s) => {
            out.write_all(s.as_bytes())?;
            out.write_all(delim)?;
        }
        _ => {
            out.write_all(value.to_string().as_bytes())?;
            out.write_all(delim)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// String-lines output (for file paths, folder names, etc.)
// ---------------------------------------------------------------------------

/// Print a list of plain strings through the output pipeline.
///
/// This is used by default-mode handlers (e.g. `item list`, `folder list`) that
/// produce derived strings (file paths, folder names) rather than JSON objects.
/// It respects `--count`, `--print0`, and structured format overrides.
pub fn output_lines(
    lines: &[String],
    config: &OutputConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // --count: output count and return early
    if config.count {
        println!("{}", lines.len());
        return Ok(());
    }

    // If an explicit structured format is requested, serialise the string list as JSON
    match config.format {
        OutputFormat::Json => {
            let value = serde_json::to_value(lines)?;
            let mut out = io::stdout().lock();
            serde_json::to_writer_pretty(&mut out, &value)?;
            writeln!(out)?;
        }
        OutputFormat::Compact => {
            let value = serde_json::to_value(lines)?;
            let mut out = io::stdout().lock();
            serde_json::to_writer(&mut out, &value)?;
            writeln!(out)?;
        }
        OutputFormat::Ndjson => {
            let mut out = io::stdout().lock();
            for line in lines {
                serde_json::to_writer(&mut out, line)?;
                writeln!(out)?;
            }
        }
        _ => {
            // Table, Id, Path all fall through to plain line output
            let mut out = io::stdout().lock();
            let delim: &[u8] = if config.print0 { b"\0" } else { b"\n" };
            for line in lines {
                out.write_all(line.as_bytes())?;
                out.write_all(delim)?;
            }
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

// ---------------------------------------------------------------------------
// CSV rendering
// ---------------------------------------------------------------------------

/// Render a JSON value as CSV (RFC 4180 quoting).
///
/// - Array of objects: columns from first object's keys.
/// - Single object: two-column KEY,VALUE layout.
/// - Scalars / non-object arrays: fall back to JSON pretty-print.
fn render_csv<W: Write>(
    value: &Value,
    no_header: bool,
    out: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        Value::Array(arr) if !arr.is_empty() && arr[0].is_object() => {
            let columns: Vec<String> = if let Value::Object(first) = &arr[0] {
                first.keys().cloned().collect()
            } else {
                return Ok(());
            };

            if !no_header {
                let header: String = columns
                    .iter()
                    .map(|c| csv_escape(c))
                    .collect::<Vec<_>>()
                    .join(",");
                writeln!(out, "{}", header)?;
            }

            for item in arr {
                let row: String = columns
                    .iter()
                    .map(|col| {
                        let cell = format_cell(item.get(col).unwrap_or(&Value::Null));
                        csv_escape(&cell)
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                writeln!(out, "{}", row)?;
            }
        }
        Value::Object(map) => {
            if !no_header {
                writeln!(out, "key,value")?;
            }
            for (key, val) in map {
                let cell = format_cell(val);
                writeln!(out, "{},{}", csv_escape(key), csv_escape(&cell))?;
            }
        }
        _ => {
            serde_json::to_writer_pretty(&mut *out, value)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

/// RFC 4180 CSV field escaping: quote the field if it contains comma, double-quote, or newline.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
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
/// When `json_mode` is true, outputs structured JSON error.
/// Otherwise, outputs plain text prefixed with "Error: ".
pub fn output_error(message: &str, json_mode: bool) {
    if json_mode {
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
            print0: false,
            dry_run: false,
            quiet: false,
            jq: None,
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
            print0: false,
            dry_run: false,
            quiet: false,
            jq: None,
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

    // ---- render_field_lines (Id/Path formats) --------------------------------

    #[test]
    fn render_field_lines_array_of_objects_id() {
        let value = json!([
            {"id": "AAA", "name": "Alpha"},
            {"id": "BBB", "name": "Beta"}
        ]);
        let mut buf = Vec::new();
        render_field_lines(&value, "id", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "AAA\nBBB\n");
    }

    #[test]
    fn render_field_lines_array_of_objects_path() {
        let value = json!([
            {"id": "X", "path": "/a/b/c.png"},
            {"id": "Y", "path": "/d/e/f.jpg"}
        ]);
        let mut buf = Vec::new();
        render_field_lines(&value, "path", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "/a/b/c.png\n/d/e/f.jpg\n");
    }

    #[test]
    fn render_field_lines_print0() {
        let value = json!([
            {"id": "A"},
            {"id": "B"}
        ]);
        let mut buf = Vec::new();
        render_field_lines(&value, "id", true, &mut buf).unwrap();
        let result = buf;
        // Should use null bytes instead of newlines
        assert_eq!(result, b"A\0B\0");
    }

    #[test]
    fn render_field_lines_missing_field() {
        let value = json!([{"name": "Alpha"}, {"name": "Beta"}]);
        let mut buf = Vec::new();
        render_field_lines(&value, "id", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        // Missing fields yield empty strings
        assert_eq!(result, "\n\n");
    }

    #[test]
    fn render_field_lines_single_object() {
        let value = json!({"id": "SINGLE"});
        let mut buf = Vec::new();
        render_field_lines(&value, "id", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "SINGLE\n");
    }

    #[test]
    fn render_field_lines_string_array() {
        let value = json!(["alpha", "beta", "gamma"]);
        let mut buf = Vec::new();
        render_field_lines(&value, "id", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "alpha\nbeta\ngamma\n");
    }

    #[test]
    fn render_field_lines_scalar() {
        let value = json!("single-value");
        let mut buf = Vec::new();
        render_field_lines(&value, "id", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "single-value\n");
    }

    #[test]
    fn render_field_lines_numeric_field() {
        let value = json!([{"id": 42}, {"id": 99}]);
        let mut buf = Vec::new();
        render_field_lines(&value, "id", false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result, "42\n99\n");
    }

    // ---- OutputConfig with new fields ----------------------------------------

    #[test]
    fn output_config_print0_dry_run_quiet() {
        let config = OutputConfig {
            format: OutputFormat::Id,
            explicit: true,
            fields: None,
            count: false,
            no_header: false,
            print0: true,
            dry_run: true,
            quiet: true,
            jq: None,
        };
        assert!(config.print0);
        assert!(config.dry_run);
        assert!(config.quiet);
        assert_eq!(config.format, OutputFormat::Id);
    }

    #[test]
    fn output_format_id_path_variants() {
        assert_eq!(OutputFormat::Id, OutputFormat::Id);
        assert_eq!(OutputFormat::Path, OutputFormat::Path);
        assert_ne!(OutputFormat::Id, OutputFormat::Path);
        assert_ne!(OutputFormat::Id, OutputFormat::Json);
    }

    // ---- output_error --------------------------------------------------------

    #[test]
    fn output_error_plain_text() {
        // output_error writes to stderr; just verify it doesn't panic
        output_error("test error", false);
    }

    #[test]
    fn output_error_json_mode() {
        // output_error writes to stderr; just verify it doesn't panic
        output_error("test error", true);
    }

    // ---- CSV rendering -------------------------------------------------------

    #[test]
    fn csv_array_of_objects() {
        let value = json!([
            {"id": "a", "name": "Alpha"},
            {"id": "b", "name": "Beta"}
        ]);
        let mut buf = Vec::new();
        render_csv(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.starts_with("id,name\n"));
        assert!(result.contains("a,Alpha\n"));
        assert!(result.contains("b,Beta\n"));
    }

    #[test]
    fn csv_array_no_header() {
        let value = json!([{"id": "a", "name": "Alpha"}]);
        let mut buf = Vec::new();
        render_csv(&value, true, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(!result.contains("id,name"));
        assert!(result.contains("a,Alpha"));
    }

    #[test]
    fn csv_single_object() {
        let value = json!({"id": "abc", "name": "Test"});
        let mut buf = Vec::new();
        render_csv(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.starts_with("key,value\n"));
        assert!(result.contains("id,abc"));
        assert!(result.contains("name,Test"));
    }

    #[test]
    fn csv_escape_comma() {
        assert_eq!(csv_escape("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn csv_escape_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn csv_field_with_comma_in_data() {
        let value = json!([{"tags": "a, b, c", "id": "1"}]);
        let mut buf = Vec::new();
        render_csv(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        // The tags field should be quoted because it contains commas
        assert!(result.contains("\"a, b, c\""));
    }

    #[test]
    fn csv_scalar_fallback() {
        let value = json!(42);
        let mut buf = Vec::new();
        render_csv(&value, false, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert_eq!(result.trim(), "42");
    }

    #[test]
    fn output_format_csv_variant() {
        assert_eq!(OutputFormat::Csv, OutputFormat::Csv);
        assert_ne!(OutputFormat::Csv, OutputFormat::Table);
    }

    // ---- jq filter -----------------------------------------------------------

    #[test]
    fn jq_identity_filter() {
        let input = json!({"id": "abc", "name": "test"});
        let results = apply_jq_filter(&input, ".").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], input);
    }

    #[test]
    fn jq_field_access() {
        let input = json!({"id": "abc", "name": "test"});
        let results = apply_jq_filter(&input, ".name").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!("test"));
    }

    #[test]
    fn jq_array_length() {
        let input = json!([1, 2, 3, 4, 5]);
        let results = apply_jq_filter(&input, "length").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(5));
    }

    #[test]
    fn jq_array_iterator() {
        let input = json!([{"id": "a"}, {"id": "b"}]);
        let results = apply_jq_filter(&input, ".[].id").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], json!("a"));
        assert_eq!(results[1], json!("b"));
    }

    #[test]
    fn jq_select_filter() {
        let input = json!([
            {"id": "a", "star": 3},
            {"id": "b", "star": 5},
            {"id": "c", "star": 1}
        ]);
        let results = apply_jq_filter(&input, "[.[] | select(.star >= 3)]").unwrap();
        assert_eq!(results.len(), 1);
        let arr = results[0].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], json!("a"));
        assert_eq!(arr[1]["id"], json!("b"));
    }

    #[test]
    fn jq_map_construct() {
        let input = json!([{"id": "a", "name": "Alpha"}, {"id": "b", "name": "Beta"}]);
        let results = apply_jq_filter(&input, "[.[] | {id, upper: .name}]").unwrap();
        assert_eq!(results.len(), 1);
        let arr = results[0].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], json!("a"));
    }

    #[test]
    fn jq_invalid_filter() {
        let input = json!({"id": "abc"});
        let result = apply_jq_filter(&input, ".[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn jq_null_input() {
        let input = json!(null);
        let results = apply_jq_filter(&input, ".").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], json!(null));
    }

    #[test]
    fn jq_keys_filter() {
        let input = json!({"id": "abc", "name": "test", "ext": "png"});
        let results = apply_jq_filter(&input, "keys").unwrap();
        assert_eq!(results.len(), 1);
        let arr = results[0].as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }
}
