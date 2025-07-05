.PHONY: check fmt-check fmt lint test fix build doc clean patch-exiftool codegen codegen-simple-tables sync update audit precommit help snapshot-generate snapshot-tests snapshots

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
	$(MAKE) -C codegen -f Makefile.modular clean
	@echo "Note: Generated code was not cleaned. Use 'make clean-generated' to remove it."
	@echo "      You'll need to run 'make codegen' to regenerate if you clean generated code."

# Clean generated code (use with caution - requires regeneration)
clean-generated:
	@echo "Cleaning generated code..."
	rm -rf src/generated/*
	rm -rf codegen/generated/*
	@echo "Generated code cleaned. Run 'make codegen' to regenerate."

# Deep clean - removes all build artifacts and generated code
clean-all: clean clean-generated

# Patch ExifTool modules to expose my-scoped variables
patch-exiftool:
	@echo "Patching ExifTool modules to expose my-scoped variables..."
	cd codegen && perl patch_exiftool_modules.pl
	@echo "ExifTool modules patched successfully"

# Extract simple tables from ExifTool
codegen-simple-tables:
	@echo "Extracting simple tables from ExifTool..."
	cd codegen && perl extract_simple_tables.pl > generated/simple_tables.json
	@echo "Generated: codegen/generated/simple_tables.json"

# Check that all Perl extractors are working correctly
check-extractors:
	$(MAKE) -C codegen -f Makefile.modular check-extractors

# Extract EXIF tags from ExifTool and regenerate Rust code
codegen: codegen-simple-tables
	@echo "Extracting tag tables from ExifTool..."
	perl codegen/extract_tables.pl > codegen/generated/tag_tables.json
	@echo "Generating Rust code from extractions..."
	cd codegen && cargo run -- generated/tag_tables.json --output-dir ../src/generated
	@echo "Code generation complete"

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
precommit: update codegen check-extractors fix lint compat-gen test audit build 

# Generate ExifTool JSON reference data for compatibility testing
compat-gen:
	./tools/generate_exiftool_json.sh

# Run ExifTool compatibility tests
compat-test:
	cargo test --test exiftool_compatibility_tests -- --nocapture

# Run MIME type compatibility tests
test-mime-compat:
	@echo "Running MIME type compatibility tests..."
	cargo test --test mime_type_compatibility_tests -- --nocapture

# Generate reference data and run compatibility tests
compat: compat-gen compat-test test-mime-compat

# Show available make targets
help:
	@echo "exif-oxide Makefile targets:"
	@echo ""
	@echo "Development:"
	@echo "  make check         - Run all checks without modifying (for CI)"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy linter"
	@echo "  make test          - Run tests"
	@echo "  make fix           - Fix formatting and auto-fixable issues"
	@echo "  make build         - Build in release mode"
	@echo "  make doc           - Generate and open documentation"
	@echo ""
	@echo "Code Generation:"
	@echo "  make codegen       - Generate all code from ExifTool"
	@echo "  make patch-exiftool - Patch ExifTool modules (required before codegen)"
	@echo ""
	@echo "Cleaning:"
	@echo "  make clean         - Clean build artifacts (cargo clean)"
	@echo "  make clean-generated - Clean generated code (requires regeneration)"
	@echo "  make clean-all     - Clean everything (build + generated)"
	@echo ""
	@echo "Maintenance:"
	@echo "  make update        - Update dependencies"
	@echo "  make audit         - Security audit for vulnerabilities"
	@echo "  make precommit     - Run full pre-commit checks"
	@echo ""
	@echo "Testing:"
	@echo "  make compat        - Run compatibility tests"
	@echo "  make snapshot-tests - Run snapshot tests"
