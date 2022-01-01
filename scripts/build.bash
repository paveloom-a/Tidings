#!/bin/bash

# This script builds the application

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
    cargo build --manifest-path "${CARGO_MANIFEST}"
    echo ''
    cp "${CARGO_TARGET_DIR}"/debug/"${PROJECT_NAME}" "${OUTPUT}"
else
    echo -e "\n    RELEASE BUILD\n"
    cargo build --release --manifest-path "${CARGO_MANIFEST}"
    echo ''
    cp "${CARGO_TARGET_DIR}"/release/"${PROJECT_NAME}" "${OUTPUT}"
fi
