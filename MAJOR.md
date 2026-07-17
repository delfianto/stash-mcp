# Held-back major dependency bumps

Written after the 2026-07-17 dependency sweep. These are **not** currently broken on `main` —
they're major-version bumps that `cargo upgrade` (compatible-only) correctly skipped, and that
Renovate's `renovate/non-major-dependencies` PR (mislabeled — see below) currently fails CI on.
Do not merge that PR as-is.

## reqwest 0.12 → 0.13

**Fails with:**
```
error: failed to select a version for `reqwest`.
package `stash-mcp` depends on `reqwest` with feature `rustls-tls` but `reqwest` does not have that feature.
```

reqwest 0.13 renamed its TLS feature flags. `rustls-tls` no longer exists; the replacements are:

| Old (0.12) | New (0.13) |
| --- | --- |
| `rustls-tls` | `rustls` (or `rustls-native-certs` / `rustls-no-provider` for finer control) |

**Migration**: in `Cargo.toml`, change
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```
to
```toml
reqwest = { version = "0.13", default-features = false, features = ["json", "rustls"] }
```
then `cargo build` and fix whatever else 0.13 changed (check the [reqwest 0.13
changelog](https://github.com/seanmonstar/reqwest/releases) for anything beyond the feature
rename — not fully audited here, only the compile-blocking issue was diagnosed).

## rmcp 1.8 → 2.2

Not diagnosed — no CI run has attempted this yet since Renovate's current PR only proposes
reqwest. `rmcp` is the MCP SDK this server is built on; a major bump here is likely the bigger
lift of the two. Check the [rmcp changelog/releases](https://github.com/modelcontextprotocol/rust-sdk)
for breaking changes before attempting — this server's `mcp/` module will need a compile-and-fix
pass at minimum.

## Recommended order

1. rmcp 1.8 → 2.2 first (larger, more central to the codebase — better to do it deliberately
   than as a side effect of a reqwest bump).
2. reqwest 0.12 → 0.13 (small, mechanical, per above).

## Until then

Renovate will keep proposing `reqwest to 0.13` — it doesn't know this breaks, only that it's
semver-valid. Either close each new PR as it appears, or add a `packageRules` entry to
`renovate.json` disabling major-version PRs for this repo (matching the pattern already used in
`the-bannered-mare`'s `renovate.json`) if you'd rather stop seeing them.
