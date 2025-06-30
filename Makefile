.PHONY: check fmt-check fmt lint test fix build doc clean codegen sync update audit precommit help

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

# Extract EXIF tags from ExifTool and regenerate Rust code
codegen:
	@echo "🔄 Extracting tags from ExifTool..."
	perl codegen/extract_tables.pl > codegen/tag_tables.json
	@echo "🦀 Generating Rust code..."
	cd codegen && cargo run -- tag_tables.json --output-dir ../src/generated
	@echo "✅ Code generation complete!"

# Extract all ExifTool algorithms and regenerate code  
sync: codegen
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

# Show available targets
help:
	@echo "📋 Available targets:"
	@echo "  check      - Run all checks (format, lint, test)"
	@echo "  fmt        - Format code"
	@echo "  lint       - Run clippy linter"
	@echo "  test       - Run all tests"
	@echo "  fix        - Fix formatting and auto-fixable issues"
	@echo "  build      - Build in release mode"
	@echo "  doc        - Generate and open documentation"
	@echo "  clean      - Clean build artifacts"
	@echo "  codegen    - Extract ExifTool tags and regenerate Rust code"
	@echo "  sync       - Run codegen and build"
	@echo "  update     - Update dependencies"
	@echo "  audit      - Security audit of dependencies"
	@echo "  precommit  - Run all pre-commit checks"
