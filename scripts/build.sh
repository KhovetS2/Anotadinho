#!/usr/bin/env bash
# Build release do Anotadinho: GUI (Tauri) + CLI headless
# (anotadinho-cli, ciclo 110) lado a lado — sem o CLI, quem builda a
# aplicação fica sem a peça usada por um agente pra operar o vault
# sem a janela aberta (ciclo 114).

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Buildando anotadinho-cli (release)..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" -p anotadinho-cli || exit 1

echo "==> Buildando a GUI (cargo tauri build)..."
( cd "$ROOT_DIR/src-tauri" && cargo tauri build )
GUI_BUILD_STATUS=$?

GUI_DIR="$ROOT_DIR/src-tauri/target/release"
CLI_BIN="$ROOT_DIR/target/release/anotadinho-cli"

# `cargo tauri build` pode falhar só no empacotamento de um formato
# específico (ex: AppImage precisa de FUSE + downloads externos —
# comum não estar disponível num sandbox/CI) mesmo com o binário da
# GUI compilado com sucesso. Não aborta o script nesse caso — só avisa
# — mas aborta de verdade se o binário nem existe.
if [[ ! -x "$GUI_DIR/anotadinho" ]]; then
    echo "ERRO: binário da GUI não foi gerado em $GUI_DIR/anotadinho" >&2
    exit 1
fi

cp "$CLI_BIN" "$GUI_DIR/anotadinho-cli"

# Cross-compile pro Windows (raw `cargo build`, sem passar pelo bundler do
# tauri — que precisa de NSIS/WiX rodando em Windows de verdade). O
# `ui/dist` já está fresco pelo `cargo tauri build` acima, que roda o
# `beforeBuildCommand` (trunk); o build.rs do tauri embute esses assets
# no binário na hora da compilação, então o `cargo build` abaixo pega o
# mesmo frontend sem precisar rodar o trunk de novo.
WIN_TARGET="x86_64-pc-windows-gnu"
WIN_DIR="$ROOT_DIR/src-tauri/target/$WIN_TARGET/release"
WIN_BUILD_OK=0
echo ""
echo "==> Buildando para Windows (cross-compile, target $WIN_TARGET)..."
if ! rustup target list --installed 2>/dev/null | grep -qx "$WIN_TARGET"; then
    echo "    Aviso: target $WIN_TARGET não instalado (rustup target add $WIN_TARGET) — pulando build Windows."
elif ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    echo "    Aviso: mingw-w64 (pacote gcc-mingw-w64-x86-64) não encontrado — pulando build Windows."
else
    # `--features tauri/custom-protocol` é o que o `cargo tauri build`
    # liga por baixo dos panos: sem ela o binário compila em modo dev e
    # tenta abrir http://localhost:1420 (o servidor do `trunk serve`) em
    # vez de carregar os assets embutidos — tela em branco com
    # ERR_CONNECTION_REFUSED, já que não tem dev server rodando.
    ( cd "$ROOT_DIR/src-tauri" && cargo build --release --target "$WIN_TARGET" --features tauri/custom-protocol )
    if [[ -f "$WIN_DIR/anotadinho.exe" ]]; then
        WIN_BUILD_OK=1
    else
        echo "    Aviso: build Windows terminou mas $WIN_DIR/anotadinho.exe não foi gerado."
    fi
fi

WIN_DIST="$ROOT_DIR/dist-windows"
if [[ $WIN_BUILD_OK -eq 1 ]]; then
    rm -rf "$WIN_DIST"
    mkdir -p "$WIN_DIST"
    cp "$WIN_DIR/anotadinho.exe" "$WIN_DIST/"
    DLL="$(find "$WIN_DIR" -maxdepth 1 -iname 'WebView2Loader.dll' | head -1)"
    if [[ -n "$DLL" ]]; then
        cp "$DLL" "$WIN_DIST/"
    else
        echo "    Aviso: WebView2Loader.dll não achado em $WIN_DIR — copie na mão antes de testar no Windows."
    fi
fi

# Linka o anotadinho-cli em ~/.local/bin pra ficar disponível no PATH sem
# caminho completo — é o binário que um agente (ex: Claude Code no WSL)
# chama pra operar o vault sem abrir a janela (ciclo 114).
LOCAL_BIN="$HOME/.local/bin"
CLI_LINKED=0
if [[ -d "$LOCAL_BIN" ]]; then
    ln -sf "$CLI_BIN" "$LOCAL_BIN/anotadinho-cli"
    CLI_LINKED=1
fi

if [[ $GUI_BUILD_STATUS -ne 0 ]]; then
    echo ""
    echo "==> Aviso: 'cargo tauri build' terminou com erro ao empacotar algum"
    echo "    instalador específico (ex: AppImage sem FUSE disponível) — o"
    echo "    binário da GUI e os pacotes que deram certo continuam em"
    echo "    $GUI_DIR/bundle/."
fi

echo ""
echo "==> Build completo."
echo "    GUI: $GUI_DIR/anotadinho (+ instaladores em $GUI_DIR/bundle/)"
echo "    CLI: $GUI_DIR/anotadinho-cli"
if [[ $CLI_LINKED -eq 1 ]]; then
    echo "    CLI também linkado em $LOCAL_BIN/anotadinho-cli (PATH) — use \`anotadinho-cli --vault <path> ...\`"
else
    echo "    Aviso: $LOCAL_BIN não existe — CLI não foi linkado no PATH."
fi
if [[ $WIN_BUILD_OK -eq 1 ]]; then
    echo "    Windows: $WIN_DIST/ (anotadinho.exe + WebView2Loader.dll — sem instalador, só o binário cru)"
fi
