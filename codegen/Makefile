# Modular Codegen Makefile
# Simplified build system - Rust now handles extraction orchestration internally

.PHONY: all clean codegen-flocked codegen test debug-expression test-expression-file generate-expression-tests test-expressions-type

# Default target
all: codegen

# Main codegen pipeline (patches a staged ExifTool copy; the submodule is never modified)
codegen:
	@flock --exclusive --timeout=30 .make.flock $(MAKE) --no-print-directory codegen-flocked

# We can't have codegen-flocked depend on `clean`, as that breaks concurrent builds.
# TODO: When codegen is stable, switch to `cargo run --release` for faster execution.
#
# HOWTO control verbosity:
#   RUST_LOG=warn make codegen   # quiet: warnings only
#   RUST_LOG=debug make codegen  # verbose: per-module details
#   RUST_LOG=trace make codegen  # very verbose: PPI node inspection
codegen-flocked:
	@echo "🔧 Running Rust codegen against a staged ExifTool copy..." ; \
	RUST_LOG=warn ./scripts/codegen-with-staged-exiftool.sh ; ret=$$? ; \
	if [ $$ret -eq 0 ]; then \
		echo "🔧 Reformatting generated Rust code..." ; \
		cd .. && cargo fmt -- src/generated/**/*.rs 2>/dev/null ; \
	fi ; \
	exit $$ret

clean:
	cargo clean
	rm -rf tests/generated

check:
	cargo check -p codegen

# The eval makes local::lib Perl modules (PPI) visible to tests that shell out
# to scripts/ppi_ast.pl, matching codegen-with-staged-exiftool.sh.
test: generate-expression-tests
	eval "$$(perl -I "$$HOME/perl5/lib/perl5" -Mlocal::lib)" && cargo test -p codegen

# Fast Expression Development Targets (P07 Implementation)

# Debug single expression pipeline (1-2 seconds)
debug-expression:
	@if [ -z "$(EXPR)" ]; then \
		echo "Usage: make debug-expression EXPR='sprintf(\"%.2f V\", \$$val)'"; \
		exit 1; \
	fi
	@echo "🔍 Debugging expression: $(EXPR)"
	cargo run --bin debug-ppi -- '$(EXPR)'

# Test single expression file (5-10 seconds end-to-end)
test-expression-file:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make test-expression-file FILE=tests/config/print_conv/voltage.json"; \
		exit 1; \
	fi
	@echo "🔍 Testing expression from $(FILE)..."
	mkdir -p tests/generated
	cargo run --bin generate-expression-tests -- \
		--file "$(FILE)" --output "tests/generated/"
	@echo "🧪 Running generated tests..."
	cargo test --test generated_expressions
	@echo "✅ Expression test complete"

# Generate all expression test files from tests/config/ directory
generate-expression-tests:
	@echo "🔍 Generating all expression tests..."
	mkdir -p tests/generated
	find tests/generated -maxdepth 3 -name "*.rs" -type f -delete
	cargo run --bin generate-expression-tests -- \
		--dir tests/config/ --output "tests/generated/"
	@echo "✅ All expression test files generated"

# Test specific expression type (print_conv, value_conv, conditions)
test-expressions-type:
	@if [ -z "$(TYPE)" ]; then \
		echo "Usage: make test-expressions-type TYPE=print_conv"; \
		exit 1; \
	fi
	@echo "🔍 Testing $(TYPE) expressions..."
	mkdir -p tests/generated
	cargo run --bin generate-expression-tests -- \
		--dir "tests/config/$(TYPE)/" --output "tests/generated/"
	@echo "🧪 Running $(TYPE) tests..."
	cargo test --test generated_expressions
	@echo "✅ $(TYPE) tests complete"
