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

# Garante target wasm32.
#
# Olha a biblioteca do alvo no sysroot do rustc em vez de perguntar ao
# rustup: um Rust de distro (o `rust` do Fedora, por exemplo) não tem
# rustup, e o alvo vem num pacote à parte
# (`rust-std-static-wasm32-unknown-unknown`). Perguntar só ao rustup
# fazia o script morrer com o alvo já instalado.
if [ ! -d "$(rustc --print sysroot)/lib/rustlib/wasm32-unknown-unknown" ]; then
    if command -v rustup >/dev/null 2>&1; then
        rustup target add wasm32-unknown-unknown || {
            echo "ERRO: não foi possível instalar o target wasm32-unknown-unknown" >&2
            exit 1
        }
    else
        echo "ERRO: falta o target wasm32-unknown-unknown e não há rustup." >&2
        echo "Num Rust de distro, instale o pacote do alvo, por exemplo:" >&2
        echo "  sudo dnf install rust-std-static-wasm32-unknown-unknown" >&2
        exit 1
    fi
fi

# Verifica tauri-cli
if ! command -v cargo-tauri >/dev/null 2>&1; then
    echo "Instalando tauri-cli..."
    # `--locked`: sem ele o cargo resolve dependências mais novas que o
    # Cargo.lock publicado, e elas podem pedir um rustc mais novo que o
    # instalado.
    cargo install tauri-cli --version "^2.0" --locked
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
