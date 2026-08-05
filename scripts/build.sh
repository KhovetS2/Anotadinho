#!/usr/bin/env bash
# Build release do Anotadinho.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR/src-tauri"

exec cargo tauri build
