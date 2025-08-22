# Modular Codegen Makefile
# Simplified build system - Rust now handles extraction orchestration internally

.PHONY: all clean codegen-flocked codegen test

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
	@rm -rf "$$(python3 -c "import tempfile; print(tempfile.gettempdir())" 2>/dev/null || echo /tmp)/exif_oxide_test_cache" 2>/dev/null || true

check:
	cargo check -p codegen

test:
	cargo test -p codegen
