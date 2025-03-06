#!/usr/bin/env bash

# Extract version from Cargo.toml
version=$(grep '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
# Update pyproject.toml (Linux syntax; adjust for macOS if needed)
sed -i -E "s/version = \".*\"/version = \"$version\"/" pyproject.toml
