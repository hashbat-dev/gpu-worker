#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Function to print colored output
print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

# Parse command line arguments
RUN_BENCH=false
RUN_COVERAGE=false
VERBOSE=false
QUICK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --bench)
            RUN_BENCH=true
            shift
            ;;
        --coverage)
            RUN_COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --quick)
            QUICK=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --bench      Run benchmarks"
            echo "  --coverage   Generate code coverage report"
            echo "  --verbose    Show verbose output"
            echo "  --quick      Run only essential tests"
            echo "  --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Start testing
echo "GPU Worker Test Suite"
echo "===================="
echo ""

# Check Rust toolchain
print_step "Checking Rust toolchain"
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust."
    exit 1
fi

RUST_VERSION=$(rustc --version | cut -d' ' -f2)
print_success "Rust $RUST_VERSION found"

# Clean previous test artifacts
if [ "$QUICK" = false ]; then
    print_step "Cleaning previous test artifacts"
    cargo clean
    print_success "Clean complete"
fi

# Format check
print_step "Checking code formatting"
if cargo fmt --all -- --check; then
    print_success "Code formatting check passed"
else
    print_error "Code formatting check failed"
    print_warning "Run 'cargo fmt --all' to fix formatting"
    exit 1
fi

# Clippy lints
print_step "Running clippy lints"
if [ "$VERBOSE" = true ]; then
    cargo clippy --all-targets --all-features -- -D warnings
else
    cargo clippy --all-targets --all-features -- -D warnings 2>&1 | grep -E "(error|warning)" || true
fi

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    print_success "Clippy check passed"
else
    print_error "Clippy check failed"
    exit 1
fi

# Build check
print_step "Building project"
if [ "$VERBOSE" = true ]; then
    cargo build --all --all-features
else
    cargo build --all --all-features --quiet
fi
print_success "Build successful"

# Unit tests
print_step "Running unit tests"
if [ "$VERBOSE" = true ]; then
    cargo test --all --lib --bins -- --nocapture
else
    cargo test --all --lib --bins --quiet
fi
print_success "Unit tests passed"

# Integration tests
print_step "Running integration tests"
if [ "$VERBOSE" = true ]; then
    cargo test --all --test '*' -- --nocapture
else
    cargo test --all --test '*' --quiet
fi
print_success "Integration tests passed"

# Doc tests
if [ "$QUICK" = false ]; then
    print_step "Running documentation tests"
    if [ "$VERBOSE" = true ]; then
        cargo test --all --doc
    else
        cargo test --all --doc --quiet
    fi
    print_success "Documentation tests passed"
fi

# Example tests
if [ "$QUICK" = false ]; then
    print_step "Testing examples"
    cargo test --examples --quiet
    print_success "Example tests passed"
fi

# Feature tests
if [ "$QUICK" = false ]; then
    print_step "Testing with different feature combinations"
    # Test with no default features
    cargo test --no-default-features --quiet
    # Test with all features
    cargo test --all-features --quiet
    print_success "Feature tests passed"
fi

# Benchmarks
if [ "$RUN_BENCH" = true ]; then
    print_step "Running benchmarks"
    cargo bench --quiet
    print_success "Benchmarks complete"
    print_warning "Benchmark results saved in target/criterion/"
fi

# Coverage
if [ "$RUN_COVERAGE" = true ]; then
    print_step "Generating code coverage"
    
    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    fi
    
    # Run coverage
    if [ "$VERBOSE" = true ]; then
        cargo tarpaulin --out Html --all-features --workspace
    else
        cargo tarpaulin --out Html --all-features --workspace --print-summary
    fi
    
    print_success "Coverage report generated"
    print_warning "Coverage report saved in tarpaulin-report.html"
fi

# Security audit
if [ "$QUICK" = false ]; then
    print_step "Running security audit"
    
    # Check if cargo-audit is installed
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not found. Installing..."
        cargo install cargo-audit
    fi
    
    if cargo audit; then
        print_success "No security vulnerabilities found"
    else
        print_warning "Security vulnerabilities detected"
    fi
fi

# Check for outdated dependencies
if [ "$QUICK" = false ]; then
    print_step "Checking for outdated dependencies"
    
    # Check if cargo-outdated is installed
    if command -v cargo-outdated &> /dev/null; then
        cargo outdated
    else
        print_warning "cargo-outdated not found. Skipping dependency check."
    fi
fi

# Release build test
if [ "$QUICK" = false ]; then
    print_step "Testing release build"
    if [ "$VERBOSE" = true ]; then
        cargo build --release --all
    else
        cargo build --release --all --quiet
    fi
    print_success "Release build successful"
fi

# Summary
echo ""
echo "===================="
echo "Test Summary"
echo "===================="
print_success "All tests passed!"

if [ "$RUN_COVERAGE" = true ]; then
    echo ""
    echo "Coverage Report: file://$(pwd)/tarpaulin-report.html"
fi

if [ "$RUN_BENCH" = true ]; then
    echo ""
    echo "Benchmark Report: file://$(pwd)/target/criterion/report/index.html"
fi

echo ""
echo "Next steps:"
echo "- Review any warnings above"
echo "- Run with --coverage for code coverage"
echo "- Run with --bench for performance benchmarks"
echo "- Use --verbose for detailed output"

exit 0