.PHONY: check fmt lint test clippy doc build clean

# Run all checks before committing
check: fmt lint test

# Format code
fmt:
	cargo fmt --all -- --check

# Run clippy (Rust linter)
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo test --all

# Fix formatting
fix-fmt:
	cargo fmt --all

# Fix linting issues where possible
fix-lint:
	cargo clippy --all-targets --all-features --fix

# Build in release mode
build:
	cargo build --release

# Generate documentation
doc:
	cargo doc --no-deps --open

# Clean build artifacts
clean:
	cargo clean

# Help
help:
	@echo "Available targets:"
	@echo "  check     - Run fmt, lint, and test (pre-commit checks)"
	@echo "  fmt       - Check code formatting"
	@echo "  lint      - Run clippy linter"
	@echo "  test      - Run tests"
	@echo "  fix-fmt   - Fix code formatting"
	@echo "  fix-lint  - Fix linting issues"
	@echo "  build     - Build in release mode"
	@echo "  doc       - Generate and open documentation"
	@echo "  clean     - Clean build artifacts"