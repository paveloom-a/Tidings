#!/bin/bash

# This script builds the application and runs it

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
    PROFILE=debug
fi

# Define the ID of the app
if [ "${PROFILE}" != "release" ]; then
    ID=paveloom.apps.tidings
else
    ID=paveloom.apps.tidings.debug
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

# Define the general args
GENERAL_ARGS=(
    --nofilesystem=host
    --filesystem="${ROOT}"
    --filesystem="${TARGET}"
    --filesystem="${TARGET}"
    --env=PATH="${_PATH}"
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
    # Configure the build
    flatpak-build meson setup --prefix /app "${TARGET}" --buildtype "${PROFILE}"
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
    --allow=devel \
    --bind-mount=/run/user/1000/doc=/run/user/1000/doc/by-app/"${ID}" \
    "${FINISH_ARGS[@]}" \
    "${TARGET}" \
    tidings