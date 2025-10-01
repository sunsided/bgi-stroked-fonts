# Define environment variables

set dotenv-load := true

# Show help.
[private]
help:
    @just --list --unsorted

# Update pre-commit
update-pre-commit:
    pre-commit autoupdate

# Update dependencies
update: update-pre-commit
    cargo update

# Format the code.
fmt:
    cargo sort --workspace
    cargo fmt --all

# Lint the code
lint *ARGS:
    cargo clippy --all --all-targets {{ ARGS }}

# Lint the code and apply fixes.
lint-fix *ARGS:
    cargo clippy --all --all-targets --fix --allow-staged {{ ARGS }}
    cargo fmt --all

# Run checks on the code.
check:
    cargo fmt --all --check

# Sweep the target directory
sweep:
    cargo sweep --installed

# Clean all build artifacts (use sweep for a lightweight clean)
clean:
    cargo clean

# Build in debug mode.
build:
    cargo build

convert:
    cargo run --no-default-features --features=_convert --bin convert $(pwd)/c
    mv $(pwd)/c/*.rs src/
    cargo fmt --all

# Run all tests.
test:
    RUST_LOG="disabled" cargo test --all

# Run doctests only.
test-docs:
    RUST_LOG="disabled" cargo test --all --doc

# Upgrade the dependencies.
upgrade-deps:
    cargo upgrade
    cargo update

# Build the documentation; pass --open to also open it.
docs *ARGS:
    rm -rf target/doc
    cargo doc --workspace --document-private-items --no-deps {{ ARGS }}

# Build and open the docs
[private]
docs-open *ARGS:
    @just open-docs {{ ARGS }}

# Build and open the docs
open-docs *ARGS:
    @just docs --open {{ ARGS }}
