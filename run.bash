#!/bin/bash

# This script builds the application and runs it

# Exit when any command fails
set -e

# Name of the application
NAME=tidings

# Parse the profile
PROFILE="$1"
if [ "${PROFILE}" != "--release" ] && [ "${PROFILE}" != "" ]; then
    echo "Only the \`--release\` flag or an empty flag are allowed."
    exit 1
fi

# Redefine the `PROFILE` variable
if [ "${PROFILE}" = "--release" ]; then
    PROFILE=release
else
    PROFILE=dev
fi

# Define the ID of the app
if [ "${PROFILE}" = "release" ]; then
    ID=paveloom.apps."${NAME}"
else
    ID=paveloom.apps."${NAME}".dev
fi

# Define the paths
ROOT="$(dirname "$(realpath -s "$0")")"
if [ "${PROFILE}" = "release" ]; then
    TARGET="${ROOT}"/target/flatpak/release
else
    TARGET="${ROOT}"/target/flatpak/debug
fi
MANIFEST="${ROOT}"/manifests/"${PROFILE}".yml

# Append to the `PATH`
_PATH=$PATH:$(yq eval .build-options.append-path "${MANIFEST}")

# Parse the build args
mapfile -t BUILD_ARGS < <(yq eval .build-options.build-args[] "${MANIFEST}")
mapfile -t TEST_ARGS < <(yq eval .build-options.test-args[] "${MANIFEST}")
mapfile -t FINISH_ARGS < <(yq eval .finish-args[] "${MANIFEST}")
mapfile -t CONFIG_OPTS < <(yq eval .modules[0].config-opts[] "${MANIFEST}")

# Define the general args
GENERAL_ARGS=(
    --nofilesystem=host
    --filesystem="${ROOT}"
    --filesystem="${TARGET}"
    --env=PATH="${_PATH}"
    --env=LD_LIBRARY_PATH=/app/lib
    --env=PKG_CONFIG_PATH=/app/lib/pkgconfig:/app/share/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig
)

# Execute the build command in the build environment
flatpak-build() {
    flatpak build \
        "${BUILD_ARGS[@]}" \
        "${GENERAL_ARGS[@]}" \
        "${TARGET}" "$@"
}

# Execute the test command in the build environment
flatpak-test() {
    flatpak build \
        "${TEST_ARGS[@]}" \
        "${GENERAL_ARGS[@]}" \
        "${TARGET}" "$@"
}

if [ ! -d "${TARGET}" ]; then
    # Initialize a directory for building
    flatpak build-init \
        "${TARGET}" \
        "${ID}" \
        "$(yq eval .sdk "${MANIFEST}")" \
        "$(yq eval .runtime "${MANIFEST}")" \
        "$(yq eval .runtime-version "${MANIFEST}")"
    # Download the dependencies
    flatpak-builder \
        --ccache \
        --force-clean \
        --disable-updates \
        --download-only \
        --state-dir="${ROOT}"/.flatpak-builder \
        --stop-at="${NAME}" \
        "${TARGET}" \
        "${MANIFEST}"
    # Build the dependencies
    flatpak-builder \
        --ccache \
        --force-clean \
        --disable-updates \
        --disable-download \
        --build-only \
        --keep-build-dirs \
        --state-dir="${ROOT}"/.flatpak-builder \
        --stop-at="${NAME}" \
        "${TARGET}" \
        "${MANIFEST}"
    # Configure the build
    flatpak-build meson setup --prefix /app "${TARGET}" "${CONFIG_OPTS[@]}"
fi

# Build the application
flatpak-build meson compile -C "${TARGET}"
# Validate the resources
flatpak-test meson test -qC "${TARGET}"
# Install the binary and resources
flatpak-build meson install --quiet -C "${TARGET}"

# Run the application
flatpak build \
    --with-appdir \
    "${FINISH_ARGS[@]}" \
    "${TARGET}" \
    "${NAME}"
