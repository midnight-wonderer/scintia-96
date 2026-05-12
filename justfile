# Run all tests with default features
test:
    cargo test

# Run all tests with all features enabled
test-all:
    cargo test --all-features

# Run tests for specific feature sets
test-no-features:
    cargo test --no-default-features

test-cipher:
    cargo test --features cipher

# Run clippy for all features
lint:
    cargo clippy --all-features -- -D warnings

# Check compilation for all features
check:
    cargo check --all-features

# Format the code
fmt *args:
    cargo fmt --all {{args}}
