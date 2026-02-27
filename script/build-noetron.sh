#!/usr/bin/env bash
# build-noetron.sh — Build only the Noetron Studio crates (works on 1–2 GB RAM)
#
# Usage:
#   ./script/build-noetron.sh          # debug build
#   ./script/build-noetron.sh release  # release build (smaller binary)
#   ./script/build-noetron.sh check    # type-check only (fastest, ~2 min)

set -e

PROFILE="${1:-debug}"
CRATES="-p noetron_aiproj -p noetron_ir -p noetron_db -p noetron_executor -p noetron_toggle -p noetron_ui"

# Use 1 job to keep peak RAM below ~1.5 GB
JOBS=1

echo "==> Building Noetron Studio crates (profile: $PROFILE, jobs: $JOBS)"
echo "    This avoids compiling Zed's heavy WASM runtime (cranelift-codegen)."
echo ""

case "$PROFILE" in
  check)
    cargo check $CRATES --jobs $JOBS
    ;;
  release)
    cargo build $CRATES --release --jobs $JOBS
    ;;
  *)
    cargo build $CRATES --jobs $JOBS
    ;;
esac

echo ""
echo "==> Done! Noetron Studio crates built successfully."
echo ""
echo "    To build the full Zed binary (needs 8+ GB RAM):"
echo "      cargo build -p zed --jobs 1"
echo ""
echo "    To add swap space before full build (Linux):"
echo "      sudo fallocate -l 8G /swapfile && sudo chmod 600 /swapfile"
echo "      sudo mkswap /swapfile && sudo swapon /swapfile"
echo "      cargo build -p zed --jobs 1"
echo "      sudo swapoff /swapfile && sudo rm /swapfile"
