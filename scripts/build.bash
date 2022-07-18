#!/bin/bash

# This script builds the binary crate

# Exit when any command fails
set -e

# Parse the arguments
MESON_BUILD_ROOT="$1"
MESON_SOURCE_ROOT="$2"
OUTPUT="$3"
PROFILE="$4"
PROJECT_NAME="$5"

# Define Cargo subdirectories
export CARGO_TARGET_DIR="${MESON_BUILD_ROOT}"/target
CARGO_MANIFEST="${MESON_SOURCE_ROOT}"/Cargo.toml

# Define the Cargo Home directory
export CARGO_HOME="${CARGO_TARGET_DIR}"/cargo-home

# Build the chosen profile
if [[ "$PROFILE" = "dev" ]]; then
    echo -e "\n    DEV BUILD\n"
    # Use mold as a linker for dev builds
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang"
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=/usr/lib/sdk/rust-stable/bin/mold"
    # Build the crate
    cargo build --manifest-path "${CARGO_MANIFEST}"
    # Copy the binary
    cp "${CARGO_TARGET_DIR}"/debug/"${PROJECT_NAME}" "${OUTPUT}"
    echo
else
    echo -e "\n    RELEASE BUILD\n"
    # Build the crate
    cargo build --release --manifest-path "${CARGO_MANIFEST}"
    # Copy the binary
    cp "${CARGO_TARGET_DIR}"/release/"${PROJECT_NAME}" "${OUTPUT}"
    echo
fi
