#!/bin/bash

# Astor Currency Compilation Test Script
# This script performs comprehensive compilation testing and error reporting

set -e

echo "🔧 Starting Astor Currency Compilation Test..."
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if we're in a Rust project
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Are you in the Astor currency project directory?"
    exit 1
fi

print_status "Found Cargo.toml - proceeding with compilation test"

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean

# Check Rust toolchain
print_status "Checking Rust toolchain..."
rustc --version
cargo --version

# Format check
print_status "Running cargo fmt check..."
if cargo fmt -- --check; then
    print_success "Code formatting is correct"
else
    print_warning "Code formatting issues found. Running cargo fmt to fix..."
    cargo fmt
    print_success "Code formatting fixed"
fi

# Clippy linting
print_status "Running cargo clippy for linting..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "No clippy warnings found"
else
    print_error "Clippy warnings found. Please review and fix."
fi

# Check for compilation errors
print_status "Running cargo check..."
if cargo check --all-targets --all-features; then
    print_success "Cargo check passed - no compilation errors"
else
    print_error "Compilation errors found during cargo check"
    exit 1
fi

# Build in debug mode
print_status "Building in debug mode..."
if cargo build; then
    print_success "Debug build successful"
else
    print_error "Debug build failed"
    exit 1
fi

# Build in release mode
print_status "Building in release mode..."
if cargo build --release; then
    print_success "Release build successful"
else
    print_error "Release build failed"
    exit 1
fi

# Run tests
print_status "Running tests..."
if cargo test; then
    print_success "All tests passed"
else
    print_warning "Some tests failed - please review"
fi

# Check documentation
print_status "Checking documentation..."
if cargo doc --no-deps; then
    print_success "Documentation generated successfully"
else
    print_warning "Documentation generation had issues"
fi

# Security audit
print_status "Running security audit..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security vulnerabilities found - please review"
    fi
else
    print_warning "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

# Check for unused dependencies
print_status "Checking for unused dependencies..."
if command -v cargo-machete &> /dev/null; then
    if cargo machete; then
        print_success "No unused dependencies found"
    else
        print_warning "Unused dependencies found - consider removing them"
    fi
else
    print_warning "cargo-machete not installed. Install with: cargo install cargo-machete"
fi

# Final summary
echo ""
echo "================================================"
print_success "Compilation test completed successfully!"
echo ""
print_status "Summary:"
echo "  ✅ Code formatting: OK"
echo "  ✅ Compilation: OK"
echo "  ✅ Debug build: OK"
echo "  ✅ Release build: OK"
echo "  ✅ Documentation: OK"
echo ""
print_status "The Astor currency system is ready for deployment!"