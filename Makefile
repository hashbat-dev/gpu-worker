.PHONY: help build test clean fmt lint check run docker-build docker-run bench coverage install-tools

# Default target
help:
	@echo "GPU Worker Development Commands"
	@echo "==============================="
	@echo "make build        - Build the project in debug mode"
	@echo "make release      - Build the project in release mode"
	@echo "make test         - Run all tests"
	@echo "make test-quick   - Run tests quickly (skip slow tests)"
	@echo "make clean        - Clean build artifacts"
	@echo "make fmt          - Format code with rustfmt"
	@echo "make lint         - Run clippy lints"
	@echo "make check        - Run format check and lints"
	@echo "make run          - Run the service locally"
	@echo "make docker-build - Build Docker image"
	@echo "make docker-run   - Run Docker container"
	@echo "make bench        - Run benchmarks"
	@echo "make coverage     - Generate code coverage report"
	@echo "make install-tools - Install development tools"
	@echo "make ci           - Run CI pipeline locally"
	@echo "make docs         - Generate and open documentation"

# Build targets
build:
	cargo build --all

release:
	cargo build --release --all

# Test targets
test:
	cargo test --all --verbose

test-quick:
	./run_tests.sh --quick

test-integration:
	cargo test --all --test '*' -- --nocapture

test-unit:
	cargo test --all --lib --bins

# Code quality
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt-check lint
	cargo check --all

# Run targets
run:
	RUST_LOG=info cargo run

run-release:
	RUST_LOG=info cargo run --release

run-debug:
	RUST_LOG=debug cargo run

# Docker targets
docker-build:
	docker build -t gpu-worker:latest .

docker-run:
	docker run -p 8080:8080 --rm gpu-worker:latest

docker-push: docker-build
	docker tag gpu-worker:latest gpu-worker:$(shell git rev-parse --short HEAD)
	@echo "Tagged as gpu-worker:$(shell git rev-parse --short HEAD)"

# Benchmarks
bench:
	cargo bench

bench-compare:
	cargo bench -- --save-baseline new
	cargo bench -- --baseline new --compare

# Coverage
coverage:
	cargo tarpaulin --out Html --all-features --workspace

coverage-ci:
	cargo tarpaulin --out Xml --all-features --workspace

# Documentation
docs:
	cargo doc --all --no-deps --open

docs-all:
	cargo doc --all --open

# Development tools
install-tools:
	rustup component add rustfmt clippy
	cargo install cargo-tarpaulin cargo-audit cargo-outdated cargo-edit

update-deps:
	cargo update
	cargo outdated

audit:
	cargo audit

# Clean
clean:
	cargo clean
	rm -rf tarpaulin-report.html cobertura.xml

clean-all: clean
	rm -rf Cargo.lock

# CI simulation
ci: fmt-check lint test coverage-ci audit
	@echo "CI pipeline passed!"

# Development workflow
dev-check: fmt lint test-quick
	@echo "Quick development check passed!"

# Utility targets
watch:
	cargo watch -x check -x test -x run

tree:
	tree -I 'target|.git|node_modules' -a

loc:
	tokei . --exclude target --exclude .git