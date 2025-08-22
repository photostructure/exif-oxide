.PHONY: all ast-check ast-test check check-fmt check-json fmt lint yamllint unit-test test t codegen-test fix build install doc clean clean-generated clean-all check-deps check-perl codegen sync subdirectory-coverage check-subdirectory-coverage perl-setup perl-deps update upgrade-gha upgrade audit tests precommit compat-gen compat-gen-force compat-test test-mime-compat binary-compat-test cmp compat compat-force compat-full help

# Default target: build the project
all: build

# Check for required external tools
check-deps:
	@./scripts/check-deps.sh

# Check Perl files for syntax errors
check-perl:
	@./scripts/check-perl.sh

# Run all checks without modifying (for CI)
check: check-fmt lint yamllint check-json check-perl

# Check formatting without modifying
check-fmt:
	cargo fmt --all -- --check

check-json:
	@./scripts/check-json.sh

# Format code
fmt: check-deps
	@./scripts/fmt.sh

# Run clippy (Rust linter)
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run yamllint on YAML files
yamllint: check-deps
	yamllint .github/ *.yml *.yaml 2>/dev/null || true

# Run unit tests only (no integration tests)
unit-test:
	cargo test --all --features test-helpers

# Run all tests including integration tests
test:
	cargo test --all --features test-helpers,integration-tests

t: test

ast-check:
	cargo check --package ast

ast-test:
	cargo test --package ast

# Run codegen tests
codegen-check:
	$(MAKE) -C codegen check

# Run codegen tests
codegen-test:
	$(MAKE) -C codegen test

# Fix formatting and auto-fixable clippy issues
fix: fmt
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
	$(MAKE) -C codegen clean

# Clean generated code (use with caution - requires regeneration)
clean-generated:
	rm -rf src/generated
	rm -rf codegen/generated
	codegen/scripts/exiftool-patcher-undo.sh

# Deep clean - removes all build artifacts and generated code
clean-all: clean clean-generated
	rm -rf target
	rm -rf codegen/target

# Extract EXIF tags from ExifTool and regenerate Rust code
codegen:
	@echo "🔧 Running code generation..."
	@$(MAKE) --no-print-directory -C codegen codegen
	
# Extract all ExifTool algorithms and regenerate code  
sync: codegen
	cargo build

# Generate SubDirectory coverage report
subdirectory-coverage:
	@echo "📊 Generating SubDirectory coverage report..."
	@mkdir -p codegen/coverage
	@cd codegen/extractors && perl subdirectory_discovery.pl --json 2>/dev/null > ../coverage/subdirectory_coverage.json
	@cd codegen/extractors && perl subdirectory_discovery.pl --markdown 2>/dev/null > ../../docs/generated/SUBDIRECTORY-COVERAGE.md
	@coverage=$$(jq '.summary.coverage_percentage' codegen/coverage/subdirectory_coverage.json); \
	echo "SubDirectory coverage: $$coverage%"

# Check SubDirectory coverage and warn if below threshold
check-subdirectory-coverage: subdirectory-coverage
	@coverage=$$(jq -r '.summary.coverage_percentage' codegen/coverage/subdirectory_coverage.json | cut -d. -f1); \
	if [ "$$coverage" -lt 80 ]; then \
		echo "⚠️  Warning: SubDirectory coverage is only $${coverage}%"; \
		echo "   See docs/generated/SUBDIRECTORY-COVERAGE.md for details"; \
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
upgrade: upgrade-gha check-deps
	cargo upgrade --incompatible

# Security audit for vulnerabilities in dependencies (requires: cargo install cargo-audit)
audit: check-deps
	cargo audit

# All tests including unit, integration, codegen, and compatibility tests
tests: test codegen-test compat-full

# Pre-commit checks: codegen, fix code, lint, test, audit, and build. We don't
# clean-all because that leaves the tree temporarily broken, and we don't
# upgrade because changes to Cargo.toml can cause the rust analyzer to OOM (!!)
# IMPORTANT: codegen must run BEFORE any cargo commands to ensure generated files exist
precommit: perl-deps codegen audit fix check tests build 
	@echo "✅ precommit successful 🥳"

# This should
prerelease: clean-all upgrade precommit
	@echo "✅ prerelease successful 🥳"

# Generate ExifTool JSON reference data for compatibility testing (only missing files)
compat-gen:
	./tools/generate_exiftool_json.sh

# Force regenerate all ExifTool JSON reference data
compat-gen-force:
	./tools/generate_exiftool_json.sh --force

# Run ExifTool compatibility tests
compat-test:
	cargo test --test exiftool_compatibility_tests --features integration-tests -- --nocapture

# Run ExifTool compatibility tests with custom tag filtering
# Usage: TAGS_FILTER="Composite:Lens,EXIF:Make" make compat-tags
# Usage: make compat-tags TAGS_FILTER="Composite:Duration"
compat-tags:
	@if [ -z "$(TAGS_FILTER)" ]; then \
		echo "Error: TAGS_FILTER must be set. Example: TAGS_FILTER=\"Composite:Lens,EXIF:Make\" make compat-tags"; \
		exit 1; \
	fi
	@echo "Running compatibility tests with tag filter: $(TAGS_FILTER)"
	TAGS_FILTER="$(TAGS_FILTER)" cargo test --test exiftool_compatibility_tests --features integration-tests -- --nocapture

# Run MIME type compatibility tests
test-mime-compat:
	@echo "Running MIME type compatibility tests..."
	cargo test --test mime_type_compatibility_tests --features integration-tests -- --nocapture

# Run comprehensive binary extraction compatibility tests
binary-compat-test:
	@echo "Running comprehensive binary extraction compatibility tests..."
	@echo "⏱️  Note: This test processes 344+ files and takes several minutes to complete"
	cargo test --test binary_extraction_comprehensive --features integration-tests -- --nocapture --ignored

# Compare exif-oxide output with ExifTool using the Rust comparison tool
# Usage: make cmp -- image.jpg [group_prefix]
# Examples:
#   make cmp -- image.jpg
#   make cmp -- image.jpg File:
#   make cmp -- image.jpg EXIF:
cmp:
	@cargo run --bin compare-with-exiftool -- $(filter-out $@,$(MAKECMDGOALS))

# Test binary extraction for a specific file (for debugging)
# Usage: make binary-test-file FILE=test-images/nikon/d850.nef
binary-test-file:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make binary-test-file FILE=path/to/image.ext"; \
		echo "Examples:"; \
		echo "  make binary-test-file FILE=test-images/nikon/d850.nef"; \
		echo "  make binary-test-file FILE=test-images/sony/sony_a7c_ii_02.arw"; \
		echo "  make binary-test-file FILE=test-images/canon/5d_mark_iv.cr2"; \
		exit 1; \
	fi
	@echo "🔍 Testing binary extraction for specific file: $(FILE)"
	BINARY_TEST_FILE=$(FILE) cargo test --test binary_extraction_comprehensive test_specific_binary_extraction --features integration-tests -- --nocapture

# Generate reference data and run compatibility tests
compat: compat-gen compat-test test-mime-compat

compat-force: compat-gen-force compat-test test-mime-compat

# Run all compatibility tests including binary extraction
compat-full: compat binary-compat-test

# Show available make targets
help:
	@echo "exif-oxide Makefile targets:"
	@echo ""
	@echo "Default:"
	@echo "  make               - Build the project (same as 'make all')"
	@echo "  make all           - Build the project"
	@echo ""
	@echo "Setup:"
	@echo "  make check-deps    - Check for required external tools (jq, yamllint, jsonschema, etc.)"
	@echo ""
	@echo "Development:"
	@echo "  make check         - Run all checks without modifying (for CI)"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy linter"
	@echo "  make yamllint      - Run yamllint on YAML files"
	@echo "  make check-json    - Validate JSON config files against schemas"
	@echo "  make unit-test     - Run unit tests only (fast)"
	@echo "  make test          - Run all tests including integration tests"
	@echo "  make t             - Alias for 'make test'"
	@echo "  make fix           - Fix formatting and auto-fixable issues"
	@echo "  make build         - Build in release mode"
	@echo "  make install       - Install the binary locally"
	@echo "  make doc           - Generate and open documentation"
	@echo ""
	@echo "Code Generation:"
	@echo "  make codegen       - Generate all code from ExifTool"
	@echo "  make codegen-test  - Run codegen tests"
	@echo "  make sync          - Extract all ExifTool algorithms and regenerate code"
	@echo "  make check-perl    - Check Perl files for syntax errors"
	@echo "  make subdirectory-coverage - Generate SubDirectory coverage report"
	@echo "  make check-subdirectory-coverage - Check SubDirectory coverage and warn if low"
	@echo ""
	@echo "Cleaning:"
	@echo "  make clean         - Clean build artifacts (cargo clean)"
	@echo "  make clean-generated - Clean generated code (requires regeneration)"
	@echo "  make clean-all     - Clean everything (build + generated)"
	@echo ""
	@echo "Maintenance:"
	@echo "  make update        - Update dependencies"
	@echo "  make upgrade-gha   - Upgrade GitHub Actions to latest versions"
	@echo "  make upgrade       - Upgrade to latest dependency versions"
	@echo "  make perl-setup    - Set up local Perl environment"
	@echo "  make perl-deps     - Install Perl dependencies"
	@echo "  make audit         - Security audit for vulnerabilities"
	@echo "  make precommit     - Run full pre-commit checks"
	@echo ""
	@echo "Testing:"
	@echo "  make tests         - Run all tests including unit, integration, codegen, and compatibility"
	@echo "  make compat        - Run compatibility tests"
	@echo "  make compat-gen    - Generate missing ExifTool JSON reference files"
	@echo "  make compat-gen-force - Force regenerate all ExifTool JSON reference files"
	@echo "  make compat-test   - Run ExifTool compatibility tests"
	@echo "  make compat-force  - Force regenerate and run compatibility tests"
	@echo "  make test-mime-compat - Run MIME type compatibility tests"
	@echo "  make binary-compat-test - Run comprehensive binary extraction tests"
	@echo "  make binary-test-file FILE=path - Test binary extraction for specific file (debugging)"
	@echo "  make cmp -- file [group] - Compare exif-oxide output with ExifTool (supported tags by default, --all for all tags)"
	@echo "  make compat-full   - Run all compatibility tests including binary extraction"
