#!/usr/bin/env bash
# Roda o Anotadinho em modo dev.
# Requer: cargo, tauri-cli, trunk, rust target wasm32-unknown-unknown

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Garante ambiente Rust (rustup + ~/.cargo/bin)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"

# Garante target wasm32
if ! rustup target list --installed 2>/dev/null | grep -qF wasm32-unknown-unknown; then
    rustup target add wasm32-unknown-unknown 2>/dev/null || {
        echo "ERRO: não foi possível instalar o target wasm32-unknown-unknown" >&2
        echo "Verifique se o rustup está instalado e tente manualmente:" >&2
        echo "  rustup target add wasm32-unknown-unknown" >&2
        exit 1
    }
fi

# Verifica tauri-cli
if ! command -v cargo-tauri >/dev/null 2>&1; then
    echo "Instalando tauri-cli..."
    cargo install tauri-cli --version "^2.0"
fi

# Verifica trunk (prefere cargo-binstall pra binário)
if ! command -v trunk >/dev/null 2>&1; then
    if command -v cargo-binstall >/dev/null 2>&1; then
        echo "Instalando trunk (via cargo-binstall)..."
        cargo binstall trunk -y
    else
        echo "Instalando cargo-binstall..."
        cargo install cargo-binstall
        echo "Instalando trunk (via cargo-binstall)..."
        cargo binstall trunk -y
    fi
fi

echo "Iniciando Anotadinho em modo dev..."
cd "$ROOT_DIR/src-tauri"
exec cargo tauri dev
