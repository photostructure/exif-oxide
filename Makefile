.PHONY: check fmt lint test clippy doc build clean fix bench sync update precommit audit

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

# Extract all ExifTool algorithms and regenerate code  
sync:
	cargo run --bin exiftool_sync extract-all
	cargo build

# Update dependencies
update:
	cargo update

# Security audit for vulnerabilities in dependencies (requires: cargo install cargo-audit)
audit:
	@command -v cargo-audit >/dev/null 2>&1 || { echo "cargo-audit not found. Install with: cargo install cargo-audit --locked"; exit 1; }
	cargo audit

# Pre-commit checks: update deps, fix code, lint, test, audit, and build
precommit: update fix lint test audit build
	@echo "✅ All pre-commit checks passed!"

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
	@echo "  sync      - Extract all ExifTool algorithms and regenerate code"
	@echo "  update    - Update dependencies"
	@echo "  audit     - Security audit for vulnerabilities in dependencies"
	@echo "  precommit - Run all pre-commit checks (update, fix, lint, test, audit, build)"