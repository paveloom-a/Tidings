#!/bin/bash

# This script updates resources caches after installing

# Define the path to the data
DATA="${MESON_INSTALL_PREFIX}"/share

# Update icon cache
gtk4-update-icon-cache -qtf "${DATA}"/icons/hicolor
# Compile new schemas
glib-compile-schemas "${DATA}"/glib-2.0/schemas
# Update desktop database
update-desktop-database "${DATA}"/applications
