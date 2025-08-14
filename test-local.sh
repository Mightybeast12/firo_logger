#!/bin/bash

# Local testing script for firo_logger
# This script runs the same checks that the CI will run

set -e

echo "🧪 Running local pre-push checks for firo_logger..."
echo

echo "📋 1. Formatting check..."
cargo fmt -- --check
echo "✅ Formatting OK"
echo

echo "🔍 2. Linting (clippy)..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Linting OK"
echo

echo "🏗️ 3. Building..."
cargo build --all-features
echo "✅ Build OK"
echo

echo "🧪 4. Running tests..."
cargo test --all-features
echo "✅ Tests OK"
echo

echo "📚 5. Doc tests..."
cargo test --doc --all-features
echo "✅ Doc tests OK"
echo

echo "📖 6. Documentation..."
cargo doc --no-deps --all-features
echo "✅ Documentation OK"
echo

echo "📦 7. Package verification..."
cargo package --list > /dev/null
echo "✅ Package OK"
echo

echo "🔍 8. Publish dry run..."
cargo publish --dry-run
echo "✅ Publish dry run OK"
echo

echo "🎉 All checks passed! Ready for CI/CD"
echo
echo "To release:"
echo "1. Update version in Cargo.toml"
echo "2. Update CHANGELOG.md"
echo "3. git add . && git commit -m 'chore: release vX.X.X'"