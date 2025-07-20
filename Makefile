# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run clippy linter
lint:
	cargo clippy --all-targets --all-features

# Format code
fmt:
	cargo fmt

# Run all checks
check: lint test

# Create release build
release: check build

# Install locally
install:
	cargo install --path .

# Clean build artifacts
clean:
	cargo clean

# Run with example
run-example:
	cargo run -- generate "AI-powered photo editor" --count 5 --style creative --tlds com,io,ai --check

# Run interactive mode
interactive:
	cargo run -- interactive

# Show help
help:
	cargo run -- --help

# Generate documentation
docs:
	cargo doc --open

# Run with debug logging
debug:
	RUST_LOG=debug cargo run -- generate "startup app" --verbose

# Create sample config
sample-config:
	echo '# Sample configuration file' > sample-config.toml
	cat domain-forge.toml >> sample-config.toml

.PHONY: build test lint fmt check release install clean run-example interactive help docs debug sample-config