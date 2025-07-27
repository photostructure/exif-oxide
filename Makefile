.PHONY: all check fmt-check fmt lint yamllint test codegen-test fix build install doc clean patch-exiftool codegen sync update audit precommit help snapshot-generate snapshot-tests snapshots perl-deps perl-setup

# Default target: build the project
all: build

# Run all checks without modifying (for CI)
check: fmt-check lint yamllint test

# Check formatting without modifying
fmt-check:
	cargo fmt --all -- --check

# Format code
fmt:
	cargo fmt --all

# Run clippy (Rust linter)
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run yamllint on YAML files
yamllint:
	yamllint .github/ *.yml *.yaml 2>/dev/null || true

# Run unit tests only (no integration tests)
unit-test:
	cargo test --all --features test-helpers

# Run all tests including integration tests
test:
	cargo test --all --features test-helpers,integration-tests

# Run codegen tests
codegen-test:
	$(MAKE) -C codegen -f Makefile.modular test

# Fix formatting and auto-fixable clippy issues
fix:
	cargo fmt --all
	cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# Build in release mode
build:
	cargo build --release

# Install the binary locally
install:
	cargo install --path . --force

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

# Check that all Perl extractors are working correctly
check-extractors:
	$(MAKE) -C codegen -f Makefile.modular check-extractors

# Extract EXIF tags from ExifTool and regenerate Rust code
codegen:
	@echo "🔧 Running code generation..."
	@$(MAKE) --no-print-directory -C codegen -f Makefile.modular -j4 codegen
	
# Extract all ExifTool algorithms and regenerate code  
sync: codegen
	cargo build

# Generate SubDirectory coverage report
subdirectory-coverage:
	@echo "📊 Generating SubDirectory coverage report..."
	@mkdir -p codegen/coverage
	@cd codegen/extractors && perl subdirectory_discovery.pl --json 2>/dev/null > ../coverage/subdirectory_coverage.json
	@cd codegen/extractors && perl subdirectory_discovery.pl --markdown 2>/dev/null > ../../docs/reference/SUBDIRECTORY-COVERAGE.md
	@coverage=$$(jq '.summary.coverage_percentage' codegen/coverage/subdirectory_coverage.json); \
	echo "SubDirectory coverage: $$coverage%"

# Check SubDirectory coverage and warn if below threshold
check-subdirectory-coverage: subdirectory-coverage
	@coverage=$$(jq -r '.summary.coverage_percentage' codegen/coverage/subdirectory_coverage.json | cut -d. -f1); \
	if [ "$$coverage" -lt 80 ]; then \
		echo "⚠️  Warning: SubDirectory coverage is only $${coverage}%"; \
		echo "   See docs/reference/SUBDIRECTORY-COVERAGE.md for details"; \
	else \
		echo "✅ SubDirectory coverage is $${coverage}%"; \
	fi

# Set up local Perl environment and install cpanminus
perl-setup:
	@echo "Setting up local Perl environment..."
	@if [ ! -f "$$HOME/perl5/bin/cpanm" ]; then \
		echo "Installing cpanminus locally..."; \
		curl -L http://cpanmin.us | perl - -l ~/perl5 App::cpanminus local::lib; \
	fi
	@echo "Setting up local::lib..."
	@eval $$(perl -I ~/perl5/lib/perl5/ -Mlocal::lib)
	@echo "Perl environment ready. Run 'make perl-deps' to install dependencies."

# Install Perl dependencies using cpanminus
perl-deps: perl-setup
	@echo "Installing Perl dependencies from cpanfile..."
	@eval $$(perl -I ~/perl5/lib/perl5/ -Mlocal::lib) && ~/perl5/bin/cpanm --installdeps .

# Update dependencies
update:
	cargo update

upgrade-gha:
	pinact run -u || { echo "pinact not found. Install with: go install github.com/suzuki-shunsuke/pinact/cmd/pinact@latest"; exit 1; }

# Upgrade to latest versions (requires: cargo install cargo-edit)
upgrade: upgrade-gha
	@command -v cargo-upgrade >/dev/null 2>&1 || { echo "cargo-upgrade not found. Install with: cargo install cargo-edit"; exit 1; }
	cargo upgrade --incompatible

# Security audit for vulnerabilities in dependencies (requires: cargo install cargo-audit)
audit:
	@command -v cargo-audit >/dev/null 2>&1 || { echo "cargo-audit not found. Install with: cargo install cargo-audit --locked"; exit 1; }
	cargo audit

# Pre-commit checks: do everything: update deps, codegen, fix code, lint, test, audit, and build
precommit: update perl-deps codegen check-subdirectory-coverage check-extractors fix yamllint compat-gen test codegen-test audit build
	@echo "✅ precommit successful 🥳" 

# Generate ExifTool JSON reference data for compatibility testing (only missing files)
compat-gen:
	./tools/generate_exiftool_json.sh

# Force regenerate all ExifTool JSON reference data
compat-gen-force:
	./tools/generate_exiftool_json.sh --force

# Run ExifTool compatibility tests
compat-test:
	cargo test --test exiftool_compatibility_tests --features integration-tests -- --nocapture

# Run MIME type compatibility tests
test-mime-compat:
	@echo "Running MIME type compatibility tests..."
	cargo test --test mime_type_compatibility_tests --features integration-tests -- --nocapture

# Generate reference data and run compatibility tests
compat: compat-gen compat-test test-mime-compat

compat-force: compat-gen-force compat-test test-mime-compat

# Show available make targets
help:
	@echo "exif-oxide Makefile targets:"
	@echo ""
	@echo "Default:"
	@echo "  make               - Build the project (same as 'make all')"
	@echo "  make all           - Build the project"
	@echo ""
	@echo "Development:"
	@echo "  make check         - Run all checks without modifying (for CI)"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy linter"
	@echo "  make yamllint      - Run yamllint on YAML files"
	@echo "  make unit-test     - Run unit tests only (fast)"
	@echo "  make test          - Run all tests including integration tests"
	@echo "  make fix           - Fix formatting and auto-fixable issues"
	@echo "  make build         - Build in release mode"
	@echo "  make install       - Install the binary locally"
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
	@echo "  make upgrade       - Upgrade to latest dependency versions"
	@echo "  make perl-setup    - Set up local Perl environment"
	@echo "  make perl-deps     - Install Perl dependencies"
	@echo "  make audit         - Security audit for vulnerabilities"
	@echo "  make precommit     - Run full pre-commit checks"
	@echo ""
	@echo "Testing:"
	@echo "  make compat        - Run compatibility tests"
	@echo "  make compat-gen    - Generate missing ExifTool JSON reference files"
	@echo "  make compat-gen-force - Force regenerate all ExifTool JSON reference files"
	@echo "  make compat-test   - Run ExifTool compatibility tests"
	@echo "  make snapshot-tests - Run snapshot tests"
