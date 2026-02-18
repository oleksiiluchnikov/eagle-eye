# eagle-eye

A command-line interface for the [Eagle App](https://eagle.cool/) written in Rust.
Wraps Eagle's local HTTP API (`localhost:41595`) with a composable, agent-friendly CLI.

> **Not affiliated with Eagle.** This is a third-party tool. Use at your own risk.

## Features

- **24/24 Eagle API endpoints** covered (application, folder, item, library, tag)
- **7 output formats**: json, compact, ndjson, table, csv, id, path
- **TTY auto-detection**: table for humans, json for pipes
- **Field projection** (`--fields id,name,tags`) to limit output
- **jq filtering** (`--jq '.[] | select(.tags | length == 0)'`) via embedded jaq engine
- **Batch operations** (`--stdin`) with partial failure tracking (exit code 4)
- **Mutation safety**: `--dry-run` on all mutations, `--force` for destructive ops
- **Idempotent imports**: `--if-exists skip|error` for add commands
- **Shell completions**: bash, zsh, fish, powershell, elvish
- **Plugin discovery**: find and call Eagle plugin servers
- **Pipe-friendly**: `--print0`, `--no-header`, `--count`, null-delimited output

## Installation

```bash
git clone https://github.com/oleksiiluchnikov/eagle-eye.git
cd eagle-eye
cargo install --path .
```

Requires [Rust](https://rustup.rs/) and a running [Eagle App](https://eagle.cool/) instance.

## Quick Start

```bash
# Check Eagle is running
eagle-eye app

# List items (table format in terminal, json when piped)
eagle-eye item list --limit 10

# Get specific fields as JSON
eagle-eye item list --json --fields id,name,ext --limit 5

# Get item count
eagle-eye item list --count

# Filter with jq
eagle-eye item list --json --jq '[.[] | select(.ext == "png") | .name]'

# Item info
eagle-eye item info ITEM_ID

# Save a thumbnail path
eagle-eye item thumbnail ITEM_ID

# Import from URL (dry-run first)
eagle-eye item add-from-url "https://example.com/image.png" "My Image" --dry-run
eagle-eye item add-from-url "https://example.com/image.png" "My Image"

# Import idempotently (skip if already exists)
eagle-eye item add-from-url "https://example.com/image.png" "My Image" --if-exists skip

# Batch update via stdin
echo '["ID1","ID2","ID3"]' | eagle-eye item update --stdin --tags "reviewed"

# Move to trash (requires --force)
eagle-eye item move-to-trash ITEM_ID --force

# List folders as tree
eagle-eye folder list

# List tags
eagle-eye tag list

# Library info
eagle-eye library info

# Save library icon
eagle-eye library icon > icon.png
```

## Subcommands

### `app`

| Command | Description |
|---------|-------------|
| `app` | Application info |
| `app --version` | Eagle version |

### `item`

| Command | Description |
|---------|-------------|
| `item list` | List items (with filters: `--ext`, `--keyword`, `--folder`, `--tags`, `--order`, `--limit`, `--offset`) |
| `item info ID` | Get item details |
| `item thumbnail ID` | Get item thumbnail path |
| `item update ID` | Update item properties (tags, annotation, url, star) |
| `item add-from-url URL NAME` | Import from URL |
| `item add-from-urls JSON` | Batch import from URLs |
| `item add-from-path PATH NAME` | Import from local file |
| `item add-from-paths JSON` | Batch import from local files |
| `item add-bookmark URL NAME` | Add a bookmark |
| `item refresh-palette ID` | Refresh color palette |
| `item refresh-thumbnail ID` | Refresh thumbnail |
| `item move-to-trash ID --force` | Move to trash (requires `--force`) |

### `folder`

| Command | Description |
|---------|-------------|
| `folder list` | List all folders (tree view) |
| `folder list-recent` | List recently used folders |
| `folder create NAME [PARENT_ID]` | Create a folder |
| `folder rename FOLDER_ID NEW_NAME` | Rename a folder |
| `folder update FOLDER_ID [NAME] [DESC] [COLOR]` | Update folder properties |

### `library`

| Command | Description |
|---------|-------------|
| `library info` | Library info |
| `library history` | Library path history |
| `library switch --path PATH` | Switch to a different library |
| `library current` | Current working library path |
| `library icon` | Library icon path |

### `tag`

| Command | Description |
|---------|-------------|
| `tag list` | List all tags |
| `tag all` | Get all tag data (tags, recent, groups, starred) |
| `tag list-recent` | Recently used tags |
| `tag groups` | Tag groups |

### `plugin`

| Command | Description |
|---------|-------------|
| `plugin list` | Discover running plugin servers |
| `plugin routes PLUGIN_ID` | List routes for a plugin |
| `plugin call PLUGIN_ID METHOD PATH` | Call a plugin endpoint |

### `completions`

```bash
# Generate shell completions (hidden command)
eagle-eye completions --shell bash >> ~/.bashrc
eagle-eye completions --shell zsh >> ~/.zshrc
eagle-eye completions --shell fish > ~/.config/fish/completions/eagle-eye.fish
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON (shorthand for `--output json`) |
| `--output FORMAT` | Output format: `json`, `compact`, `ndjson`, `table`, `csv`, `id`, `path` |
| `--fields FIELDS` | Comma-separated field projection (e.g. `--fields id,name,tags`) |
| `--count` | Print count of results instead of data |
| `--jq EXPR` | Filter output with a jq expression (bypasses `--fields`/`--output`) |
| `--no-header` | Omit table headers (for awk/cut processing) |
| `--debug` | Log HTTP request/response details to stderr |
| `--port PORT` | Eagle server port (default: 41595) |
| `--dry-run` | Preview changes without executing (mutations only) |
| `--print0` | Null-delimited output (for `xargs -0`) |
| `-q`, `--quiet` | Suppress non-essential stderr output |

## Output Formats

| Format | When to use | Example |
|--------|-------------|---------|
| `json` | Piping to jq, agent consumption | `[{"id":"X","name":"Y"}]` |
| `compact` | Minimal JSON, single line | `[{"id":"X"}]` |
| `ndjson` | Streaming, large datasets | `{"id":"X"}\n{"id":"Y"}\n` |
| `table` | Human reading (default in terminal) | Aligned columns with headers |
| `csv` | Spreadsheet import | `id,name\nX,Y` |
| `id` | Piping IDs to other commands | One ID per line |
| `path` | File operations with xargs | One path per line |

Default: `table` when stdout is a terminal, `json` when piped.

## Pipe Composition

eagle-eye is designed for shell pipelines and AI agent workflows:

```bash
# List all PNG items, get just IDs
eagle-eye item list --ext png --output id

# Count items by extension
eagle-eye item list --json --jq '[.[].ext] | group_by(.) | map({ext: .[0], count: length})'

# Refresh thumbnails for items matching a keyword
eagle-eye item list --keyword "photo" --output id | eagle-eye item refresh-thumbnail --stdin

# Export folder list as CSV
eagle-eye folder list --output csv > folders.csv

# Find untagged items
eagle-eye item list --json --jq '[.[] | select(.tags | length == 0) | .id]'

# Batch move to trash from a file of IDs
cat ids.txt | eagle-eye item move-to-trash --stdin --force

# Null-delimited output for paths with spaces
eagle-eye item list --print0 --output path | xargs -0 ls -la
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (runtime failure, API error) |
| 2 | Usage error (invalid arguments) |
| 3 | Connection error (Eagle not running) |
| 4 | Partial failure (batch: some succeeded, some failed) |

## Development

```bash
# Build
cargo build

# Run from source
cargo run -- item list --limit 5

# Format
cargo fmt

# Lint (CI-grade)
cargo clippy -- -D warnings

# Test (173 tests)
cargo test

# Build release
cargo build --release
```

## License

[MIT](https://choosealicense.com/licenses/mit/)
