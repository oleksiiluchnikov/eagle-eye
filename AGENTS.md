# AGENTS.md - eagle-eye

CLI tool for the [Eagle App](https://eagle.cool/) written in Rust (edition 2021).
Wraps Eagle's local HTTP API (`localhost:41595`) with a `clap`-based CLI.

## Build / Run / Test Commands

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run
cargo run -- --help
cargo run -- app --version
cargo run -- item list --limit 10

# Install locally
cargo install --path .

# Check (type-check without producing binary — fast feedback)
cargo check

# Format
cargo fmt
cargo fmt -- --check          # CI: verify formatting without changes

# Lint
cargo clippy                  # all warnings
cargo clippy -- -D warnings   # treat warnings as errors (CI-grade)

# Test (173 tests in total as of now)
cargo test                    # run all tests
cargo test <test_name>        # run a single test by name
cargo test <mod>::             # run all tests in a module
cargo test -- --nocapture     # show println! output during tests

# Test coverage summary:
# - lib/types.rs: 38 tests (Order Display, QueryParams, serde round-trips, GetLibraryIconParams)
# - lib/client.rs: 12 tests (endpoint construction, decode_body)
# - cli/item/list: 15 tests (order string parsing)
# - cli/item/move_to_trash: 7 tests (stdin ID parsing via shared module)
# - cli/folder/list: 6 tests (recursive printing)
# - cli/folder/list/args/tree.rs: 10 tests (tree rendering, TTY detection)
# - cli/output.rs: 63 tests (field projection, count, format_cell, truncate, table, CSV, NDJSON,
#     render_field_lines, OutputConfig, output_error, 9 jq filter tests)
# - cli/stdin.rs: 9 tests (JSON array, newline, null-delimited, empty, whitespace, single ID)
# - cli/plugin.rs: 4 tests (discovery, response roundtrips)
# - cli/completions.rs: 4 tests (hidden flag check, bash/zsh/fish generation)
```

## Project Structure

```
eagle-eye/
├── Cargo.toml                     # manifest: hyper 0.14, tokio, serde, clap 4, rayon, jaq, clap_complete, dirs, libc
├── src/
│   ├── main.rs                    # entry point, declares mod lib + mod cli
│   ├── lib/                       # core library layer
│   │   ├── mod.rs
│   │   ├── api.rs                 # per-resource request structs (Application, Folder, Item, Library, Tag)
│   │   ├── client.rs              # EagleClient: HTTP transport, endpoint builder, request execution
│   │   └── types.rs               # all serde types, query-param builders, enums
│   └── cli/                       # CLI layer (clap builder API)
│       ├── mod.rs                 # build_command(), get_matches(), execute(), subcommand router
│       ├── app.rs                 # `app` subcommand (info, version)
│       ├── completions.rs         # hidden `completions --shell` subcommand (bash/zsh/fish/powershell/elvish)
│       ├── library.rs             # `library` subcommand (info, history, switch, current, icon)
│       ├── output.rs              # OutputConfig, OutputFormat, output pipeline, jq filter, table/CSV/NDJSON rendering
│       ├── plugin.rs              # `plugin` subcommand (list, routes, call) — discovers plugin servers
│       ├── stdin.rs               # shared parse_ids_input(), read_ids_from_stdin() for batch ops
│       ├── tag.rs                 # `tag` subcommand (list, all, list-recent, groups)
│       ├── folder/                # `folder` subcommand group
│       │   ├── mod.rs
│       │   ├── rename.rs
│       │   └── list/
│       │       ├── mod.rs
│       │       └── args/
│       │           ├── mod.rs
│       │           └── tree.rs    # tree rendering, TTY detection
│       └── item/                  # `item` subcommand group
│           ├── mod.rs
│           ├── add_bookmark.rs
│           ├── add_from_path.rs   # --if-exists support
│           ├── add_from_paths.rs
│           ├── add_from_url.rs    # --if-exists support
│           ├── add_from_urls.rs
│           ├── info.rs
│           ├── list/
│           │   └── mod.rs
│           ├── move_to_trash.rs   # --stdin, --force
│           ├── refresh_palette.rs # --stdin, partial failure (exit 4)
│           ├── refresh_thumbnail.rs # --stdin, partial failure (exit 4)
│           ├── thumbnail.rs
│           └── update.rs          # --stdin, partial failure (exit 4)
```

## Architecture

- **`lib/client.rs`** — `EagleClient` owns an `hyper::Client<HttpConnector>` and an
  `Authority`. It builds URIs via `endpoint()` and executes typed requests via
  `execute_request<T: Deserialize>()`. For binary responses (thumbnail, icon), use
  `execute_raw_request()` which returns `Vec<u8>`.
- **`lib/api.rs`** — Thin request structs (`FolderRequest<'a>`, `ItemRequest<'a>`, etc.)
  that borrow `&EagleClient` and expose async methods per Eagle API action.
- **`lib/types.rs`** — All request param structs, response result/data structs, and the
  `QueryParams` trait for URL encoding. Serde rename attributes map snake_case to the
  Eagle API's camelCase JSON.
- **`cli/mod.rs`** — `build_command()` constructs the full clap `Command` tree (exposed
  separately for shell completion generation). `get_matches()` calls `build_command()`
  and parses args. `execute()` routes to subcommand handlers.
- **`cli/output.rs`** — `OutputConfig` struct holds all format-related flags.
  `output_pipeline()` / `output()` / `output_value()` / `output_lines()` drive the
  format pipeline. `apply_jq_filter()` embeds jaq v2 for `--jq`. Table, CSV, NDJSON
  renderers. Structured JSON errors on stderr when `--json` is active.
- **`cli/stdin.rs`** — Shared `parse_ids_input()` and `read_ids_from_stdin()` used by
  `move_to_trash`, `update`, `refresh_palette`, `refresh_thumbnail` for `--stdin` batch ops.
- **`cli/completions.rs`** — Hidden `completions --shell SHELL` subcommand that uses
  `clap_complete::generate()` with the `build_command()` output.
- **`cli/`** — Each subcommand module exposes `pub fn build() -> Command` and an
  `async fn execute(&EagleClient, &ArgMatches) -> Result<(), Box<dyn Error>>`.

## Output Pipeline

The output pipeline in `cli/output.rs` processes all command output:

1. Commands produce `serde_json::Value` (single or array)
2. If `--jq EXPR` is set: apply jaq filter and output raw JSON, bypassing all other formatting
3. If `--count` is set: output count instead of data
4. If `--fields` is set: project only requested fields from each object
5. Format according to `--output` / `--json` / TTY auto-detection:
   - `Json`: pretty-printed JSON array
   - `Compact`: minified JSON
   - `Ndjson`: one JSON object per line
   - `Table`: aligned columns with headers (default when TTY)
   - `Csv`: RFC 4180 compliant
   - `Id`: extract `id` field, one per line
   - `Path`: extract file path, one per line

### Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Shorthand for `--output json` |
| `--output FORMAT` | `json`, `compact`, `ndjson`, `table`, `csv`, `id`, `path` |
| `--fields FIELDS` | Comma-separated field projection |
| `--count` | Print count instead of data |
| `--jq EXPR` | jq filter via embedded jaq v2 (bypasses --fields/--output) |
| `--no-header` | Omit table/CSV headers |
| `--debug` | Log HTTP request/response to stderr |
| `--port PORT` | Eagle server port (default: 41595) |
| `--dry-run` | Preview mutations without executing |
| `--print0` | Null-delimited output |
| `-q` / `--quiet` | Suppress non-essential stderr |

## Exit Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | SUCCESS | Operation completed |
| 1 | ERROR | Runtime failure, API error |
| 2 | USAGE | Invalid arguments, unknown flags |
| 3 | CONNECTION | Eagle server not running, timeout |
| 4 | PARTIAL | Batch: some items succeeded, some failed |

## Batch Operations & Stdin

Commands supporting `--stdin`: `item update`, `item refresh-palette`, `item refresh-thumbnail`,
`item move-to-trash`.

The shared `cli/stdin.rs` module accepts:
- JSON array: `["ID1","ID2","ID3"]`
- Newline-delimited: one ID per line
- Null-delimited: for `--print0` piping

Batch commands track per-item success/failure and exit with code 4 on partial failure.

## Mutation Safety

| Mechanism | Where | Behavior |
|-----------|-------|----------|
| `--dry-run` | All 13 mutation handlers | Prints what WOULD happen, does not execute |
| `--force` | `item move-to-trash` | Required to confirm destructive operation |
| `--if-exists skip\|error` | `item add-from-url`, `item add-from-path` | `skip` suppresses duplicate errors, `error` (default) propagates |

## Dependencies

| Crate | Purpose |
|-------|---------|
| hyper 0.14 | HTTP client (full features) |
| tokio 1 | Async runtime (full features) |
| serde / serde_json | Serialization / deserialization |
| clap 4 | CLI argument parsing (builder pattern) |
| clap_complete 4 | Shell completion generation |
| rayon 1.8 | Parallel iterators |
| percent-encoding 2.3 | URL-encode query parameters |
| jaq-core 2 | jq filter engine (core) |
| jaq-json 1 | jq JSON value bridge (serde_json feature) |
| jaq-std 2 | jq standard library filters |
| dirs 5 | Platform-specific directories (plugin discovery) |
| libc 0.2 | TTY detection (isatty) |

## Code Style Guidelines

### Naming

- **snake_case** for functions, methods, variables, modules, file names.
- **PascalCase** for structs, enums, traits, type aliases.
- **SCREAMING_SNAKE_CASE** for constants and enum variants representing fixed strings
  (e.g., `Order::CREATEDATE`).
- Structs representing API responses follow `Get<Resource><Action>Result` /
  `<Resource><Action>Data` pattern (e.g., `GetItemListResult`, `ItemListData`).
- Request param structs: `Get<Resource><Action>Params`.

### Imports

- Group imports: `std` first, then external crates, then `super::`/`crate::` locals.
- Prefer specific imports over globs. Exception: `use super::types::*` is used in `api.rs`.
- Use `use crate::lib::...` for cross-module references from CLI to library layer.

### Types & Serde

- All API response types derive `Debug, Serialize, Deserialize`.
- Use `#[serde(rename = "camelCase")]` on fields where Eagle's JSON key differs from
  Rust's snake_case. Do NOT use `#[serde(rename_all)]` at the struct level — rename
  individual fields.
- Optional fields use `Option<T>` and will be skipped when `None` during serialization.
- Use `serde_json::Value` for untyped/opaque JSON blobs.

### Error Handling

- Return `Result<T, Box<dyn std::error::Error>>` everywhere (no custom error types yet).
- Propagate errors with `?` operator.
- For user-facing missing arguments, print a message and return `Ok(())` rather than
  returning an error.
- In `decode_body`, on parse failure, log context around the error column before
  returning `Err`.

### Async

- Runtime: `#[tokio::main]` on `main()`.
- All API methods are `async fn`.
- Use `rayon::par_iter()` for CPU-bound parallel work on collections (not tokio tasks).

### CLI (clap)

- Use the **builder pattern** (`Command::new`, `Arg::new`), not derive macros.
- Each subcommand module exposes `pub fn build() -> Command` and
  `pub async fn execute(client, matches) -> Result<...>`.
- Router in `cli/mod.rs` matches on `subcommand()` name strings.
- `build_command()` is separate from `get_matches()` so `clap_complete::generate()`
  can access the `Command` tree for shell completions.

### Query Parameters

- Implement the `QueryParams` trait (`fn to_query_string(&self) -> String`) for each
  params struct.
- Use `percent_encoding::percent_encode` with `NON_ALPHANUMERIC` for all values.
- Filter out `None` fields with `filter_map`.

### Formatting & Linting

- No `rustfmt.toml` or `clippy.toml` — use default `cargo fmt` and `cargo clippy` rules.
- Keep lines reasonable length (no hard limit configured).
- Run `cargo fmt` before committing.
- Run `cargo clippy` and fix warnings before committing.

### General

- Keep the library layer (`lib/`) free of CLI/presentation concerns.
- Keep the CLI layer (`cli/`) free of HTTP/serialization logic.
- Eagle server address is hardcoded: `localhost:41595` (`cli/mod.rs`).
- Doc comments (`///`) on public structs and methods.
- Avoid `unwrap()` in library code; acceptable only in infallible builder setups
  (e.g., `Authority::from_maybe_shared`).
- Commented-out code and `todo!()` markers exist — clean them up when touching
  those areas.
