#!/bin/bash

# Allow recursive glob patterns
shopt -s globstar

# This script creates a list of source files

ROOT=$(dirname "$(dirname "$0")")

for file in "${ROOT}"/src/**/*.rs; do
  echo "${file}"
done
