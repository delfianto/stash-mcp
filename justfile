bin     := "stash-mcp"
bin_dir := env_var("HOME") / ".local/bin"
sys_dir := "/usr/local/bin"

# List available recipes
default:
    @just --list

# Build release binary
build:
    cargo build --release

# Run all unit tests (no live Stash required)
test:
    cargo test

# Run only the integration tests (requires .env with live credentials)
test-integration:
    cargo test integration -- --nocapture

# Run every test: unit + integration
test-all:
    cargo test -- --nocapture

# Lint — treat warnings as errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Build, test, and lint in one shot
check: build test lint

# Compress the release binary with upx (skips if already packed)
compress: build
    upx -t target/release/{{bin}} >/dev/null 2>&1 || upx --best --lzma target/release/{{bin}}

# Install stash-mcp into ~/.local/bin (pass --system for /usr/local/bin via sudo)
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
    $sudo install -Dm755 target/release/{{bin}} "$dir/{{bin}}"
    echo "installed $dir/{{bin}}"

# Remove installed binary (pass --system for /usr/local/bin via sudo)
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
    $sudo rm -f "$dir/{{bin}}"
    echo "removed $dir/{{bin}}"

# Remove build artifacts
clean:
    cargo clean
