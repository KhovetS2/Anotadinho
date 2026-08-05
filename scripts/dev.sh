#!/usr/bin/env bash
# Roda o Anotadinho em modo dev.
# Requer: cargo, tauri-cli, trunk, rust target wasm32-unknown-unknown

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Instala targets se faltarem
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Instala tauri-cli se faltar
if ! command -v cargo-tauri >/dev/null 2>&1; then
    echo "Instalando tauri-cli..."
    cargo install tauri-cli --version "^2.0" --locked
fi

# Instala trunk se faltar
if ! command -v trunk >/dev/null 2>&1; then
    echo "Instalando trunk..."
    cargo install trunk --locked
fi

cd "$ROOT_DIR/src-tauri"
exec cargo tauri dev
