.PHONY: check fmt-check fmt lint test fix build doc clean codegen sync update audit precommit help snapshot-generate snapshot-tests snapshots

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
	cargo test --all --features test-helpers

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
	perl codegen/extract_tables.pl > codegen/generated/tag_tables.json
	cd codegen && cargo run -- generated/tag_tables.json --output-dir ../src/generated

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

# Pre-commit checks: do everything: update deps, codegen, fix code, lint, test, audit, and build
precommit: update codegen fix lint compat-gen test audit build 

# Generate ExifTool JSON reference data for compatibility testing
compat-gen:
	./tools/generate_exiftool_json.sh

# Run ExifTool compatibility tests
compat-test:
	cargo test --test exiftool_compatibility_tests -- --nocapture

# Generate reference data and run compatibility tests
compat: compat-gen compat-test
