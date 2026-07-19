# stash-mcp — baseline + integration test helpers

bins    := "stash-mcp"
bin_dir := env_var("HOME") / ".local/bin"
sys_dir := "/usr/local/bin"

# List available recipes
default:
    @just --list

# Build release binaries
build:
    cargo build --release

# Run unit/integration tests that do not need live external services
test:
    cargo test

# Run only the integration tests (requires .env with live credentials)
test-integration:
    cargo test integration -- --nocapture

# Run every test: unit + integration (nocapture)
test-all:
    cargo test -- --nocapture

# Auto-format the tree
fmt:
    cargo fmt --all

# Check formatting (CI gate)
fmt-check:
    cargo fmt --all -- --check

# Lint — warnings denied (CI gate)
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Full local gate, mirrors CI (fmt + clippy + tests)
check: fmt-check lint test

# Compress every release binary with upx (skips a binary if already packed)
compress: build
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v upx >/dev/null 2>&1; then
        echo "compress: upx not found in PATH" >&2
        exit 1
    fi
    for b in {{bins}}; do
        path="target/release/$b"
        if [ ! -f "$path" ]; then
            echo "compress: missing $path (is bins= correct?)" >&2
            exit 1
        fi
        upx -t "$path" >/dev/null 2>&1 || upx --best --lzma "$path"
        echo "compressed $path"
    done

# Install into ~/.local/bin (default) or /usr/local/bin (--system, via sudo)
install *flags: compress
    #!/usr/bin/env bash
    set -euo pipefail
    dir="{{bin_dir}}"
    sudo=""
    for f in {{flags}}; do
        case "$f" in
            --system) dir="{{sys_dir}}"; sudo="sudo" ;;
            *) echo "install: unknown flag '$f' (only --system is supported)" >&2; exit 1 ;;
        esac
    done
    for b in {{bins}}; do
        $sudo install -Dm755 "target/release/$b" "$dir/$b"
        echo "installed $dir/$b"
    done

# Remove installed binaries (pass --system for /usr/local/bin via sudo)
uninstall *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="{{bin_dir}}"
    sudo=""
    for f in {{flags}}; do
        case "$f" in
            --system) dir="{{sys_dir}}"; sudo="sudo" ;;
            *) echo "uninstall: unknown flag '$f' (only --system is supported)" >&2; exit 1 ;;
        esac
    done
    for b in {{bins}}; do
        $sudo rm -f "$dir/$b"
        echo "removed $dir/$b"
    done

# Remove build artifacts
clean:
    cargo clean
