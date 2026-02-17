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

# Test (71 tests in total as of now)
cargo test                    # run all tests
cargo test <test_name>        # run a single test by name
cargo test <mod>::             # run all tests in a module
cargo test -- --nocapture     # show println! output during tests

# Test coverage summary:
# - lib/types.rs: 35 tests (Order Display, QueryParams, serde round-trips)
# - lib/client.rs: 9 tests (endpoint construction, decode_body)
# - cli/item/list: 19 tests (order string parsing)
# - cli/folder/list: 6 tests (recursive printing)
```

## Project Structure

```
eagle-eye/
├── Cargo.toml               # manifest: hyper 0.14, tokio, serde, clap 4, rayon
├── src/
│   ├── main.rs              # entry point, declares mod lib + mod cli
│   ├── lib/                 # core library layer
│   │   ├── mod.rs
│   │   ├── api.rs           # per-resource request structs (Application, Folder, Item, Library)
│   │   ├── client.rs        # EagleClient: HTTP transport, endpoint builder, request execution
│   │   └── types.rs         # all serde types, query-param builders, enums
│   └── cli/                 # CLI layer (clap builder API)
│       ├── mod.rs           # subcommand router, execute()
│       ├── app.rs           # `app` subcommand
│       ├── library.rs       # `library` subcommand
│       ├── folder/          # `folder` subcommand group
│       │   ├── mod.rs
│       │   ├── rename.rs
│       │   └── list/
│       │       ├── mod.rs
│       │       └── args/
│       └── item/            # `item` subcommand group
│           ├── mod.rs
│           ├── info.rs
│           ├── thumbnail.rs
│           └── list/
│               └── mod.rs
```

## Architecture

- **`lib/client.rs`** — `EagleClient` owns an `hyper::Client<HttpConnector>` and an
  `Authority`. It builds URIs via `endpoint()` and executes typed requests via
  `execute_request<T: Deserialize>()`.
- **`lib/api.rs`** — Thin request structs (`FolderRequest<'a>`, `ItemRequest<'a>`, etc.)
  that borrow `&EagleClient` and expose async methods per Eagle API action.
- **`lib/types.rs`** — All request param structs, response result/data structs, and the
  `QueryParams` trait for URL encoding. Serde rename attributes map snake_case to the
  Eagle API's camelCase JSON.
- **`cli/`** — Each subcommand has a `build() -> Command` and an
  `async fn execute(&EagleClient, &ArgMatches) -> Result<(), Box<dyn Error>>`.

## Dependencies

| Crate            | Purpose                        |
|------------------|--------------------------------|
| hyper 0.14       | HTTP client (full features)    |
| tokio 1          | Async runtime (full features)  |
| serde / serde_json | Serialization / deserialization |
| clap 4           | CLI argument parsing (builder) |
| rayon 1.8        | Parallel iterators             |
| percent-encoding | URL-encode query parameters    |

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
