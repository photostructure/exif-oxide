.PHONY: check fmt lint test clippy doc build clean fix bench

# Run all checks without modifying (for CI)
check: fmt-check lint test

# Check formatting without modifying
fmt-check:
	cargo fmt --all -- --check

# Format code
fmt:
	cargo fmt --all

# Run clippy (Rust linter)
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo test --all

# Fix formatting and auto-fixable clippy issues
fix:
	cargo fmt --all
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Build in release mode
build:
	cargo build --release

# Generate documentation
doc:
	cargo doc --no-deps --open

# Clean build artifacts
clean:
	cargo clean

# Run benchmarks
bench:
	cargo bench

# Help
help:
	@echo "Available targets:"
	@echo "  check     - Run all checks without modifying (for CI)"
	@echo "  fmt       - Format code"
	@echo "  fmt-check - Check formatting without modifying"
	@echo "  lint      - Run clippy linter"
	@echo "  test      - Run tests"
	@echo "  fix       - Fix formatting and auto-fixable clippy issues"
	@echo "  build     - Build in release mode"
	@echo "  doc       - Generate and open documentation"
	@echo "  clean     - Clean build artifacts"
	@echo "  bench     - Run benchmarks"