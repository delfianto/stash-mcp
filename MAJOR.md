# Major dependency upgrades — done

Landed 2026-07-17. These were previously held back after a dependency sweep;
they are now on current majors on `main`.

## Completed

| Crate | From | To | Notes |
| --- | --- | --- | --- |
| `rmcp` | 1.8 | **2.2** | Model API aligned to MCP 2025-11-25; see migration discussion [#926](https://github.com/modelcontextprotocol/rust-sdk/discussions/926). Features: `server`, `transport-io` (`schemars` pulled in via `server`). |
| `reqwest` | 0.12 | **0.13** | Feature rename `rustls-tls` → `rustls`. Still `default-features = false` + `json` + `rustls`. Crypto provider is aws-lc (reqwest 0.13 default). |

### rmcp app-side changes (for archaeology)

- `Content` → `ContentBlock`
- `RawResource` / `RawResourceTemplate` + `AnnotateAble` → `Resource` / `ResourceTemplate` builders
- `ResourceContents::text(...).with_mime_type(...)` for read payloads
- `PromptMessageRole` → `Role`

### reqwest

- No source changes required beyond `Cargo.toml` feature rename.

## Renovate

Major updates remain disabled in `renovate.json` (`matchUpdateTypes: major` →
`enabled: false`). Future majors should be deliberate, not auto-PRs.
