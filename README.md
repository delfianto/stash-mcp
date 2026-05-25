# stash-mcp

> An MCP server for [Stash](https://github.com/stashapp/stash) — because your AI assistant deserves to know about your _extremely personal_ media library.

[![Build](https://github.com/delfianto/stash-mcp/actions/workflows/rust.yml/badge.svg)](https://github.com/delfianto/stash-mcp/actions)

A single-binary [Model Context Protocol](https://modelcontextprotocol.io) server written in Rust that lets AI assistants query and analyze a [Stash](https://github.com/stashapp/stash) instance. Performs, tags, studios, scenes — all of it, at your model's fingertips, with zero runtime dependencies and the memory footprint of a disappointed parent.

---

## Why Rust?

The Python version works great. This version:

- Ships as a **single static binary** — no interpreter, no virtualenv, no `pip install` archaeology
- Starts in milliseconds — your AI won't be waiting longer than your attention span
- Uses ~5 MB of RAM at idle — leaving the other 64 GB for Stash itself
- Panics loudly and informatively instead of silently returning `None`

---

## Features

- Full GraphQL integration with your Stash instance
- Performers, scenes, studios, and tags — all queryable with composable filters
- Rich MCP surface: **7 tools**, **6 resources**, **5 resource templates**, **4 prompts**
- Reads config from a single `.env` file — no YAML, no TOML, no XML, no regrets
- Structured logging to stderr (stdout stays clean for the MCP stdio transport)

---

## Tools

| Tool                            | Description                                                | Required args     |
| ------------------------------- | ---------------------------------------------------------- | ----------------- |
| `health_check`                  | Connectivity check — are we even talking to Stash?         | —                 |
| `get_performer_info`            | Full profile for a single performer by exact name          | `performer_name`  |
| `get_all_performers`            | List performers with advanced filtering                    | —                 |
| `get_all_scenes`                | List scenes with tag/rating/organized filters              | —                 |
| `get_all_scenes_from_performer` | Every scene for a given performer                          | `performer_name`  |
| `advanced_performer_analysis`   | Deep stats: scene count, tag frequency, similar performers | `performer_name`  |
| `batch_performer_insights`      | Aggregated insights across multiple performers at once     | `performer_names` |

### Advanced filters for `get_all_performers`

| Parameter        | Type   | Description                                     |
| ---------------- | ------ | ----------------------------------------------- |
| `favorites_only` | bool   | Limit to favorited performers (default: `true`) |
| `country`        | string | Filter by country                               |
| `ethnicity`      | string | Filter by ethnicity                             |
| `eye_color`      | string | Filter by eye color                             |
| `hair_color`     | string | Filter by hair color                            |
| `measurements`   | string | Filter by measurements                          |
| `piercings`      | string | Filter by piercings (INCLUDES match)            |
| `tattoos`        | string | Filter by tattoos (INCLUDES match)              |
| `height_cm`      | int    | Filter by height in cm                          |
| `weight`         | int    | Filter by weight in kg                          |

Numeric filters accept a `_modifier` companion (`EQUALS`, `NOT_EQUALS`, `GREATER_THAN`, `LESS_THAN`, `BETWEEN`, `NOT_BETWEEN`) and a `_value2` for range queries.

### Filters for `get_all_scenes`

| Parameter        | Type   | Description                                    |
| ---------------- | ------ | ---------------------------------------------- |
| `organized_only` | bool   | Only return organized scenes (default: `true`) |
| `include_tags`   | string | Comma-separated tag names that must be present |
| `exclude_tags`   | string | Comma-separated tag names to exclude           |
| `min_rating`     | int    | Minimum rating 0–100 (inclusive)               |
| `max_rating`     | int    | Maximum rating 0–100 (inclusive)               |

---

## Resources

Static resources — read them directly by URI:

| URI                       | Description                                    |
| ------------------------- | ---------------------------------------------- |
| `stash://performer/all`   | All favorite performers with basic info        |
| `stash://performer/stats` | Country, ethnicity, height/weight statistics   |
| `stash://studio/all`      | All favorite studios                           |
| `stash://studio/stats`    | Scene counts, rating stats, hierarchy summary  |
| `stash://tag/all`         | All favorite tags                              |
| `stash://tag/stats`       | Scene/marker association counts, tag hierarchy |

Template resources — fill in the `{placeholder}`:

| URI template                              | Description                           |
| ----------------------------------------- | ------------------------------------- |
| `stash://performer/{name}`                | Full profile for a specific performer |
| `stash://performer/country/{country}`     | Performers filtered by country        |
| `stash://performer/ethnicity/{ethnicity}` | Performers filtered by ethnicity      |
| `stash://studio/{name}`                   | Full profile for a specific studio    |
| `stash://tag/{name}`                      | Full profile for a specific tag       |

---

## Prompts

| Prompt                | Description                                            | Parameters       |
| --------------------- | ------------------------------------------------------ | ---------------- |
| `analyze-performer`   | Full performer breakdown with scene stats and insights | `performer_name` |
| `library-insights`    | Strategic overview of your entire library              | —                |
| `recommend-scenes`    | Personalised scene recommendations                     | `preferences`    |
| `discover-performers` | Performer discovery by arbitrary criteria              | `criteria`       |

---

## Configuration

Create a `.env` file (or point `--config` at any env-format file):

```env
STASH_HOST=http://localhost:9999
STASH_API_KEY=your_api_key_here
STASH_DB_API_KEY=your_db_api_key_here   # optional
FAVORITES_ONLY=true
```

| Variable           | Default                 | Description                              |
| ------------------ | ----------------------- | ---------------------------------------- |
| `STASH_HOST`       | `http://localhost:9999` | Full URL of your Stash instance          |
| `STASH_API_KEY`    | —                       | API key (Settings → Security → API Keys) |
| `STASH_DB_API_KEY` | —                       | StashDB API key (optional)               |
| `FAVORITES_ONLY`   | `true`                  | Restrict resource browsing to favorites  |

---

## Installation

### Prebuilt binary (recommended)

```bash
# clone and install to ~/.local/bin in one shot
git clone https://github.com/delfianto/stash-mcp.git
cd stash-mcp
just install
```

`just install` builds an optimised release binary and drops it at `~/.local/bin/stash-mcp`. Make sure `~/.local/bin` is on your `PATH`.

### Build manually

```bash
cargo build --release
cp target/release/stash-mcp ~/.local/bin/
```

### Available `just` recipes

```
just build            # debug build
just release          # optimised release build
just test             # unit tests (no live Stash required)
just test-integration # integration tests against a real Stash instance (requires .env)
just test-all         # everything
just lint             # clippy -D warnings
just install          # release build → ~/.local/bin/stash-mcp
just uninstall        # remove from ~/.local/bin
just ci               # test → release → install
just clean            # cargo clean
```

---

## MCP Client Configuration

Point your MCP client at the binary. Replace the path and credentials as needed.

### Claude Desktop / Cursor / VS Code (`mcp.json`)

```json
{
    "servers": {
        "stash-mcp": {
            "type": "stdio",
            "command": "/home/you/.local/bin/stash-mcp",
            "args": ["--config", "/path/to/.env"]
        }
    }
}
```

Or rely on the automatic `.env` / environment variable fallback and skip `--config`:

```json
{
    "servers": {
        "stash-mcp": {
            "type": "stdio",
            "command": "/home/you/.local/bin/stash-mcp",
            "env": {
                "STASH_HOST": "http://localhost:9999",
                "STASH_API_KEY": "your_api_key_here"
            }
        }
    }
}
```

### Config resolution order

1. `--config <path>` / `-c <path>` — explicit file
2. `./stash.env` — if present in the working directory
3. Environment variables — fallback

---

## Running

```bash
# with an explicit config file
stash-mcp --config /path/to/.env

# rely on ./stash.env or environment variables
stash-mcp

# help
stash-mcp --help
```

Logs go to **stderr**; stdout is reserved for the MCP stdio transport. Set `RUST_LOG=debug` for verbose output.

---

## Development

```bash
git clone https://github.com/delfianto/stash-mcp.git
cd stash-mcp
cp stash-mcp-python/.env.example .env   # or write your own
# edit .env ...

just test             # 100+ unit tests, no network required
just test-integration # live tests — needs a real Stash in .env
```

The integration tests skip themselves automatically when no `.env` is present, so `just test` is always safe to run in CI without credentials.

---

## Credits

This project is a Rust reimplementation inspired by the original Python MCP server by [@donlothario](https://github.com/donlothario/stash_mcp_server). The tool surface, filter design, resource URI scheme, and overall architecture are all derived from that work. If you prefer Python, want caching, or need Docker images, go use that one.

[Stash](https://github.com/stashapp/stash) itself is the open-source media server that makes any of this worth building. The real hero here is whoever organized their library well enough for an AI to say something meaningful about it.
