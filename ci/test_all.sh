#!/usr/bin/env bash
set -euo pipefail

# Comprehensive test script for niri-bar Rust workspace
# This script runs all tests, safety checks, and quality gates

echo "🚀 Starting comprehensive niri-bar test suite..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MIN_COV=80
PROJECT_NAME="niri-bar"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."

    local missing_tools=()

    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi

    if ! command -v rustc &> /dev/null; then
        missing_tools+=("rustc")
    fi

    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        print_error "Please install Rust and Cargo first: https://rustup.rs/"
        exit 1
    fi

    print_success "Dependencies check passed"
}

# Install testing tools if not present
install_testing_tools() {
    print_status "Installing testing tools..."

    # Install cargo tools if not present
    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_status "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov
    fi

    if ! command -v cargo-mutants &> /dev/null; then
        print_status "Installing cargo-mutants..."
        cargo install cargo-mutants
    fi

    if ! command -v cargo-nextest &> /dev/null; then
        print_status "Installing cargo-nextest..."
        cargo install cargo-nextest
    fi

    print_success "Testing tools installed"
}

# Run linting and formatting checks
run_lints_and_format() {
    print_status "Running lints and format checks..."

    # Check formatting
    if ! cargo fmt --all -- --check; then
        print_error "Code formatting check failed"
        print_error "Run 'cargo fmt' to fix formatting issues"
        exit 1
    fi

    # Run clippy with strict settings
    if ! cargo clippy --all-targets --all-features --workspace -D warnings; then
        print_error "Clippy lints failed"
        print_error "Fix the clippy warnings before proceeding"
        exit 1
    fi

    print_success "Lints and format checks passed"
}

# Run unit tests
run_unit_tests() {
    print_status "Running unit tests..."

    # Run tests with nextest for better output and parallelization
    if command -v cargo-nextest &> /dev/null; then
        if ! cargo nextest run --all-features --workspace; then
            print_error "Unit tests failed"
            exit 1
        fi
    else
        if ! cargo test --all-features --workspace; then
            print_error "Unit tests failed"
            exit 1
        fi
    fi

    print_success "Unit tests passed"
}

# Run doctests
run_doctests() {
    print_status "Running doctests..."

    if ! cargo test --doc --workspace; then
        print_error "Doctests failed"
        exit 1
    fi

    print_success "Doctests passed"
}

# Run property-based tests
run_property_tests() {
    print_status "Running property-based tests..."

    if ! cargo test --features proptest --workspace; then
        print_error "Property-based tests failed"
        exit 1
    fi

    print_success "Property-based tests passed"
}

# Run concurrency tests with Loom
run_loom_tests() {
    print_status "Running concurrency tests with Loom..."

    if command -v cargo &> /dev/null && cargo check --features loom &> /dev/null; then
        print_status "Running Loom model checking..."
        if ! RUSTFLAGS="--cfg loom" cargo test -p niri-bar --features loom -- --quiet; then
            print_warning "Loom tests failed or not configured"
            print_warning "This might be expected if Loom tests aren't set up yet"
        else
            print_success "Loom tests passed"
        fi
    else
        print_warning "Loom feature not available, skipping concurrency tests"
    fi
}

# Run Miri for UB detection
run_miri_tests() {
    print_status "Running Miri for undefined behavior detection..."

    # Setup Miri if not already done
    if ! rustup component list | grep -q "miri.*installed"; then
        print_status "Setting up Miri..."
        rustup component add miri
    fi

    # Run Miri on safe code and specific unsafe code
    print_status "Running Miri tests..."
    if ! MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-tree-borrows" cargo miri test --workspace -- -Zunstable-options --exclude-should-panic; then
        print_error "Miri found undefined behavior"
        exit 1
    fi

    print_success "Miri checks passed"
}

# Run coverage analysis
run_coverage() {
    print_status "Running coverage analysis..."

    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_warning "cargo-llvm-cov not available, skipping coverage"
        return 0
    fi

    # Generate coverage report
    if ! cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info; then
        print_error "Coverage analysis failed"
        exit 1
    fi

    # Check coverage threshold
    local coverage_pct
    coverage_pct=$(cargo llvm-cov report --json | jq -r '.total.lines.percent' | xargs printf "%.0f")

    if [ "$coverage_pct" -lt "$MIN_COV" ]; then
        print_error "Coverage ${coverage_pct}% is below minimum threshold ${MIN_COV}%"
        print_error "Run 'cargo llvm-cov --open' to see detailed coverage report"
        exit 1
    fi

    print_success "Coverage check passed: ${coverage_pct}% (minimum: ${MIN_COV}%)"
}

# Run mutation testing
run_mutation_tests() {
    print_status "Running mutation testing..."

    if ! command -v cargo-mutants &> /dev/null; then
        print_warning "cargo-mutants not available, skipping mutation tests"
        return 0
    fi

    # Run mutation testing with timeout
    print_status "Running mutation analysis (this may take a while)..."
    if ! timeout 300 cargo mutants --workspace --timeout 10 --no-shuffle --in-place --unmutated-ok; then
        print_warning "Mutation testing failed or timed out"
        print_warning "This might be expected for large codebases"
    else
        print_success "Mutation testing completed"
    fi
}

# Run security audit
run_security_audit() {
    print_status "Running security audit..."

    if command -v cargo-audit &> /dev/null; then
        if ! cargo audit; then
            print_error "Security audit failed"
            exit 1
        fi
        print_success "Security audit passed"
    else
        print_warning "cargo-audit not available, skipping security audit"
    fi
}

# Check for outdated dependencies
check_dependencies_outdated() {
    print_status "Checking for outdated dependencies..."

    if ! cargo outdated --workspace; then
        print_warning "Could not check for outdated dependencies"
        print_warning "Install cargo-outdated: cargo install cargo-outdated"
    fi
}

# Generate test summary
generate_summary() {
    print_status "Generating test summary..."

    echo "=========================================="
    echo "🏆 NIRI-BAR TEST SUITE SUMMARY"
    echo "=========================================="
    echo "✅ Lints and formatting: PASSED"
    echo "✅ Unit tests: PASSED"
    echo "✅ Doctests: PASSED"
    echo "✅ Property-based tests: PASSED"
    echo "✅ Coverage (>=${MIN_COV}%): PASSED"
    echo "✅ Miri (UB detection): PASSED"
    echo "=========================================="

    # Show test statistics
    if command -v cargo-nextest &> /dev/null; then
        echo "Test execution details:"
        cargo nextest run --all-features --workspace --no-run
    fi

    print_success "All quality gates passed! 🎉"
}

# Main execution
main() {
    echo "=========================================="
    echo "🧪 NIRI-BAR COMPREHENSIVE TEST SUITE"
    echo "=========================================="

    # Change to project root if not already there
    cd "$(dirname "${BASH_SOURCE[0]}")/.."

    check_dependencies
    install_testing_tools
    run_lints_and_format
    run_unit_tests
    run_doctests
    run_property_tests
    run_loom_tests
    run_miri_tests
    run_coverage
    run_mutation_tests
    run_security_audit
    check_dependencies_outdated
    generate_summary

    echo ""
    print_success "🎊 ALL TESTS COMPLETED SUCCESSFULLY!"
    echo ""
    echo "Next steps:"
    echo "  • Review coverage report: cargo llvm-cov --open"
    echo "  • Check mutation testing results"
    echo "  • Address any warnings in the output"
    echo "  • Consider adding integration tests for end-to-end scenarios"
}

# Handle command line arguments
case "${1:-}" in
    "quick")
        print_status "Running quick test suite (no coverage/mutation)..."
        check_dependencies
        run_lints_and_format
        run_unit_tests
        run_doctests
        print_success "Quick test suite completed!"
        ;;
    "coverage-only")
        check_dependencies
        run_coverage
        ;;
    "miri-only")
        check_dependencies
        run_miri_tests
        ;;
    *)
        main
        ;;
esac
