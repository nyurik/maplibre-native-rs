#!/usr/bin/env just --justfile

@_default:
    just --list

# Clean all build artifacts
clean:
    cargo clean
    rm -f Cargo.lock

# Update all dependencies, including the breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:
    cargo msrv find --write-msrv --ignore-lockfile

# Run cargo clippy to lint the code
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo +nightly &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Build and open code documentation
docs:
    cargo doc --no-deps --open

# Quick compile without building a binary
check:
    RUSTFLAGS='-D warnings' cargo check --workspace --all-targets

# Build the library
build:
    RUSTFLAGS='-D warnings' cargo build --workspace --all-targets

# Run the demo binary
run:
    cargo run -p render

# Run all tests
test:
    cargo test --all-targets --workspace

# Run all tests and accept the changes. Requires cargo-insta to be installed.
test-accept:
    cargo insta test --accept

# Test documentation
test-doc:
    RUSTDOCFLAGS="-D warnings" cargo test --doc
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

test-publishing:
    cargo publish --dry-run

package:
    cargo package

# Print Rust version information
@rust-info:
    rustc --version
    cargo --version
    echo "PWD $(pwd)"

# Run all tests as expected by CI
ci-test: rust-info test-fmt clippy build test test-doc

# Run minimal subset of tests to ensure compatibility with MSRV (Minimum Supported Rust Version). This assumes the default toolchain is already set to MSRV.
ci-test-msrv: rust-info build test

# Verify that the current version of the crate is not the same as the one published on crates.io
check-if-published: (assert "jq")
    #!/usr/bin/env bash
    LOCAL_VERSION="$(cargo metadata --format-version 1 | jq -r '.resolve.root | sub(".*@"; "")')"
    echo "Detected crate version:  $LOCAL_VERSION"
    CRATE_NAME="$(cargo metadata --format-version 1 | jq -r '.resolve.root | sub(".*#"; "") | sub("@.*"; "")')"
    echo "Detected crate name:     $CRATE_NAME"
    PUBLISHED_VERSION="$(cargo search ${CRATE_NAME} | grep "^${CRATE_NAME} =" | sed -E 's/.* = "(.*)".*/\1/')"
    echo "Published crate version: $PUBLISHED_VERSION"
    if [ "$LOCAL_VERSION" = "$PUBLISHED_VERSION" ]; then
        echo "ERROR: The current crate version has already been published."
        exit 1
    else
        echo "The current crate version has not yet been published."
    fi

# Ensure that a certain command is available
[private]
assert $COMMAND:
    @if ! type "{{COMMAND}}" > /dev/null; then \
        echo "Command '{{COMMAND}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi
