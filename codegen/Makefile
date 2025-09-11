# Modular Codegen Makefile
# Simplified build system - Rust now handles extraction orchestration internally

.PHONY: all clean codegen-flocked codegen test debug-expression test-expression-file generate-expression-tests test-expressions-type

# Default target
all: codegen

# Main codegen pipeline (Rust now handles everything including patch cleanup)
codegen:
	@flock --exclusive --timeout=30 .make.flock $(MAKE) --no-print-directory codegen-flocked

# We can't have codegen-flocked depend on `clean`, as that breaks concurrent builds. 
codegen-flocked:
	@echo "🔧 Applying universal ExifTool patching..." ; \
	./scripts/exiftool-patcher.sh ; \
	echo "🚀 Running Rust codegen..." ; \
	RUST_LOG=info cargo run ; ret=$$? ; \
	if [ $$ret -eq 0 ]; then \
		echo "🔧 Reformatting generated Rust code..." ; \
		cd .. && cargo fmt -- src/generated/**/*.rs 2>/dev/null ; \
	fi ; \
	exit $$ret

clean:
	cargo clean
	./scripts/exiftool-patcher-undo.sh
	rm -rf tests/generated

check:
	cargo check -p codegen

test: generate-expression-tests
	cargo test -p codegen

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
