default:
    @just --list

# Build all crates
build:
    cargo build --workspace

# Build in release mode
release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Run clippy lints
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Build, lint, and test
check: fmt-check lint test

# Run the GUI app
gui:
    cargo run -p wfc-gui

# Generate built-in sample pattern images into ./samples/
generate-samples:
    cargo run -p wfc-cli -- generate-samples

# Run the CLI (e.g. `just cli run --seed 42`, `just cli generate-samples`)
cli *ARGS:
    cargo run -p wfc-cli -- {{ARGS}}

# Build wfc-core without optional dependencies
core-minimal:
    cargo build -p wfc-core --no-default-features

# Clean build artifacts
clean:
    cargo clean
