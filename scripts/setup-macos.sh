#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This setup helper is intended for macOS."
  exit 1
fi

missing=0
for command_name in node npm rustc cargo xcode-select; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo
  echo "Install Node.js 22+, the Rust stable toolchain, and Xcode Command Line Tools, then run this command again."
  exit 1
fi

if ! xcode-select -p >/dev/null 2>&1; then
  echo "Xcode Command Line Tools are not configured. Run: xcode-select --install"
  exit 1
fi

echo "Node:  $(node --version)"
echo "npm:   $(npm --version)"
echo "Rust:  $(rustc --version)"
echo "Cargo: $(cargo --version)"
echo

echo "Installing JavaScript dependencies..."
npm install

echo

echo "First-run setup is complete."
echo "Launch Imgen Pro with: npm run tauri dev"
echo "The app starts in mock image mode and expects a local text endpoint at http://127.0.0.1:8080/v1."
