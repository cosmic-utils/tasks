#!/bin/bash
set -e
# Vendor dependencies
just vendor

# Build the flatpak
flatpak-builder --force-clean --jobs=1 -v build-dir com.github.digit1024.ms-todo-app.json
