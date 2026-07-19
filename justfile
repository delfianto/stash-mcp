bin := "stash-mcp"
install_dir := env("HOME") / ".local/bin"

# List available recipes
default:
    @just --list

# Build a debug binary
build:
    cargo build

# Build an optimised release binary
release:
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

# Check that the code compiles without producing a binary
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Compress the release binary with upx (skips if already packed)
compress: release
    upx -t target/release/{{ bin }} >/dev/null 2>&1 || upx --best --lzma target/release/{{ bin }}

# Build release binary and copy it to ~/.local/bin
install: compress
    @mkdir -p "{{ install_dir }}"
    cp target/release/{{ bin }} "{{ install_dir }}/{{ bin }}"
    @echo "installed → {{ install_dir }}/{{ bin }}"

# Remove the installed binary from ~/.local/bin
uninstall:
    rm -f "{{ install_dir }}/{{ bin }}"
    @echo "removed {{ install_dir }}/{{ bin }}"

# Build, test, then install
ci: test release install

# Remove build artefacts
clean:
    cargo clean
