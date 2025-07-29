.PHONY: all check fmt-check fmt lint yamllint unit-test test t codegen-test fix build install doc clean clean-generated clean-all check-extractors codegen sync subdirectory-coverage check-subdirectory-coverage perl-setup perl-deps update upgrade-gha upgrade audit checks tests precommit compat-gen compat-gen-force compat-test test-mime-compat binary-compat-test cmp compat compat-force compat-full help

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

t: test

# Run codegen tests
codegen-test:
	$(MAKE) -C codegen -f Makefile.modular test

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
	@$(MAKE) --no-print-directory -C codegen -f Makefile.modular codegen
	
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

checks: check fmt-check lint yamllint check-subdirectory-coverage check-extractors

# All tests including unit, integration, codegen, and compatibility tests
tests: test codegen-test compat-full

# Pre-commit checks: do everything: update deps, codegen, fix code, lint, test, audit, and build
precommit: update audit perl-deps codegen fix checks tests build
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
	@echo "Development:"
	@echo "  make check         - Run all checks without modifying (for CI)"
	@echo "  make checks        - Run all checks (alias for various check targets)"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy linter"
	@echo "  make yamllint      - Run yamllint on YAML files"
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
	@echo "  make check-extractors - Check that all Perl extractors are working"
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
